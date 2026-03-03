use gpui::*;
use gpui_component::v_flex;

#[derive(IntoElement)]
pub struct Welcome;

impl RenderOnce for Welcome {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .justify_center()
            .items_center()
            .child("Welcome to Harmony!")
            .child("Harmony is configured using a config file.")
    }
}
