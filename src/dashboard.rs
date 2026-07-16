pub mod header;

use gpui::*;
use gpui_component::{
    ActiveTheme, h_flex,
    resizable::{h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::{assets::IconName, dashboard::header::Header, matrix::Matrix, screen::ScreenContainer};

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
                                ))),
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
