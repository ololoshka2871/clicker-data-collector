use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_template::{Key, RenderHtml};
use serde::Serialize;
use tokio::sync::Mutex;
