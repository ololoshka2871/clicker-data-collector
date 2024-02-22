use std::{cmp::min, collections::HashSet, sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use clicker_data_collector::{box_plot::BoxPlot, Config};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct ControlRequest {
    /*
    #[serde(rename = "Channel", skip_serializing_if = "Option::is_none")]
    channel: Option<u32>,

    #[serde(rename = "CameraAction", skip_serializing_if = "Option::is_none")]
    camera_action: Option<String>,

    #[serde(rename = "TargetPosition", skip_serializing_if = "Option::is_none")]
    target_position: Option<i32>,

    #[serde(rename = "MoveOffset", skip_serializing_if = "Option::is_none")]
    move_offset: Option<i32>,
    */
}

#[derive(Serialize, Debug, Default)]
pub struct ControlResult {
    success: bool,
    error: Option<String>,
    message: Option<String>,
}

impl ControlResult {
    pub fn new(success: bool, error: Option<String>, message: Option<String>) -> Self {
        Self {
            success,
            error,
            message,
        }
    }

    pub fn success(message: Option<String>) -> Self {
        Self::new(true, None, message)
    }

    pub fn error(err_message: String) -> Self {
        Self::new(false, Some(err_message), None)
    }
}

// Сюда будут поступать команды от веб-интерфейса
pub(crate) async fn handle_control(
    Path(path): Path<String>,
    Json(payload): Json<ControlRequest>,
) -> impl IntoResponse {
    (StatusCode::OK, "Done")
}
