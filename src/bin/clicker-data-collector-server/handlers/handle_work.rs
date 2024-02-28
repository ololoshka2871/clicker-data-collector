use axum::{extract::State, response::IntoResponse};
use axum_template::{Key, RenderHtml};

use serde::Serialize;

use crate::AppEngine;

pub(crate) async fn handle_work(State(engine): State<AppEngine>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct WorkModel {}

    let model = WorkModel {};

    RenderHtml(Key("work".to_owned()), engine, model)
}
