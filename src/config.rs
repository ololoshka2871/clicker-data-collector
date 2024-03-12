use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn default_serial_port() -> String {
    "COM1".to_string()
}

fn default_web_port() -> u16 {
    3289
}

fn default_cycles() -> u32 {
    3
}

#[derive(Deserialize, Clone, Serialize)]
pub struct Config {
    #[serde(rename = "RkMeterPort", default = "default_serial_port")]
    pub rk_meter_port: String,

    #[serde(rename = "WebPort", default = "default_web_port")]
    pub web_port: u16,

    #[serde(rename = "Cycles", default = "default_cycles")]
    pub cycles: u32,
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
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(path.clone()) {
                return (serde_json::from_str::<Config>(&contents).unwrap(), path);
            }
        } else {
            path.parent().map(|p| std::fs::create_dir_all(p));
        }

        tracing::warn!("Failed to read {:?} file! Load default config!", path);

        let config: Self = serde_json::from_str("{}").unwrap();
        config.save();

        (config, path)
    }

    pub fn save(&self) {
        tracing::debug!("Save settings");

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
        writeln!(f, "WebPort: {}", self.web_port)?;
        writeln!(f, "Cycles: {}", self.cycles)?;

        Ok(())
    }
}
