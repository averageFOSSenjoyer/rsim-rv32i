use crate::frontend::core_app::CoreApp;

mod backend;
mod frontend;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1600.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rsim_rv32i",
        native_options,
        Box::new(|cc| Ok(Box::new(CoreApp::new(cc)))),
    )
    .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    console_log::init().unwrap();
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Trace).ok();

    wasm_bindgen_futures::spawn_local(async {
        let web_options = eframe::WebOptions::default();

        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(CoreApp::new(cc)))),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
