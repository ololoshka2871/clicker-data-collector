use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use clicker_data_collector::{
    data_model::{DataModel, ResonatorData},
    ClickerController,
};
use serde::{Deserialize, Serialize};
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

async fn measure_common<F: Fn(&mut DataModel) + Send + 'static>(
    data_model: Arc<Mutex<DataModel>>,
    clicker_ctrl: Arc<Mutex<ClickerController>>,
    after_measure: F,
) -> axum::response::Response {
    use clicker_data_collector::{MeasureProcessStat, MeasureProcessState};

    #[derive(Serialize)]
    struct MeasureStatus {
        result: String,
        data: MeasureProcessStat,
    }

    let rx = {
        let mut guard = clicker_ctrl.lock().await;
        match guard.start_mesure() {
            Ok(_) => guard.subscribe_measure_status(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    };

    match rx {
        Some(mut rx) => {
            let stream = async_stream::stream! {
                loop {
                    match rx.changed().await {
                        Ok(_) => {
                            let res = (*rx.borrow()).clone();
                            match res.state {
                                MeasureProcessState::Idle => {
                                    continue;
                                }
                                MeasureProcessState::Running => {
                                    yield res;
                                }
                                MeasureProcessState::Interrupted | MeasureProcessState::Finished => {
                                    yield res;
                                    break;
                                }
                            }

                        }
                        Err(_) => return,
                    }
                }

                let mut guard = data_model.lock().await;
                after_measure(&mut guard);
            };
            axum_streams::StreamBodyAs::json_nl(stream).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось запустить измерительный процесс!",
        )
            .into_response(),
    }
}

// Начать процедуру измерения нового резонатора
pub(crate) async fn handle_measurements_append(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    State(clicker_ctrl): State<Arc<Mutex<ClickerController>>>,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_add");

    let after_measure = |data_model: &mut DataModel| {
        data_model.resonators.push(generate_fake_res_data());
    };

    measure_common(data_model, clicker_ctrl, after_measure).await
}

pub(crate) async fn handle_measurements_insert(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    State(clicker_ctrl): State<Arc<Mutex<ClickerController>>>,
    Path(id): Path<u32>,
    body: String,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_insert: id={}, insert={}", id, body);

    let insert = body.to_uppercase() == "TRUE";
    let id = {
        let id = id.saturating_sub(1);
        if id as usize > data_model.lock().await.resonators.len() {
            return StatusCode::NOT_FOUND.into_response();
        }
        id
    };

    let after_measure = move |data_model: &mut DataModel| {
        let new_res = generate_fake_res_data();

        if insert {
            data_model.resonators.insert(id as usize, new_res);
        } else {
            data_model.resonators[id as usize] = new_res;
        }
    };

    measure_common(data_model, clicker_ctrl, after_measure).await
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

// Удалить резонатор id
pub(crate) async fn handle_measurements_cancel(
    State(clicker_ctrl): State<Arc<Mutex<ClickerController>>>,
) -> impl IntoResponse {
    tracing::debug!("handle_measurements_cancel");

    clicker_ctrl.lock().await.interrupt_mesure().await;

    StatusCode::OK
}