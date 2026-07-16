use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::Button,
    h_flex,
    resizable::{h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::{
    assets::IconName,
    screen::{Screen, ScreenContainer},
};

pub struct Dashboard {
    screen_container: Entity<ScreenContainer>,
    sidebar_collapsed: bool,
}

impl Dashboard {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            screen_container: ScreenContainer::view(window, cx),
            sidebar_collapsed: false,
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body =
            h_resizable("dashboard-container")
                .child(
                    resizable_panel()
                        .size(px(255.0))
                        .size_range(px(200.0)..px(320.0))
                        .child(
                            Sidebar::left()
                                .w(relative(1.0))
                                .border_0()
                                .collapsed(self.sidebar_collapsed)
                                .child(SidebarGroup::new("Spaces").child(SidebarMenu::new().child(
                                    SidebarMenuItem::new("Test space 1").icon(IconName::House),
                                )))
                                .child(SidebarGroup::new("Rooms").child(SidebarMenu::new().child(
                                    SidebarMenuItem::new("Test room 1").icon(IconName::Frame),
                                )))
                                .footer(
                                    Button::new("settings-btn")
                                        .icon(gpui_component::IconName::Settings)
                                        .tooltip("Settings")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.screen_container.update(cx, |sc, cx| {
                                                sc.set_screen(Screen::Settings, cx);
                                            });
                                        })),
                                ),
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .h_full()
                        .overflow_x_hidden()
                        .child(
                            h_flex()
                                .id("header")
                                .p_4()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .justify_between()
                                .items_start()
                                .child(div().text_xl().child("Test Header")),
                        )
                        .child(
                            div()
                                .id("screen")
                                .flex_1()
                                .child(self.screen_container.clone()),
                        )
                        .into_any_element(),
                );

        v_flex()
            .size_full()
            .child(div().flex_1().min_h_0().child(body))
    }
}
