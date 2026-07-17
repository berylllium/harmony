use gpui::*;
use gpui_component::{
    ActiveTheme, StyledExt,
    avatar::Avatar,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    scroll::ScrollableElement,
    v_flex,
};

struct ChatMessage {
    sender: &'static str,
    timestamp: &'static str,
    body: &'static str,
}

fn test_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            sender: "Berkay",
            timestamp: "10:02",
            body: "test",
        },
        ChatMessage {
            sender: "Paart",
            timestamp: "10:03",
            body: "test2",
        },
    ]
}

pub struct Chat {
    focus_handle: FocusHandle,
    message_input: Entity<InputState>,
}

impl Chat {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, focus_handle: FocusHandle) -> Self {
        Chat {
            focus_handle,
            message_input: cx.new(|cx| InputState::new(window, cx).placeholder("Message...")),
        }
    }

    pub fn view(window: &mut Window, cx: &mut App, focus_handle: FocusHandle) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, focus_handle))
    }
}

impl Render for Chat {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                div()
                    .id("chat-messages")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_4()
                    .child(
                        v_flex()
                            .gap_4()
                            .children(test_messages().into_iter().map(|message| {
                                h_flex()
                                    .items_start()
                                    .gap_3()
                                    .child(Avatar::new().name(message.sender))
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .child(
                                                h_flex()
                                                    .gap_2()
                                                    .items_baseline()
                                                    .child(
                                                        div()
                                                            .font_bold()
                                                            .child(message.sender),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(message.timestamp),
                                                    ),
                                            )
                                            .child(div().child(message.body)),
                                    )
                            })),
                    ),
            )
            .child(
                h_flex()
                    .p_3()
                    .gap_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(Input::new(&self.message_input).flex_1())
                    .child(Button::new("send-message").primary().label("Send")),
            )
    }
}

impl Focusable for Chat {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
