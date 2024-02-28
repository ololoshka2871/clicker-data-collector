pub mod common;
pub mod config;
pub mod handle_control;
pub mod handle_stat;
pub mod into_body;
pub mod static_files;
pub mod handle_work;

pub(crate) use config::{handle_config, /*handle_config_and_save,*/ handle_update_config};
pub(crate) use handle_control::handle_control;
pub(crate) use handle_work::handle_work;
/*
pub(crate) use handle_stat::{
    handle_stat_auto, handle_stat_manual, handle_stat_rez_auto, handle_stat_rez_manual,
};
*/