use std::path::PathBuf;

use crate::environment;
use gpui::{App, Global};
use serde::{Deserialize, Serialize};

pub mod font;
pub mod theme;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    pub font_family: String,
    pub font_size: f32,
    pub theme_name: String,
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

    pub fn save(&self) -> Result<(), Error> {
        let config_bytes = toml::to_string_pretty(self)
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

    /// Applies the config to the App state.
    pub fn apply_to_state(&self, cx: &mut App) {
        theme::set_theme_name(self.theme_name.clone().into(), cx);
        font::set_font_size(self.font_size, cx);
        font::set_font_family(self.font_family.clone().into(), cx);
    }

    pub fn create_initial_config() {
        let config_file_path = Self::path();
        if config_file_path.exists() {
            return;
        }

        let config_bytes = toml::to_string_pretty(&Config::default())
            .expect("expected valid default config serialization")
            .into_bytes();

        let _ = std::fs::write(config_file_path, config_bytes);
    }
}

impl Global for Config {}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_family: "Lilex".to_owned(),
            font_size: 16.0,
            theme_name: "Gruvbox Dark".to_owned(),
        }
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
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
