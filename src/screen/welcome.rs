use gpui::*;
use gpui_component::v_flex;

pub struct Welcome {
    focus_handle: FocusHandle,
}

impl Welcome {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>, focus_handle: FocusHandle) -> Self {
        Welcome { focus_handle }
    }

    pub fn view(window: &mut Window, cx: &mut App, focus_handle: FocusHandle) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, focus_handle))
    }
}

impl Render for Welcome {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .justify_center()
            .items_center()
            .child("Welcome to Harmony!")
            .child("Harmony is configured using a config file.")
    }
}

impl Focusable for Welcome {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// impl Screen for Welcome {
//     fn title() -> &'static str {
//         "Welcome"
//     }

//     fn description() -> &'static str {
//         "The landing screen."
//     }

//     fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render> {
//         Self::view(window, cx)
//     }
// }
