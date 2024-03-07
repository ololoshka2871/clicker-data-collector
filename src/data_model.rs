use std::vec;

use chrono::{DateTime, Local};

use crate::MeasureProcessStat;

#[derive(Clone)]
pub struct ResonatorData {
    ///! Время снятия данных
    pub timestamp: DateTime<Local>,
    ///! Частота
    pub frequency: f32,
    ///! Отклонение частоты
    pub frequency_deviation: f32,
    ///! Динамическое сопротивление
    pub rk: f32,
    ///! Отклонение динамического сопротивления
    pub rk_deviation: f32,
    ///! Коментарий
    pub comment: String,
}

pub struct DataModel {
    ///! Тип партии резонаторов
    pub data_type: String,
    ///! Маршрутный лист
    pub route_id: String,
    ///! Температура окружающей среды
    pub ambient_temperature_range: Vec<f32>,
    ///! Коментарий к партии
    pub comment: String,
    ///! Время снятия данных
    pub timestamp: DateTime<Local>,
    ///! Данные по резонаторам
    pub resonators: Vec<ResonatorData>,
}

impl Default for DataModel {
    fn default() -> Self {
        Self {
            data_type: String::new(),
            route_id: String::new(),
            ambient_temperature_range: vec![20.0],
            comment: String::new(),
            timestamp: Local::now(),
            resonators: Vec::new(),
        }
    }
}

impl From<MeasureProcessStat> for ResonatorData {
    fn from(stat: MeasureProcessStat) -> Self {
        let freqs_avg = stat.freqs_avg.unwrap();
        let rks_avg = stat.rks_avg.unwrap();
        let timestamp: DateTime<Local> = stat.timestamp.into();
        Self {
            timestamp,
            frequency: freqs_avg.median(),
            frequency_deviation: freqs_avg.iqr(),
            rk: rks_avg.median(),
            rk_deviation: rks_avg.iqr(),
            comment: String::new(),
        }
    }
}
