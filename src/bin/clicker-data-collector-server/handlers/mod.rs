pub mod config;
pub mod handle_control;
pub mod handle_work;
pub mod into_body;
pub mod static_files;

pub(crate) use config::{handle_config, /*handle_config_and_save,*/ handle_update_config};
pub(crate) use handle_control::{
    handle_measurements_delete, handle_measurements_get, handle_measurements_append,
    handle_measurements_put, handle_measurements_insert, handle_measurements_cancel
};
pub(crate) use handle_work::{
    handle_generate_report_excel, handle_get_globals, handle_set_globals, handle_get_work,
    handler_reset_globals
};
