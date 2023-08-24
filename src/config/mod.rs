pub mod entity;
pub mod visuals;
pub mod glyphs;
pub mod messages;
pub mod char_create;
mod load;

use rltk::prelude::*;
use toml::de::Error as TomlError;
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

#[derive(Debug)]
pub enum ReadError {
    Io(std::io::Error),
    Toml(TomlError),
}

impl From<std::io::Error> for ReadError {
    fn from(error: std::io::Error) -> Self {
        ReadError::Io(error)
    }
}

impl From<TomlError> for ReadError {
    fn from(error: TomlError) -> Self {
        ReadError::Toml(error)
    }
}

impl Config {
    pub fn load_from_file(filename: &str) -> Result<Self, ReadError> {
        let contents = std::fs::read_to_string(filename).map_err(|e| ReadError::Io(e))?;
        let config: Config = toml::from_str(&contents).map_err(|e| ReadError::Toml(e))?;
        return Ok(config);
    }
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string(self)?;
        std::fs::write(filename, toml_string)?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn try_load_configuration() -> Config {
    let config: Config = match Config::load_from_file("config.toml") {
        Ok(config) => {
            console::log(format!("Successfully loaded config: {:?}", config));
            config
        }
        Err(e) => {
            console::log(format!("Error loading config: {:?}", e));
            let config = Config::default();
            if let Err(write_err) = config.save_to_file("config.toml") {
                eprintln!("Error writing default config: {:?}", write_err);
            }
            config
        }
    };

    return config;
}

#[cfg(target_arch = "wasm32")]
pub fn try_load_configuration() -> Config {
    let config = Config::default();
}
