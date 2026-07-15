pub mod welcome;

use std::collections::HashMap;

use gpui::*;
use gpui_component::scroll::ScrollableElement;

use crate::screen::welcome::Welcome;

#[derive(PartialEq, Eq, Hash)]
pub enum Screen {
    Welcome,
}

pub struct ScreenContainer {
    screens: HashMap<Screen, AnyView>,
    current_screen: Screen,
    focus_handle: FocusHandle,
}

impl ScreenContainer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let screens = HashMap::from([(
            Screen::Welcome,
            Welcome::view(window, cx, focus_handle.clone()).into(),
        )]);

        Self {
            screens,
            current_screen: Screen::Welcome,
            focus_handle,
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for ScreenContainer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("screen-container")
            .size_full()
            .overflow_y_scrollbar()
            .track_focus(&self.focus_handle)
            .child(self.screens[&self.current_screen].clone())
    }
}

// pub trait Screen: Render + Sized {
//     fn title() -> &'static str;

//     fn description() -> &'static str {
//         ""
//     }

//     fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render>;
// }
