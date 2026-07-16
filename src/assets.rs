use std::borrow::Cow;

use anyhow::Context;
use gpui::{App, AssetSource, Result, SharedString};
use gpui_component::IconNamed;
use rust_embed::Embed;

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
    pub fn load_fonts(cx: &App) -> anyhow::Result<()> {
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
