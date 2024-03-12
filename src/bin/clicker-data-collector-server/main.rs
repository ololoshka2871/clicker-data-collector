#![feature(async_iterator)]

mod handlers;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::FromRef,
    response::Redirect,
    routing::{get, put},
    Router,
};

use clicker_data_collector::data_model::DataModel;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

use axum_template::engine::Engine;

use minijinja::Environment;

use handlers::*;

pub(crate) type AppEngine = Engine<Environment<'static>>;

#[derive(Clone, FromRef)]
struct AppState {
    engine: AppEngine,
    config: clicker_data_collector::Config,
    config_file: std::path::PathBuf,

    data_model: Arc<Mutex<DataModel>>,
    clicker_ctrl: Arc<Mutex<clicker_data_collector::ClickerController>>,
}

fn float2dgt(value: String) -> String {
    if let Ok(v) = value.parse::<f32>() {
        format!("{v:.2}")
    } else {
        value
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Enable tracing using Tokio's https://tokio.rs/#tk-lib-tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clicker-data-collector-server=debug,clicker-data-collector=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let emulate_freq = std::env::var("EMULATE_FREQ")
        .map(|v| v.parse::<f32>().unwrap_or_default())
        .ok();
    if let Some(f) = &emulate_freq {
        tracing::warn!("Emulating frequency: {}", f);
    }

    tracing::info!("Loading config...");
    let (config, config_file) = clicker_data_collector::Config::load();
    tracing::info!("Config loaded:\n{}", config);

    let mut clicker = clicker_data_collector::FakeClicker::new(std::time::Duration::from_secs(1));
    /*
    let mut clicker = clicker_data_collector::Clicker::new(
        config.rk_meter_port.clone(),
        std::time::Duration::from_millis(250),
    );
    */
    tracing::warn!("Testing connection...");
    if let Err(e) = clicker.test().await {
        panic!("Failed to connect to clicker: {:?}", e);
    }
    let clicker_ctrl = clicker_data_collector::ClickerController::new(
        clicker,
        std::time::Duration::from_millis(250), // интервал опроса
        3, // цыклов переключения Rk -> Freq -> Rk для получения данных
    );

    // State for our application
    let mut minijinja = Environment::new();
    minijinja
        .add_template("work", include_str!("wwwroot/html/work.jinja"))
        .unwrap();
    minijinja
        .add_template("config", include_str!("wwwroot/html/config.jinja"))
        .unwrap();

    minijinja.add_filter("float2dgt", float2dgt);

    let web_port = config.web_port;

    let data_model = DataModel::default();

    let app_state = AppState {
        engine: Engine::from(minijinja),
        config,
        config_file,

        data_model: Arc::new(Mutex::new(data_model)),
        clicker_ctrl: Arc::new(Mutex::new(clicker_ctrl)),
    };

    // Build our application with some routes
    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/work") }))
        .route("/work", get(handle_get_work))
        .route(
            "/global",
            get(handle_get_globals)
                .put(handle_set_globals)
                .delete(handler_reset_globals),
        )
        .route("/report", get(handle_generate_report_excel))
        .route("/config", get(handle_config).patch(handle_config_and_save))
        //.route("/config-and-save", patch(handle_config_and_save))
        .route("/static/:path/:file", get(static_files::handle_static))
        .route("/lib/*path", get(static_files::handle_lib))
        // rest_api
        .route(
            "/Measurements",
            get(handle_measurements_get)
                .post(handle_measurements_append)
                .delete(handle_measurements_cancel),
        )
        .route(
            "/Measurements/:id",
            put(handle_measurements_put)
                .post(handle_measurements_insert)
                .delete(handle_measurements_delete),
        )
        .with_state(app_state)
        // Using tower to add tracing layer
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // In practice: Use graceful shutdown.
    // Note that Axum has great examples for a log of practical scenarios,
    // including graceful shutdown (https://github.com/tokio-rs/axum/tree/main/examples)
    let addr = SocketAddr::from(([0, 0, 0, 0], web_port));

    tracing::info!("Listening on {}", addr);
    axum_server::bind(addr).serve(app.into_make_service()).await
}
