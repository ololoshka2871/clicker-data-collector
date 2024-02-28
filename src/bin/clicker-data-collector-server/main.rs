#![feature(async_iterator)]

mod handlers;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::FromRef,
    response::Redirect,
    routing::{get, patch, post},
    Router,
};

use clicker_data_collector::data_point::DataPoint;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

use axum_template::engine::Engine;

use minijinja::Environment;

use handlers::*;

pub(crate) type AppEngine = Engine<Environment<'static>>;

#[derive(Clone)]
struct ChannelState {
    current_step: u32,
    initial_freq: Option<f32>,

    points: Vec<DataPoint<f64>>,
}

#[derive(Clone, FromRef)]
struct AppState {
    engine: AppEngine,
    config: clicker_data_collector::Config,
    config_file: std::path::PathBuf,
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
                .unwrap_or_else(|_| "laser_precision_adjust_server=debug,tower_http=info".into()),
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

    tracing::warn!("Testing connection...");
    // todo

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

    let app_state = AppState {
        engine: Engine::from(minijinja),
        config,
        config_file,
    };

    // Build our application with some routes
    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/work") }))
        .route("/work", get(handle_work))
        .route("/control/:action", post(handle_control))
        //.route("/report/:part_id", get(handle_generate_report))
        .route("/config", get(handle_config).patch(handle_update_config))
        //.route("/config-and-save", patch(handle_config_and_save))
        .route("/static/:path/:file", get(static_files::handle_static))
        .route("/lib/*path", get(static_files::handle_lib))
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
