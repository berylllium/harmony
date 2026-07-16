use gpui::*;
use gpui_component::setting::{
    SettingField, SettingGroup, SettingItem, SettingPage, Settings as SettingsWidget,
};
use gpui_component::v_flex;

use crate::config::font::{font_family, font_options, font_size, font_size_options};
use crate::config::theme::{is_dark_mode, theme_name, theme_options};
use crate::config::{Config, DARK_MODE, DEFAULT_FONT_SIZE, DEFAULT_THEME_NAME, FONT_FAMILY};

pub struct Settings {
    focus_handle: FocusHandle,
    config: Config,
}

impl Settings {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>, focus_handle: FocusHandle) -> Self {
        Settings { focus_handle, config: Config::default() }
    }

    pub fn view(window: &mut Window, cx: &mut App, focus_handle: FocusHandle) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, focus_handle))
    }
}

impl Render for Settings {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .justify_center()
            .items_center()
            .child(SettingsWidget::new("app-settings").pages(vec![
                SettingPage::new("General").default_open(true).groups(vec![
                    SettingGroup::new()
                        .title("Appearance")
                        .item(SettingItem::new(
                            "Dark Mode",
                            SettingField::switch(is_dark_mode, Config::change_dark_mode)
                                .default_value(DARK_MODE),
                        ))
                        .item(SettingItem::new(
                            "Theme",
                            SettingField::dropdown(theme_options(cx), theme_name, |name, cx| {
                                Config::change_theme(name.to_string(), cx)
                            })
                            .default_value(DEFAULT_THEME_NAME),
                        )),
                    SettingGroup::new()
                        .title("Font")
                        .item(SettingItem::new(
                            "Font",
                            SettingField::dropdown(font_options(cx), font_family, |font, cx| {
                                Config::change_font_family(font.to_string(), cx)
                            })
                            .default_value(FONT_FAMILY),
                        ))
                        .item(SettingItem::new(
                            "Font Size",
                            SettingField::number_input(font_size_options(), font_size, |size, cx| {
                                Config::change_font_size(size as f32, cx)
                            })
                            .default_value(DEFAULT_FONT_SIZE as f64),
                        )),
                ]),
            ]))
    }
}

impl Focusable for Settings {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
