use gpui::*;
use gpui_component::{
    ActiveTheme, StyledExt,
    button::{Button, ButtonVariants},
    form::{field, v_form},
    input::{Input, InputState},
    menu::PopupMenuItem::Separator,
    v_flex,
};
use rand::make_rng;

use crate::matrix::{AuthInfo, Matrix, session::SessionMetadata};

pub struct Welcome {
    login_prompt: Entity<Login>,
    focus_handle: FocusHandle,
    matrix: Entity<Matrix>,
}

impl Welcome {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        focus_handle: FocusHandle,
        matrix: Entity<Matrix>,
    ) -> Self {
        Welcome {
            login_prompt: cx.new(|cx| Login::new(window, cx, matrix.clone())),
            focus_handle,
            matrix,
        }
    }

    pub fn view(
        window: &mut Window,
        cx: &mut App,
        focus_handle: FocusHandle,
        matrix: Entity<Matrix>,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, focus_handle, matrix))
    }
}

impl Render for Welcome {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_session = SessionMetadata::exists();
        let matrix = self.matrix.read(cx);

        v_flex()
            .size_full()
            .justify_center()
            .items_center()
            .child(format!(
                "Welcome {}to Harmony!",
                if has_session { "back " } else { "" }
            ))
            .child(match &matrix.connection {
                crate::matrix::ConnectionState::CheckingForSession => {
                    div().child("Checking for an existing Matrix session...")
                }
                crate::matrix::ConnectionState::AwaitingLogin => v_flex()
                    .gap(px(10.0))
                    .child("No existing Matrix session found, please log in below.")
                    .child(self.login_prompt.clone()),
                crate::matrix::ConnectionState::Connecting => div().child("Connecting..."),
                crate::matrix::ConnectionState::Connected(client) => div().child(format!(
                    "Connected to {}.",
                    client.user_id().unwrap().to_string()
                )),
                crate::matrix::ConnectionState::Error(error) => div().child(format!(
                    "An error occured while connecting to Matrix: {}",
                    error
                )),
            })
    }
}

impl Focusable for Welcome {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct Login {
    homeserver_input: Entity<InputState>,
    username_input: Entity<InputState>,
    password_input: Entity<InputState>,
    matrix: Entity<Matrix>,
}

impl Login {
    fn new(window: &mut Window, cx: &mut Context<Self>, matrix: Entity<Matrix>) -> Self {
        Self {
            homeserver_input: cx.new(|cx| InputState::new(window, cx).placeholder("homeserver")),
            username_input: cx.new(|cx| InputState::new(window, cx).placeholder("username")),
            password_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("password")
                    .masked(true)
            }),
            matrix,
        }
    }
}

impl Render for Login {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().items_center().justify_center().child(
            v_flex()
                .gap_1()
                .w(px(360.))
                .p_2()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().secondary)
                .child(Input::new(&self.homeserver_input))
                .child(Input::new(&self.username_input))
                .child(Input::new(&self.password_input).mask_toggle())
                .child(
                    Button::new("log-in")
                        .primary()
                        .label("Log In")
                        .w_full()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.matrix.update(cx, |matrix, cx| {
                                matrix
                                    .auth(
                                        cx,
                                        AuthInfo::Password {
                                            homeserver: this
                                                .homeserver_input
                                                .read(cx)
                                                .value()
                                                .into(),
                                            username: this.username_input.read(cx).value().into(),
                                            password: this.password_input.read(cx).value().into(),
                                        },
                                    )
                                    .detach();
                            })
                        })),
                ),
        )
    }
}
