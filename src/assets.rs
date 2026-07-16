use std::borrow::Cow;

use anyhow::Context;
use gpui::{App, AssetSource, Result, SharedString};
use gpui::{Global, ReadGlobal, UpdateGlobal};
use gpui_component::IconNamed;
use gpui_component::ThemeRegistry;
use rust_embed::Embed;

use crate::config::Config;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Self::get(path)
            .map(|f| Some(f.data))
            .with_context(|| format!("loading asset at path {path:?}"))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect())
    }
}

impl Assets {
    fn load_fonts(cx: &App) -> anyhow::Result<()> {
        let font_paths = cx.asset_source().list("fonts")?;
        let mut embedded_fonts = Vec::new();
        for font_path in font_paths {
            if font_path.ends_with(".otf") || font_path.ends_with(".ttf") {
                let font_bytes = cx
                    .asset_source()
                    .load(&font_path)?
                    .expect("Assets should never return None.");
                embedded_fonts.push(font_bytes);
            }
        }

        cx.text_system().add_fonts(embedded_fonts)
    }

    fn load_themes(cx: &mut App) {
        /// Debounce global to only load themes once after they've been loaded.
        /// Necessary because gpui-component doesn't expose the
        /// ThemeRegistry::reload_themes function.
        struct LoadThemeDebounce(bool);

        impl Global for LoadThemeDebounce {}

        cx.set_global(LoadThemeDebounce(false));

        if let Err(err) =
            ThemeRegistry::watch_dir(std::path::PathBuf::from("./themes"), cx, |cx: &mut App| {
                // Only apply the settings on the first call to this closure.
                if !LoadThemeDebounce::global(cx).0 {
                    Config::update_global(cx, |config, cx| {
                        config.apply_to_state(cx);
                    })
                }

                // Set the debounce to true so that future calls to this
                // closure will not apply the config again.
                LoadThemeDebounce::update_global(cx, |debounce, _| debounce.0 = true);
            })
        {
            tracing::error!("Failed to watch themes directory: {}", err);
        }
    }

    pub fn load_resources(cx: &mut App) -> anyhow::Result<()> {
        Self::load_fonts(cx)?;
        Self::load_themes(cx);
        Ok(())
    }
}

pub enum IconName {
    Frame,
    House,
}

impl IconNamed for IconName {
    fn path(self) -> SharedString {
        match self {
            IconName::Frame => "icons/frame.svg",
            IconName::House => "icons/house.svg",
        }
        .into()
    }
}
