pub mod chat;
pub mod settings;
pub mod welcome;

use std::collections::HashMap;

use gpui::*;

use crate::screen::{chat::Chat, settings::Settings, welcome::Welcome};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Screen {
    Welcome,
    Settings,
    Chat,
}

pub struct ScreenContainer {
    pub screens: HashMap<Screen, AnyView>,
    pub current_screen: Screen,
    pub focus_handle: FocusHandle,
}

impl ScreenContainer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let screens = HashMap::from([
            (
                Screen::Welcome,
                Welcome::view(window, cx, focus_handle.clone()).into(),
            ),
            (
                Screen::Settings,
                Settings::view(window, cx, focus_handle.clone()).into(),
            ),
            (
                Screen::Chat,
                Chat::view(window, cx, focus_handle.clone()).into(),
            ),
        ]);

        Self {
            screens,
            current_screen: Screen::Welcome,
            focus_handle,
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }
}

impl Render for ScreenContainer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("screen-container")
            .size_full()
            .track_focus(&self.focus_handle)
            .child(self.screens[&self.current_screen].clone())
    }
}

impl Screen {
    pub fn label(&self) -> &'static str {
        match self {
            Screen::Welcome => "Welcome",
            Screen::Settings => "Settings",
            Screen::Chat => "Chat",
        }
    }
}
