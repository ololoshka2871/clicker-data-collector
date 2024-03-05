use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use clicker_data_collector::data_model::{DataModel, ResonatorData};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

use crate::generate_fake_res_data;

// Получить список всех резонаторов
pub(crate) async fn handle_measurements_get(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Model {
        records: Vec<ResData>,
        total: usize,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize)]
    struct ResData {
        id: u32,
        timestamp: String,
        F: f32,
        Rk: f32,
        Comment: String,
    }

    impl From<&ResonatorData> for ResData {
        fn from(data: &ResonatorData) -> Self {
            Self {
                id: 0,
                timestamp: data.timestamp.to_rfc3339(),
                F: data.frequency,
                Rk: data.rk,
                Comment: data.comment.clone(),
            }
        }
    }

    let data_model_guard = data_model.lock().await;
    let total = data_model_guard.resonators.len();

    Json(Model {
        records: data_model_guard
            .resonators
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let mut res = ResData::from(r);
                res.id = i as u32 + 1;
                res
            })
            .collect::<Vec<_>>(),
        total,
    })
}

// Начать процедуру измерения нового резонатора
pub(crate) async fn handle_measurements_append(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_add");

    let mut guard = data_model.lock().await;
    let new_id = guard.resonators.len() as u32 + 1;

    guard.resonators.push(generate_fake_res_data());

    (StatusCode::OK, new_id.to_string()).into_response()
}

pub(crate) async fn handle_measurements_insert(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    Path(id): Path<u32>,
    body: String,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_insert: id={}, insert={}", id, body);

    let mut guard = data_model.lock().await;
    let new_res = generate_fake_res_data();
    let id = id.saturating_sub(1);

    let new_id = if body.to_uppercase() == "TRUE" {
        guard.resonators.insert(id as usize, new_res);
        id + 1
    } else {
        match guard.resonators.get_mut(id as usize) {
            None => return StatusCode::NOT_FOUND.into_response(),
            Some(r) => *r = new_res,
        };
        id
    };

    (StatusCode::OK, new_id.to_string()).into_response()
}

// Перезапустить измерение существующего резонатора id
pub(crate) async fn handle_measurements_put(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    Path(id): Path<u32>,
    body: String,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_put: id={}, body={}", id, body);

    match data_model
        .lock()
        .await
        .resonators
        .get_mut(id as usize)
        .map(|r| r.comment = body)
    {
        None => StatusCode::NOT_FOUND,
        Some(_) => StatusCode::OK,
    }
}

// Удалить резонатор id
pub(crate) async fn handle_measurements_delete(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_delete: id={}", id);

    let id = (id as usize).saturating_sub(1);

    let mut guard = data_model.lock().await;

    if guard.resonators.len() <= id as usize {
        StatusCode::NOT_FOUND
    } else {
        guard.resonators.remove(id as usize);
        StatusCode::OK
    }
}
