use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    v_flex,
};

use crate::{
    environment,
    matrix::{AuthInfo, Matrix, session::SessionMetadata},
};

pub struct Welcome {
    login_prompt: Entity<Login>,
    focus_handle: FocusHandle,
}

impl Welcome {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, focus_handle: FocusHandle) -> Self {
        Welcome {
            login_prompt: cx.new(|cx| Login::new(window, cx)),
            focus_handle,
        }
    }

    pub fn view(window: &mut Window, cx: &mut App, focus_handle: FocusHandle) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, focus_handle))
    }
}

impl Render for Welcome {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_session = SessionMetadata::exists();
        let matrix = Matrix::global(cx);

        v_flex()
            .size_full()
            .justify_center()
            .items_center()
            .child(format!(
                "Welcome {}to Harmony v{}!",
                if has_session { "back " } else { "" },
                environment::VERSION
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
}

impl Login {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            homeserver_input: cx.new(|cx| InputState::new(window, cx).placeholder("homeserver")),
            username_input: cx.new(|cx| InputState::new(window, cx).placeholder("username")),
            password_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("password")
                    .masked(true)
            }),
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
                            Matrix::update_global(cx, |matrix, cx| {
                                let homeserver = this.homeserver_input.read(cx).value().into();
                                let username = this.username_input.read(cx).value().into();
                                let password = this.password_input.read(cx).value().into();
                                matrix
                                    .auth(
                                        cx,
                                        AuthInfo::Password {
                                            homeserver,
                                            username,
                                            password,
                                        },
                                    )
                                    .detach();
                            })
                        })),
                ),
        )
    }
}
