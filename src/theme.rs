use gpui::App;
use gpui_component::{Theme, ThemeRegistry};

const DEFAULT_THEME_NAME: &str = "Gruvbox Dark";

pub fn init(cx: &mut App) {
    if let Err(err) =
        ThemeRegistry::watch_dir(std::path::PathBuf::from("./themes"), cx, move |cx| {
            if let Some(theme) = ThemeRegistry::global(cx)
                .themes()
                .get(DEFAULT_THEME_NAME)
                .cloned()
            {
                Theme::global_mut(cx).apply_config(&theme);
            }
        })
    {
        tracing::error!("Failed to watch themes directory: {}", err);
    }
}
