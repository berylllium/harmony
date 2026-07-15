mod assets;
mod dashboard;
mod logging;
mod screen;
mod theme;

use gpui::*;
use gpui_component::*;

use crate::{assets::Assets, dashboard::Dashboard};

fn main() {
    logging::init();

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        theme::init(cx);

        Assets::load_fonts(cx).unwrap();

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = Dashboard::view(window, cx);

                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<(), anyhow::Error>(())
        })
        .detach();
    });
}
