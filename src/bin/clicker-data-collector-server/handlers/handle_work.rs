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

pub(crate) async fn handle_work(State(engine): State<AppEngine>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct WorkModel {}

    let model = WorkModel {};

    RenderHtml(Key("work".to_owned()), engine, model)
}

pub(crate) async fn handle_get_flobals(
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
    /*
    use super::into_body::IntoBody;

    const ROW_OFFSET: usize = 12;
    let report_template_xlsx = include_bytes!("report.xlsx");

    if let Ok(mut book) =
        umya_spreadsheet::reader::xlsx::read_reader(Cursor::new(report_template_xlsx), true)
    {
        let sheet = book.get_sheet_by_name_mut("report").unwrap();

        {
            // date
            use chrono::{DateTime, Local};

            let system_time = SystemTime::now();
            let datetime: DateTime<Local> = system_time.into();
            let date = datetime.format("%d.%m.%Y %T").to_string();
            let time = datetime.format("%T").to_string();
            sheet.get_cell_value_mut("F2").set_value(date);
            sheet.get_cell_value_mut("G2").set_value(time);
        }

        // Обозначение партии
        sheet.get_cell_value_mut("C4").set_value(part_id.clone());

        // Диопазон
        let (freq_target, work_offset_hz, working_offset_ppm) = {
            let guard = freqmeter_config.lock().await;
            (
                guard.target_freq,
                guard.work_offset_hz,
                guard.working_offset_ppm,
            )
        };

        sheet
            .get_cell_value_mut("C7")
            .set_value(format2digits(freq_target));

        // ppm
        sheet
            .get_cell_value_mut("E7")
            .set_value(format2digits(working_offset_ppm));

        // min-max
        let limits = Limits::from_config(freq_target, &config, working_offset_ppm);
        sheet
            .get_cell_value_mut("G7")
            .set_value(format2digits(limits.lower_limit));
        sheet
            .get_cell_value_mut("H7")
            .set_value(format2digits(limits.upper_limit));

        // поправка частотомера
        sheet
            .get_cell_value_mut("C8")
            .set_value(format2digits(work_offset_hz));

        // таблица
        let report = auto_adjust_all_ctrl.lock().await.get_status();
        if !report.rezonator_info.is_empty() {
            for (i, r) in report.rezonator_info.iter().enumerate() {
                let row = ROW_OFFSET + i; // row in table

                let current_freq = r.current_freq;
                sheet
                    .get_cell_value_mut(format!("C{row}"))
                    .set_value(format2digits(current_freq));

                let start = format2digits(r.initial_freq);
                sheet.get_cell_value_mut(format!("B{row}")).set_value(start);

                let ppm = format2digits(limits.ppm(current_freq));
                sheet.get_cell_value_mut(format!("D{row}")).set_value(ppm);

                let ok = limits.to_status_icon(current_freq).to_owned();
                sheet.get_cell_value_mut(format!("E{row}")).set_value(ok);
            }
        } else {
            // clear table
            for row in ROW_OFFSET..ROW_OFFSET + config.resonator_placement.len() {
                for col in ['B', 'C', 'D', 'E'] {
                    sheet
                        .get_cell_value_mut(format!("{col}{row}"))
                        .set_value("-");
                }
            }
        }

        let mut buf = vec![];
        match umya_spreadsheet::writer::xlsx::write_writer(&book, Cursor::new(&mut buf)) {
            Ok(_) => {
                let filename = format!("attachment; filename=\"{}\".xlsx", part_id);
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
    */
    (StatusCode::INTERNAL_SERVER_ERROR, "Not implemented").into_response()
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
