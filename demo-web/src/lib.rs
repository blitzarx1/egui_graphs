#![cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures as _; // ensure the crate is linked for wasm_bindgen async
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    // Fire and forget: kick off the async runner
    wasm_bindgen_futures::spawn_local(async {
        let _ = run().await;
    });
    Ok(())
}

#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id("the_canvas_id")
        .ok_or_else(|| JsValue::from_str("canvas with id 'the_canvas_id' not found"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("failed to cast to HtmlCanvasElement"))?;

    let web_options = eframe::WebOptions::default();
    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| Ok::<Box<dyn eframe::App>, _>(Box::new(demo_core::DemoApp::new(cc)))),
        )
        .await?;
    Ok(())
}
