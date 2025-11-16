use super::color_schema::Theme;
use clap::Parser;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub symbol: String,
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

    pub sl_pnl: Option<Decimal>,
}

#[derive(Parser, Debug)]
#[command(about = "Scalper")]
struct Cli {
    #[arg(index = 1)]
    symbol: String,
    #[arg(long)]
    theme: Option<String>,
    #[arg(long)]
    sl_pnl: Option<Decimal>,
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

        let mut config: Config = toml::from_str(&contents)?;

        let cli_overrides = Cli::parse();
        config.symbol = cli_overrides.symbol.clone();
        if let Some(sl_pnl) = cli_overrides.sl_pnl {
            config.sl_pnl = Some(sl_pnl);
        }
        if let Some(theme) = cli_overrides.theme {
            config.theme = match theme.to_lowercase().as_str() {
                "light" => Theme::Light,
                "auto" => Theme::Auto,
                _ => Theme::Dark,
            };
        }

        Ok(config)
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("No home directory.")?;
        Ok(home.join(".scalper").join("config"))
    }
}
