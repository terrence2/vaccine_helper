use anyhow::{anyhow, Result};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    window, Blob, BlobPropertyBag, Event, File, FileReader, HtmlAnchorElement, HtmlInputElement,
    Url,
};

pub fn download_file(data: &str, filename: &str, mime_type: &str) -> Result<()> {
    download_file_inner(data, filename, mime_type).map_err(|_| anyhow!("a js error occurred"))
}

pub fn download_file_inner(
    data: &str,
    filename: &str,
    mime_type: &str,
) -> std::result::Result<(), JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();

    // Create blob
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(data));

    let blob_props = BlobPropertyBag::new();
    blob_props.set_type(mime_type);

    let blob = Blob::new_with_str_sequence_and_options(&blob_parts, &blob_props)?;

    // Create URL and download
    let url = Url::create_object_url_with_blob(&blob)?;
    let anchor: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    Url::revoke_object_url(&url)?;
    Ok(())
}

pub fn create_file_picker<F>(callback: F) -> Result<()>
where
    F: Fn(String) + 'static,
{
    create_file_picker_inner(callback).map_err(|_| anyhow!("a js error occurred"))
}

fn create_file_picker_inner<F>(callback: F) -> std::result::Result<(), JsValue>
where
    F: Fn(String) + 'static,
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Create file input
    let input: HtmlInputElement = document.create_element("input")?.dyn_into()?;
    input.set_type("file");
    input.set_accept(".ron"); // optional: limit file types

    // Handle file selection
    let callback = Rc::new(callback);
    let onchange = Closure::wrap(Box::new(move |event: Event| {
        let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            let callback = callback.clone();
            read_file_content(file, move |content| {
                callback(content);
            });
        }
    }) as Box<dyn FnMut(_)>);

    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();

    // Trigger file dialog
    input.click();
    Ok(())
}

fn read_file_content<F>(file: File, callback: F)
where
    F: Fn(String) + 'static,
{
    let reader = FileReader::new().unwrap();

    let onload = Closure::wrap(Box::new(move |_event: Event| {
        let reader: FileReader = _event.target().unwrap().dyn_into().unwrap();
        if let Ok(result) = reader.result() {
            let content = result.as_string().unwrap_or_default();
            callback(content);
        }
    }) as Box<dyn FnMut(_)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    reader.read_as_text(&file).unwrap();
}
