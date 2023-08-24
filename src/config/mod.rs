pub mod entity;
pub mod visuals;
pub mod glyphs;
pub mod messages;
pub mod char_create;
mod load;

use rltk::prelude::*;
use toml::Value;
use serde::{ Serialize, Deserialize };

lazy_static! {
    pub static ref CONFIG: Config = try_load_configuration();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub logging: LogConfig,
    pub visuals: VisualConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualConfig {
    pub with_scanlines: bool,
    pub with_screen_burn: bool,
    pub with_darken_by_distance: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub show_mapgen: bool,
    pub log_spawning: bool,
    pub log_ticks: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logging: LogConfig {
                show_mapgen: false,
                log_spawning: false,
                log_ticks: false,
            },
            visuals: VisualConfig {
                with_scanlines: false,
                with_screen_burn: false,
                with_darken_by_distance: true,
            },
        }
    }
}

impl Config {
    pub fn load_from_file(filename: &str) -> Config {
        if let Ok(contents) = std::fs::read_to_string(filename) {
            let parsed_config: Result<Value, _> = toml::from_str(&contents);
            if let Ok(parsed_config) = parsed_config {
                let mut config = Config::default();
                let mut requires_write = false;
                requires_write |= config.logging.apply_values(&parsed_config);
                requires_write |= config.visuals.apply_values(&parsed_config);
                if requires_write {
                    if let Err(write_err) = config.save_to_file(filename) {
                        console::log(format!("Error writing config: {:?}", write_err));
                    }
                }

                return config;
            }
        }
        Config::default()
    }
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string(self)?;
        std::fs::write(filename, toml_string)?;
        Ok(())
    }
}

macro_rules! apply_bool_value {
    ($config:expr, $parsed_config:expr, $changed:expr, $field:ident) => {
        if let Some(value) = $parsed_config.get(stringify!($field)).and_then(|v| v.as_bool()) {
            if $config.$field != value {
                $config.$field = value;
                $changed = true;
            }
        }
    };
}

trait Section {
    fn apply_values(&mut self, parsed_config: &Value) -> bool;
}

impl Section for LogConfig {
    fn apply_values(&mut self, parsed_config: &Value) -> bool {
        if let Some(section) = parsed_config.get("logging") {
            let mut missing = false;
            apply_bool_value!(self, section, missing, log_spawning);
            apply_bool_value!(self, section, missing, log_ticks);
            missing
        } else {
            true
        }
    }
}

impl Section for VisualConfig {
    fn apply_values(&mut self, parsed_config: &Value) -> bool {
        if let Some(section) = parsed_config.get("visuals") {
            let mut missing = false;
            apply_bool_value!(self, section, missing, with_scanlines);
            apply_bool_value!(self, section, missing, with_screen_burn);
            apply_bool_value!(self, section, missing, with_darken_by_distance);
            missing
        } else {
            true
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn try_load_configuration() -> Config {
    let config: Config = Config::load_from_file("config.toml");
    return config;
}

#[cfg(target_arch = "wasm32")]
pub fn try_load_configuration() -> Config {
    let config = Config::default();
}
