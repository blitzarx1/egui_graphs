#[cfg(target_arch = "wasm32")]
pub fn download_json(filename: &str, content: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use web_sys::{window, Blob, BlobPropertyBag, HtmlElement, Url};

    let window = window().ok_or_else(|| "no window".to_string())?;
    let document = window.document().ok_or_else(|| "no document".to_string())?;

    let props = BlobPropertyBag::new();
    props.set_type("application/json;charset=utf-8");
    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(content));
    let blob =
        Blob::new_with_blob_sequence_and_options(&parts, &props).map_err(|_| "blob".to_string())?;

    let url = Url::create_object_url_with_blob(&blob).map_err(|_| "url".to_string())?;

    let a = document
        .create_element("a")
        .map_err(|_| "a".to_string())?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "anchor".to_string())?;
    a.set_href(&url);
    a.set_download(filename);
    a.set_hidden(true);

    let body = document.body().ok_or_else(|| "body".to_string())?;
    body.append_child(&a).map_err(|_| "append".to_string())?;

    let a_el: &HtmlElement = a.as_ref();
    a_el.click();

    body.remove_child(&a).ok();
    Url::revoke_object_url(&url).ok();
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn download_json(_filename: &str, _content: &str) -> Result<(), String> {
    Err("not supported on native".to_string())
}

/// Ensure the filename is safe and ends with .json. Strip path separators and disallowed characters.
pub fn sanitize_filename(input: &str) -> String {
    let mut name = input.trim().to_string();
    if name.is_empty() {
        name = "graph_export.json".to_string();
    }
    // Replace path separators and reserved characters with '_'
    let disallowed: [char; 10] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0'];
    name = name
        .chars()
        .map(|c| if disallowed.contains(&c) { '_' } else { c })
        .collect();
    // Prevent directory traversal by removing leading dots/spaces
    while name.starts_with(['.', ' ']) {
        name.remove(0);
    }
    if name.is_empty() {
        name = "graph_export.json".to_string();
    }
    if !name.to_lowercase().ends_with(".json") {
        name.push_str(".json");
    }
    name
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_export_filename() -> String {
    // Use local time; avoid ':' in filenames for portability
    let now = chrono::Local::now();
    let s = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    format!("egui_graphs_export_{}.json", s)
}

#[cfg(target_arch = "wasm32")]
pub fn default_export_filename() -> String {
    let d = js_sys::Date::new_0();
    let year = d.get_full_year();
    let month = (d.get_month() + 1) as u32; // 0-based
    let day = d.get_date() as u32;
    let h = d.get_hours() as u32;
    let m = d.get_minutes() as u32;
    let s = d.get_seconds() as u32;
    fn pad2(x: u32) -> String {
        if x < 10 {
            format!("0{}", x)
        } else {
            x.to_string()
        }
    }
    let ts = format!(
        "{:04}-{}-{}_{}-{}-{}",
        year,
        pad2(month),
        pad2(day),
        pad2(h),
        pad2(m),
        pad2(s)
    );
    format!("egui_graphs_export_{}.json", ts)
}
