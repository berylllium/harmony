use gpui::*;
use gpui_component::h_flex;

use crate::matrix::{ConnectionState, Matrix};

#[derive(IntoElement)]
pub struct Header {
    pub screen_title: String,
}

impl RenderOnce for Header {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .size_full()
            .child(
                div()
                    .id("header-title")
                    .h_full()
                    .w(relative(1.0))
                    .text_ellipsis()
                    .text_xl()
                    .child(self.screen_title),
            )
            .child(
                div()
                    .id("header-info")
                    .h_full()
                    .w(relative(2.0))
                    .text_align(TextAlign::Right)
                    .child(format!(
                        "Matrix State: {}",
                        match Matrix::global(cx).connection {
                            ConnectionState::CheckingForSession => "Checking for session...",
                            ConnectionState::AwaitingLogin => "Logged Out",
                            ConnectionState::Connecting => "Connecting...",
                            ConnectionState::Connected(_) => "Connected",
                            ConnectionState::Error(_) => "Error",
                        }
                    )),
            )
    }
}
