use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Serialize)]
pub struct Config {
    #[serde(rename = "LaserSetupPort")]
    pub rk_meter_port: String,
}

impl Config {
    fn get_path() -> PathBuf {
        use std::path;

        if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs
                .config_dir()
                .join(path::Path::new("clicker-data-collector"))
                .join(path::Path::new("config.json"))
        } else {
            panic!("Failed to get config directory!");
        }
    }

    pub fn load() -> (Self, PathBuf) {
        let path = Self::get_path();
        if let Ok(contents) = std::fs::read_to_string(path.clone()) {
            (serde_json::from_str::<Config>(&contents).unwrap(), path)
        } else {
            panic!(
                "Failed to read {:?} file! Please copy config.json.example and fill it!",
                path
            );
        }
    }

    pub fn save(
        &mut self,
        rk_meter_port: String,
    ) {
        tracing::debug!("Save settings");

        self.rk_meter_port = rk_meter_port;

        let path = Self::get_path();

        match std::fs::File::create(path) {
            Ok(f) => serde_json::to_writer_pretty(f, self).expect("Failed to save settings"),
            Err(e) => tracing::error!("Faled to save settings: {e}"),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RkMeterPort: {}", self.rk_meter_port)?;
        
        Ok(())
    }
}
