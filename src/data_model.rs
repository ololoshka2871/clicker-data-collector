use std::vec;

use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct ResonatorData {
    ///! Время снятия данных
    pub timestamp: DateTime<Local>,
    ///! Частота
    pub frequency: f32,
    ///! Динамическое сопротивление
    pub rk: f32,
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
