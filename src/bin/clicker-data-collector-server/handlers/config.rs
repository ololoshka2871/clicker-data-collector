use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_template::{Key, RenderHtml};

use clicker_data_collector::Config;
use serde::{Deserialize, Serialize};

use tokio::sync::Mutex;

use crate::AppEngine;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateAndSaveConfigValues {
    #[serde(rename = "RkMeterPort", skip_serializing_if = "Option::is_none")]
    rk_meter_port: Option<String>,

    #[serde(rename = "WebPort", skip_serializing_if = "Option::is_none")]
    web_port: Option<u16>,

    #[serde(rename = "Cycles", skip_serializing_if = "Option::is_none")]
    cycles: Option<u32>,
}

pub(crate) async fn handle_config(
    State(engine): State<AppEngine>,
    State(config): State<Config>,
    State(config_file): State<std::path::PathBuf>,
) -> impl IntoResponse {
    #[derive(Serialize)]
    struct ConfigModel {
        pub config_file: String,
        pub config: Config,
    }

    let model: ConfigModel = ConfigModel {
        config_file: config_file.to_string_lossy().to_string(),
        config,
    };

    RenderHtml(Key("config".to_owned()), engine, model)
}

pub(crate) async fn handle_config_and_save(
    State(mut config): State<Config>,
    Json(input): Json<UpdateAndSaveConfigValues>,
) -> impl IntoResponse {
    tracing::debug!("handle_update_config_and_save: {:?}", input);

    let mut modified = false;

    if let Some(rk_meter_port) = input.rk_meter_port {
        if rk_meter_port.is_empty() {
            return (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "TargetFreq Должен быть больше 0",
            );
        }
        config.rk_meter_port = rk_meter_port;
        modified = true;
    }

    if let Some(web_port) = input.web_port {
        if web_port < 1024 {
            return (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "WebPort Должен быть больше 1024",
            );
        }
        config.web_port = web_port;
        modified = true;
    }

    if let Some(cycles) = input.cycles {
        if cycles < 1 {
            return (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "Cycles Должен быть больше 0",
            );
        }
        config.cycles = cycles;
        modified = true;
    }

    if modified {
        config.save();
    }

    (StatusCode::OK, "Done")
}
