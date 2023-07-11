use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ResonatroPlacement {
    #[serde(rename = "Xcenter")]
    pub x: f32,

    #[serde(rename = "Ycenter")]
    pub y: f32,

    #[serde(rename = "Width")]
    pub w: f32,

    #[serde(rename = "Height")]
    pub h: f32,
}

#[derive(Deserialize, Clone, Copy)]
pub struct AxisConfig {
    #[serde(rename = "SwapXY")]
    pub swap_xy: bool,

    #[serde(rename = "ReverseX")]
    pub reverse_x: bool,

    #[serde(rename = "ReverseY")]
    pub reverse_y: bool,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "LaserSetupPort")]
    pub laser_setup_port: String,

    #[serde(rename = "LaserControlPort")]
    pub laser_control_port: String,

    #[serde(rename = "KosaPort")]
    pub kosa_port: String,

    #[serde(rename = "FreqFifo")]
    pub freq_fifo: Option<PathBuf>,

    #[serde(rename = "PortTimeoutMs")]
    pub port_timeout_ms: u64,

    #[serde(rename = "GCodeTimeoutMs")]
    pub gcode_timeout_ms: u64,

    #[serde(rename = "AxisConfig")]
    pub axis_config: AxisConfig,

    #[serde(rename = "BurnLaserS")]
    pub burn_laser_pump_power: f32,

    #[serde(rename = "BurnLaserA")]
    pub burn_laser_power: f32,

    #[serde(rename = "BurnLaserB")]
    pub burn_laser_frequency: u32,

    #[serde(rename = "BurnLaserF")]
    pub burn_laser_feedrate: f32,

    #[serde(rename = "TotalVerticalSteps")]
    pub total_vertical_steps: u32,

    #[serde(rename = "ResonatorsPlacement")]
    pub resonator_placement: Vec<ResonatroPlacement>,
}

impl Config {
    pub fn load() -> Self {
        use std::path;

        if let Some(base_dirs) = directories::BaseDirs::new() {
            let path = base_dirs
                .config_dir()
                .join(path::Path::new("laser-precision-adjust"))
                .join(path::Path::new("config.json"));

            if let Ok(contents) = std::fs::read_to_string(path.clone()) {
                serde_json::from_str::<Config>(&contents).unwrap()
            } else {
                panic!(
                    "Failed to read {:?} file! Please copy config.json.example and fill it!",
                    path
                );
            }
        } else {
            panic!("Failed to get config directory!");
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "LaserSetupPort: {}", self.laser_setup_port)?;
        writeln!(f, "LaserControlPort: {}", self.laser_control_port)?;
        writeln!(f, "KosaPort: {}", self.kosa_port)?;
        writeln!(f, "PortTimeoutMs: {}", self.port_timeout_ms)?;
        writeln!(f, "GCodeTimeoutMs: {}", self.gcode_timeout_ms)?;

        writeln!(f, "AxisConfig:")?;
        writeln!(f, "  SwapXY: {}", self.axis_config.swap_xy)?;
        writeln!(f, "  ReverseX: {}", self.axis_config.reverse_x)?;
        writeln!(f, "  ReverseY: {}", self.axis_config.reverse_y)?;

        writeln!(f, "BurnLaserS: {}", self.burn_laser_pump_power)?;
        writeln!(f, "BurnLaserA: {}", self.burn_laser_power)?;
        writeln!(f, "BurnLaserB: {}", self.burn_laser_frequency)?;
        writeln!(f, "BurnLaserF: {}", self.burn_laser_feedrate)?;
        writeln!(f, "VerticalStep: {}", self.total_vertical_steps)?;

        // write resonators placement as a table
        writeln!(f, "ResonatorsPlacement:")?;
        writeln!(f, "  Center\t| Width\t| Height")?;
        writeln!(f, "  ------\t| -----\t| ------")?;
        for placement in &self.resonator_placement {
            writeln!(
                f,
                "  X{} Y{}\t| {}\t| {}",
                placement.x, placement.y, placement.w, placement.h
            )?;
        }
        Ok(())
    }
}
