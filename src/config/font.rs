use gpui::{App, SharedString, px};
use gpui_component::{ActiveTheme, PixelsExt, Theme, setting::NumberFieldOptions};

fn normalize_font_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

pub fn font_options(cx: &App) -> Vec<(SharedString, SharedString)> {
    let mut bundled_families: Vec<String> = cx
        .asset_source()
        .list("fonts")
        .unwrap_or_default()
        .iter()
        .filter_map(|path| {
            let file_name = path.rsplit('/').next()?;
            let stem = file_name.split('.').next()?;
            stem.split('-').next().map(|s| s.to_string())
        })
        .collect();
    bundled_families.sort();
    bundled_families.dedup();

    let mut options: Vec<(SharedString, SharedString)> =
        vec![(".SystemUIFont".into(), "System Default".into())];

    for name in cx.text_system().all_font_names() {
        if bundled_families
            .iter()
            .any(|family| normalize_font_name(family) == normalize_font_name(&name))
        {
            options.push((name.clone().into(), name.into()));
        }
    }

    options
}

pub fn font_size_options() -> NumberFieldOptions {
    NumberFieldOptions {
        min: 1.0,
        max: 32.0,
        step: 1.0,
        ..Default::default()
    }
}

pub fn font_size(cx: &App) -> f64 {
    cx.theme().font_size.as_f64()
}

pub fn set_font_size(size: f32, cx: &mut App) {
    if size < 8.0 || size > 72.0 {
        return;
    }
    Theme::global_mut(cx).font_size = px(size);
}

pub fn font_family(cx: &App) -> SharedString {
    cx.theme().font_family.clone()
}

pub fn set_font_family(font: SharedString, cx: &mut App) {
    Theme::global_mut(cx).font_family = font;
}
