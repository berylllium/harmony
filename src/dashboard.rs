pub mod header;

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
    assets::IconName, dashboard::header::Header, matrix::Matrix, screen::{Screen, ScreenContainer},
};

pub struct Dashboard {
    screen_container: Entity<ScreenContainer>,
    sidebar_collapsed: bool,
    matrix: Entity<Matrix>,
}

impl Dashboard {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let matrix = Matrix::entity(cx);

        Self {
            screen_container: ScreenContainer::view(window, cx, matrix.clone()),
            sidebar_collapsed: false,
            matrix,
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn get_available_rooms(&self, cx: &App) -> Vec<String> {
        self.matrix
            .read(cx)
            .rooms
            .iter()
            .map(|room| {
                room.cached_display_name()
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| room.room_id().to_string())
            })
            .collect()
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
                                .child(SidebarGroup::new("Rooms").child(SidebarMenu::new().children(
                                    self.get_available_rooms(cx).into_iter().map(|room_name| {
                                        SidebarMenuItem::new(room_name)
                                            .icon(IconName::Frame)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.screen_container.update(cx, |sc, cx| {
                                                    sc.set_screen(Screen::Chat);
                                                });
                                            }))
                                    }),
                                )))
                                .footer(
                                    Button::new("settings-btn")
                                        .icon(gpui_component::IconName::Settings)
                                        .tooltip("Settings")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.screen_container.update(cx, |sc, cx| {
                                                sc.set_screen(Screen::Settings);
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
                                .w(relative(1.0))
                                .h(px(60.0))
                                .p_4()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .justify_between()
                                .items_start()
                                .child(Header {
                                    screen_title: self
                                        .screen_container
                                        .read(cx)
                                        .current_screen
                                        .label()
                                        .to_owned(),
                                    matrix: self.matrix.clone(),
                                }),
                        )
                        .child(
                            div()
                                .id("screen")
                                .size_full()
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
