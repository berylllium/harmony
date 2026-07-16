use std::rc::Rc;

use gpui::{App, SharedString};
use gpui_component::{ActiveTheme, PixelsExt, Theme, ThemeConfig, ThemeMode, ThemeRegistry};

use super::font::{set_font_family, set_font_size};

pub fn theme_options(cx: &App) -> Vec<(SharedString, SharedString)> {
    ThemeRegistry::global(cx)
        .sorted_themes()
        .into_iter()
        .map(|theme| (theme.name.clone(), theme.name.clone()))
        .collect()
}

pub fn theme_name(cx: &App) -> SharedString {
    cx.theme().theme_name().clone()
}

fn theme_family(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or(name)
}

fn find_sibling_theme(cx: &App, name: &str, mode: ThemeMode) -> Option<Rc<ThemeConfig>> {
    let family = theme_family(name);
    ThemeRegistry::global(cx)
        .themes()
        .values()
        .find(|theme| theme.mode == mode && theme_family(&theme.name) == family)
        .cloned()
}

pub fn set_theme_name(theme_name: SharedString, cx: &mut App) {
    let Some(config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() else {
        return;
    };

    // Preserve current font choice.
    let current_font_family = cx.theme().font_family.clone();
    let current_font_size = cx.theme().font_size;

    let opposite_mode = if config.mode.is_dark() {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    };
    let sibling = find_sibling_theme(cx, &theme_name, opposite_mode);

    Theme::global_mut(cx).mode = config.mode;
    Theme::global_mut(cx).apply_config(&config);
    if let Some(sibling) = sibling {
        if opposite_mode.is_dark() {
            Theme::global_mut(cx).dark_theme = sibling;
        } else {
            Theme::global_mut(cx).light_theme = sibling;
        }
    }

    set_font_family(current_font_family, cx);
    set_font_size(current_font_size.as_f32(), cx);
}
