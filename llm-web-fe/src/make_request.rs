/// Make a XmlHttpRequest to the backend.  
use crate::utility::{print_to_console, print_to_console_s};
use llm_web_common::communication::{CommType, Message};
use wasm_bindgen::prelude::*;
use web_sys::{ProgressEvent, XmlHttpRequest};
/// `message` is the data to send to back end
/// `f` is the function to call with the response.  It will call `set_page`

pub fn make_request(
    message: Message,
    mut f: impl FnMut(Message) + 'static,
) -> Result<XmlHttpRequest, JsValue> {
    // print_to_console_s(format!("make_request 1"));
    let api = match message.comm_type {
        CommType::LoginRequest => "login",
        CommType::ChatPrompt => "chat",
        _ => {
            print_to_console_s(format!("make_request Unimplemented: {message}"));
            panic!("Unimplemented")
        }
    };
    let uri = format!("/api/{api}");
    let xhr: XmlHttpRequest = XmlHttpRequest::new().unwrap();
    xhr.open("POST", uri.as_str())?;
    xhr.set_request_header("Content-Type", "application/json")?;
    let xhr_clone = xhr.clone();
    let cb = Closure::wrap(Box::new(move |data: JsValue| {
        match data.dyn_into::<ProgressEvent>() {
            Ok(pe) => print_to_console_s(format!(
                "xhr::onload callback 1 data: {pe:?}  {}/{}",
                pe.loaded(),
                pe.total()
            )),
            Err(err) => print_to_console_s(format!("xhr::onload callback 1 data: {err:?}")),
        };
        if xhr_clone.ready_state() == 4 && xhr_clone.status().unwrap() == 200 {
            print_to_console("xhr::onload callback 1.1");
            let response = xhr_clone.response_text().unwrap().unwrap();
            // Do something with response..
            let message: Message = serde_json::from_str(response.as_str()).unwrap();
            f(message);
            print_to_console("xhr::onload callback after callback ");
        }
    }) as Box<dyn FnMut(_)>);

    xhr.set_onload(Some(cb.as_ref().unchecked_ref()));
    cb.forget();

    let message_str = serde_json::to_string(&message).unwrap();
    xhr.send_with_opt_u8_array(Some(message_str.as_str().as_bytes()))
        .unwrap();
    // }
    // print_to_console("make_request 2");

    Ok(xhr)
}
