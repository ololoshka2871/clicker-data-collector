use std::{borrow::BorrowMut, io::Cursor, sync::Arc, time::SystemTime};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_template::{Key, RenderHtml};

use chrono::{DateTime, Local};
use clicker_data_collector::data_model::DataModel;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::AppEngine;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ReportHeader {
    pub data_type: String,
    pub route_id: String,
    pub ambient_temperature_range: String,
    pub comment: String,
    pub date: String,
}

pub(crate) async fn handle_get_work(State(engine): State<AppEngine>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct WorkModel {}

    let model = WorkModel {};

    RenderHtml(Key("work".to_owned()), engine, model)
}

pub(crate) async fn handle_get_globals(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    let data_model = data_model.lock().await;

    let ambient_temperature_range = data_model
        .ambient_temperature_range
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let report_header = ReportHeader {
        data_type: data_model.data_type.clone(),
        route_id: data_model.route_id.clone(),
        ambient_temperature_range,
        comment: data_model.comment.clone(),
        date: data_model.timestamp.format("%Y-%m-%d").to_string(),
    };

    Json(report_header)
}

pub(crate) async fn handle_set_globals(
    State(data_model): State<Arc<Mutex<DataModel>>>,
    Json(payload): Json<ReportHeader>,
) -> impl IntoResponse {
    if let Err(e) = try_parce_config(payload, data_model.lock().await.borrow_mut()) {
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    return StatusCode::OK.into_response();
}

// Генерация Excell отчета
pub(crate) async fn handle_generate_report_excel(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    use super::into_body::IntoBody;

    const SHEET_NAME: &str = "report";
    let report_template_xlsx = include_bytes!("report.xlsx");

    if let Ok(mut book) =
        umya_spreadsheet::reader::xlsx::read_reader(Cursor::new(report_template_xlsx), true)
    {
        // Fill header
        let filename = {
            let sheet = book.get_sheet_by_name_mut(SHEET_NAME).unwrap();
            let guard = data_model.lock().await;

            // Испытания
            sheet
                .get_cell_value_mut("D1")
                .set_value(guard.data_type.clone());

            // Маршрутный лист №
            sheet
                .get_cell_value_mut("D2")
                .set_value(guard.route_id.clone());

            // Температура
            sheet.get_cell_value_mut("D3").set_value(
                guard
                    .ambient_temperature_range
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            );

            // Комментарий
            sheet
                .get_cell_value_mut("B5")
                .set_value(guard.comment.clone());

            // Дата
            sheet
                .get_cell_value_mut("C7")
                .set_value(guard.timestamp.format("%d.%m.%Y").to_string());

            format!(
                "{data_type}@{date}",
                data_type = &guard.data_type,
                date = guard.timestamp.format("%Y-%m-%d").to_string()
            )
        };

        // Fill table
        {
            use umya_spreadsheet::{helper::coordinate::CellCoordinates, Border, Color, Worksheet};

            fn set_borders<T: Into<CellCoordinates>>(sheet: &mut Worksheet, coordinate: T) {
                let borders = sheet.get_style_mut(coordinate).get_borders_mut();
                let mut style = Border::default();
                style.get_color_mut().set_argb(Color::COLOR_BLACK);
                style.set_border_style(Border::BORDER_THIN);

                borders.set_bottom(style.clone());
                borders.set_top(style.clone());
                borders.set_left(style.clone());
                borders.set_right(style);
            }

            let resonators = data_model.lock().await.resonators.clone();

            if !resonators.is_empty() {
                book.insert_new_row(SHEET_NAME, &9, &(resonators.len() as u32));

                let sheet = book.get_sheet_by_name_mut(SHEET_NAME).unwrap();

                resonators.iter().enumerate().for_each(|(i, row)| {
                    let row_num = 9 + i as u32;

                    sheet
                        .get_cell_value_mut((2, row_num))
                        .set_value_number(i as u32 + 1);
                    set_borders(sheet, (2, row_num));
                    sheet
                        .get_cell_value_mut((3, row_num))
                        .set_value_number(row.frequency);
                    set_borders(sheet, (3, row_num));
                    sheet
                        .get_cell_value_mut((4, row_num))
                        .set_value_number(row.rk);
                    set_borders(sheet, (4, row_num));
                    sheet
                        .get_cell_value_mut((5, row_num))
                        .set_value_string(&row.comment);
                    set_borders(sheet, (5, row_num));
                });
            }
        }

        let mut buf = vec![];
        match umya_spreadsheet::writer::xlsx::write_writer(&book, Cursor::new(&mut buf)) {
            Ok(_) => {
                let filename = format!("attachment; filename=\"{}\".xlsx", filename);
                let headers = [
                    (
                        axum::http::header::CONTENT_TYPE,
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
                    ),
                    (axum::http::header::CONTENT_DISPOSITION, filename.as_str()),
                ];
                (headers, buf.into_body()).into_response()
            }
            Err(e) => {
                let err = format!("Failed to generate report: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
            }
        }
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load report template",
        )
            .into_response()
    }
}

pub(crate) async fn handler_reset_globals(
    State(data_model): State<Arc<Mutex<DataModel>>>,
) -> impl IntoResponse {
    let mut data_model = data_model.lock().await;
    *data_model = DataModel::default();

    StatusCode::OK
}

fn try_parce_config(payload: ReportHeader, data_model: &mut DataModel) -> Result<(), String> {
    data_model.data_type = payload.data_type;
    data_model.route_id = payload.route_id;

    data_model.ambient_temperature_range.clear();
    for r in payload
        .ambient_temperature_range
        .split(',')
        .map(|s| s.trim().parse::<f32>())
    {
        if r.is_err() {
            return Err(format!("Неверный набор температур '{}'.\nпроверьте что строка содержит числа, разделенные запятой", payload.ambient_temperature_range));
        } else {
            data_model.ambient_temperature_range.push(r.unwrap());
        }
    }

    data_model.comment = payload.comment;

    data_model.timestamp = dateparser::parse_with_timezone(&payload.date, &Local)
        .map_err(|_| format!("Значение '{}' не содержит дату", payload.date))?
        .into();

    Ok(())
}
