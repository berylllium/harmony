mod assets;
mod logging;
mod screen;
mod theme;

use gpui::*;
use gpui_component::{button::*, *};

use crate::assets::Assets;

pub struct Harmony;

impl Render for Harmony {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        _: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .w(relative(1.0))
            .h(relative(1.0))
            .items_center()
            .justify_center()
            .font_family(SharedString::from("Iosevka"))
            .child(screen::welcome::Welcome)
    }
}

fn main() {
    logging::init();

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        theme::init(cx);

        Assets.load_fonts(cx).unwrap();

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| Harmony);

                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<(), anyhow::Error>(())
        })
        .detach();
    });
}
