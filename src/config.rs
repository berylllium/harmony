use std::path::PathBuf;

use crate::environment;
use gpui::{App, SharedString};
use serde::{Deserialize, Serialize};

pub mod font;
pub mod theme;

pub const DEFAULT_THEME_NAME: &str = "Gruvbox Dark";
pub const DEFAULT_FONT_SIZE: f32 = 16.0;
pub const DARK_MODE: bool = true;
pub const FONT_FAMILY: &str = "Lilex";

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub theme_name: Option<String>,
    pub dark_mode: Option<bool>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let dir = environment::config_dir();

        if !dir.exists() {
            std::fs::create_dir_all(dir.as_path())
                .expect("expected permissions to create config folder");
        }

        dir
    }

    pub fn path() -> PathBuf {
        Self::config_dir().join(environment::CONFIG_FILE_NAME)
    }

    pub fn save(config: &Config) -> Result<(), Error> {
        let config_bytes = toml::to_string_pretty(config)
            .expect("expected valid config serialization")
            .into_bytes();

        std::fs::write(Self::path(), config_bytes)?;

        Ok(())
    }

    pub fn load_config() -> Result<Self, Error> {
        let path = Self::path();
        if !path.try_exists()? {
            Self::create_initial_config();
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| Error::LoadConfigFile(e.to_string()))?;

        let config =
            toml::Deserializer::parse(content.as_ref()).map_err(|e| Error::Parse(e.to_string()))?;

        let config = serde_ignored::deserialize(config, |ignored| {
            tracing::warn!("[config.toml] Ignoring unknown setting: {ignored}");
        })
        .map_err(|e| Error::Parse(e.to_string()))?;

        Ok(config)
    }

    pub fn load_settings_into_state(cx: &mut App) -> Result<(), Error> {
        let config = Self::load_config()?;

        if let Some(theme_name) = config.theme_name {
            Self::change_theme(theme_name, cx);
        }
        if let Some(font_family) = config.font_family {
            Self::change_font_family(font_family.into(), cx);
        }
        if let Some(font_size) = config.font_size {
            Self::change_font_size(font_size, cx);
        }
        if let Some(dark_mode) = config.dark_mode {
            Self::change_dark_mode(dark_mode, cx);
        }

        Ok(())
    }

    pub fn create_initial_config() {
        let config_file_path = Self::path();
        if config_file_path.exists() {
            return;
        }

        let default = Self {
            dark_mode: Some(DARK_MODE),
            font_family: Some(FONT_FAMILY.to_string()),
            font_size: Some(DEFAULT_FONT_SIZE),
            theme_name: Some(DEFAULT_THEME_NAME.to_string()),
        };

        let config_bytes = toml::to_string_pretty(&default)
            .expect("expected valid default config serialization")
            .into_bytes();

        let _ = std::fs::write(config_file_path, config_bytes);
    }

    pub fn change_font_size(size: f32, cx: &mut App) {
        font::set_font_size(size, cx);
        let mut config = Self::load_config().unwrap_or_default();
        config.font_size = Some(size);
        let _ = Self::save(&config);
    }

    pub fn change_font_family(font: String, cx: &mut App) {
        font::set_font_family(font.clone().into(), cx);
        let mut config = Self::load_config().unwrap_or_default();
        config.font_family = Some(font);
        let _ = Self::save(&config);
    }

    pub fn change_theme(theme_name: String, cx: &mut App) {
        let font_size: f32 = font::font_size(cx) as f32;
        let font_family: SharedString = font::font_family(cx);
        theme::set_theme_name(theme_name.clone().into(), cx);
        let mut config = Self::load_config().unwrap_or_default();
        config.theme_name = Some(theme_name);
        let _ = Self::save(&config);
        // Theme overrides font, restore.
        font::set_font_size(font_size, cx);
        font::set_font_family(font_family, cx);
    }

    pub fn change_dark_mode(is_dark: bool, cx: &mut App) {
        let font_size: f32 = font::font_size(cx) as f32;
        let font_family: SharedString = font::font_family(cx);
        theme::set_dark_mode(is_dark, cx);
        let mut config = Self::load_config().unwrap_or_default();
        config.dark_mode = Some(is_dark);
        let _ = Self::save(&config);
        // Theme overrides font, restore.
        font::set_font_size(font_size, cx);
        font::set_font_family(font_family, cx);
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("config could not be read: {0}")]
    LoadConfigFile(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Parse(String),
    #[error("config does not exist")]
    ConfigMissing,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
