use crate::llm_webpage::LlmWebPage;
use crate::make_request::make_request;
use crate::manipulate_css::add_css_rules;
use crate::set_page::get_doc;
use crate::set_page::set_page;
use crate::set_page::set_status;
use crate::set_page::update_cost_display;
use crate::set_page::update_user_display;
use crate::set_page::Pages;
#[allow(unused_imports)]
use crate::utility::print_to_console;
use gloo_events::EventListener;
use llm_web_common::communication::CommType;
use llm_web_common::communication::LoginRequest;
use llm_web_common::communication::LoginResponse;
use llm_web_common::communication::Message;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::XmlHttpRequest;
use web_sys::{Document, Element, HtmlInputElement};
pub struct LoginDiv;
impl LlmWebPage for LoginDiv {
    ///  The login screen
    fn initialise_page(document: &Document) -> Result<Element, JsValue> {

        let login_main_div = document.create_element("div")?;
        login_main_div.set_id("login-main-div");

        let login_fields_div = document.create_element("div")?;
        login_fields_div.set_class_name("grid-item");
        login_fields_div.set_id("login-fields-div");

        // Username and pasword elements
        let (username_input, password_input) = username_password_elements("login_div")?;

        // Hack so logging in quicker
        // username_input.set_attribute("value", "a")?;
        // password_input.set_attribute("value", "b")?;

        // Login button
        let user_text_submit = document.create_element("button")?;
        user_text_submit.set_id("user_text_submit");
        user_text_submit.set_inner_html("Login");

        // Assemble pieces
        login_fields_div.append_child(&username_input)?;
        login_fields_div.append_child(&password_input)?;
        login_fields_div.append_child(&user_text_submit)?;
        login_main_div.append_child(&login_fields_div)?;
        add_css_rules(
            document,
            "html, body",
            &[("height", "100%"), ("margin", "0")],
        )?;
        add_css_rules(
            document,
            "#login-fields-div > input",
            &[
                ("margin", "1em"),
                ("border", "2px solid black"),
                ("width", "50%"),
                ("display", "flex"),
                ("flex-direction", "column"),
            ],
        )?;
        add_css_rules(document, "#login-fields-div", &[("padding", "10px")])?;

        let on_click = EventListener::new(&user_text_submit, "click", move |_event| {
            let username: String = if let Some(input) = username_input.dyn_ref::<HtmlInputElement>()
            {
                input.value()
            } else {
                "".to_string()
            };
            let password: String = if let Some(input) = password_input.dyn_ref::<HtmlInputElement>()
            {
                input.value()
            } else {
                "".to_string()
            };
            _ = do_login(username, password).unwrap();
        });
        on_click.forget();
        Ok(login_main_div)
    }
}

pub fn do_login(username: String, password: String) -> Result<XmlHttpRequest, JsValue> {
    let login_request = LoginRequest {
        username: username.clone(),
        password,
    };
    let login_message = Message::from(login_request);
    let u = username.clone();

    make_request(
        login_message,
        move |msg: Message| login_cb(msg, u.clone()),
        || (),
    )
}

fn login_cb(msg: Message, username: String) {
    {
        match msg.comm_type {
            CommType::LoginResponse => {
                let lr: LoginResponse = serde_json::from_str(msg.object.as_str()).unwrap();
                let document = get_doc();
                if lr.success {
                    // Store token and expiry time
                    let token = lr.token.unwrap();
                    set_status(format!("Setting token: {token}").as_str());

                    let head = document.body().unwrap();
                    head.set_attribute("data.token", token.as_str()).unwrap();
                    head.set_attribute("data.username", username.as_str())
                        .unwrap();
                    head.set_attribute("data.expiry", lr.expire.to_rfc3339().as_str())
                        .unwrap();
                    set_page(Pages::ChatDiv).unwrap();
                    update_cost_display(&document, lr.credit);
                    update_user_display();
                } else {
                    set_status("Login failed");
                    set_page(Pages::LoginDiv).unwrap();
                }
            }
            _ => panic!("Expected LoginResponse got {}", msg),
        };
    };
}

/// The pair of HtmlInputElements for logging in.  `prefix` to avoid
/// name collisions
pub fn username_password_elements(
    prefix: &str,
) -> Result<(HtmlInputElement, HtmlInputElement), JsValue> {
    let document = get_doc();
    let username_input = document
        .create_element("input")?
        .dyn_into::<HtmlInputElement>()?;
    let password_input = document
        .create_element("input")?
        .dyn_into::<HtmlInputElement>()?;
    username_input.set_id(format!("{prefix}_username_input").as_str());
    password_input.set_id(format!("{prefix}_password_input").as_str());
    username_input.set_attribute("type", "text")?;
    password_input.set_attribute("type", "password")?;
    username_input.set_attribute("autocomplete", "username")?;
    password_input.set_attribute("autocomplete", "current-password")?;
    username_input.set_attribute("placeholder", "username")?;
    password_input.set_attribute("placeholder", "password")?;
    Ok((username_input, password_input))
}
