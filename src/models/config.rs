use super::color_schema::Theme;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_width")]
    pub window_width: usize,
    #[serde(default = "default_height")]
    pub window_height: usize,

    pub binance_access_key: Option<String>,
    pub binance_secret_key: Option<String>,

    #[serde(default = "default_size")]
    pub size_1: Option<Decimal>,
    #[serde(default = "default_size")]
    pub size_2: Option<Decimal>,
    #[serde(default = "default_size")]
    pub size_3: Option<Decimal>,
}

fn default_size() -> Option<Decimal> {
    Some(Decimal::from(100))
}

fn default_width() -> usize {
    800
}

fn default_height() -> usize {
    600
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        let contents = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(err) => {
                return Err(format!(
                    "Failed to read config file at {}: {}",
                    config_path.display(),
                    err
                )
                .into());
            }
        };

        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("No home directory.")?;
        Ok(home.join(".scalper").join("config"))
    }
}
