mod assets;
mod config;
mod dashboard;
mod environment;
mod logging;
mod screen;

use gpui::*;
use gpui_component::*;

use crate::{assets::Assets, config::Config, dashboard::Dashboard};

fn main() {
    logging::init();

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let config = Config::load_config().unwrap();
        cx.set_global(config);

        Assets::load_resources(cx).unwrap();

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
