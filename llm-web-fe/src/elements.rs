//! Create an HTMLElement.  Reduce boilerplate by putting it here
//! behind functions
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlDivElement};

/// Create an `HTMLDivElement`
/// `document` is the DOM tree
/// `id` is the elements id.  It is up to the uer to ensure this is unique, and is not necessary
pub fn create_div(document: &Document, id: Option<&str>) -> Result<HtmlDivElement, JsValue> {
    let result = document
        .create_element("div")
        .map_err(|err| format!("Error creating DIV for Role editing : {:?}", err))?
        .dyn_into::<HtmlDivElement>()
        .map_err(|err| format!("Error casting to HtmlLabelElement: {:?}", err))?;
    if let Some(id) = id {
        result.set_id(id);
    }
    Ok(result)
}
