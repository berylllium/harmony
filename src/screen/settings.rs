use gpui::*;
use gpui_component::setting::{
    SettingField, SettingGroup, SettingItem, SettingPage, Settings as SettingsWidget,
};
use gpui_component::v_flex;

use crate::config::Config;
use crate::config::font::{font_family, font_options, font_size, font_size_options};
use crate::config::theme::{theme_name, theme_options};

pub struct Settings {
    focus_handle: FocusHandle,
}

impl Settings {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>, focus_handle: FocusHandle) -> Self {
        Settings { focus_handle }
    }

    pub fn view(window: &mut Window, cx: &mut App, focus_handle: FocusHandle) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, focus_handle))
    }
}

impl Render for Settings {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().justify_center().items_center().child(
            SettingsWidget::new("app-settings").pages(vec![
                SettingPage::new("General").default_open(true).groups(vec![
                    SettingGroup::new()
                        .title("Appearance")
                        .item(SettingItem::new(
                            "Theme",
                            SettingField::dropdown(theme_options(cx), theme_name, |name, cx| {
                                Config::update_global(cx, |config, cx| {
                                    config.theme_name = name.to_string();
                                    config.apply_to_state(cx);
                                })
                            }),
                        )),
                    SettingGroup::new()
                        .title("Font")
                        .item(SettingItem::new(
                            "Font",
                            SettingField::dropdown(font_options(cx), font_family, |font, cx| {
                                Config::update_global(cx, |config, cx| {
                                    config.font_family = font.to_string();
                                    config.apply_to_state(cx);
                                })
                            }),
                        ))
                        .item(SettingItem::new(
                            "Font Size",
                            SettingField::number_input(
                                font_size_options(),
                                font_size,
                                |size, cx| {
                                    Config::update_global(cx, |config, cx| {
                                        config.font_size = size as f32;
                                        config.apply_to_state(cx);
                                    })
                                },
                            ),
                        )),
                ]),
            ]),
        )
    }
}

impl Focusable for Settings {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
