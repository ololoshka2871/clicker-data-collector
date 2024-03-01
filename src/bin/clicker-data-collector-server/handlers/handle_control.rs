use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use clicker_data_collector::data_model::DataModel;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// Получить список всех резонаторов
pub(crate) async fn handle_measurements_get(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Model {
        records: Vec<ResonatorData>,
        total: usize,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize)]
    struct ResonatorData {
        id: u32,
        timestamp: String,
        F: f32,
        Rk: f32,
        Comment: String,
    }

    impl From<&clicker_data_collector::data_model::ResonatorData> for ResonatorData {
        fn from(data: &clicker_data_collector::data_model::ResonatorData) -> Self {
            Self {
                id: data.id,
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
            .map(ResonatorData::from)
            .collect::<Vec<_>>(),
        total,
    })
}

// Начать процедуру измерения нового резонатора
pub(crate) async fn handle_measurements_post(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    (StatusCode::OK, "Done")
}

// Перезапустить измерение существующего резонатора id
pub(crate) async fn handle_measurements_put(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    (StatusCode::OK, "Done")
}

// Удалить резонатор id
pub(crate) async fn handle_measurements_delete(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    (StatusCode::OK, "Done")
}
