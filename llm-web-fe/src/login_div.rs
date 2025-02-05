use crate::elements::create_div;
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
use web_sys::{Document, HtmlDivElement, HtmlInputElement};
pub struct LoginDiv;

impl LlmWebPage for LoginDiv {
    ///  Generate and return the login screen
    fn initialise_page(document: &Document) -> Result<HtmlDivElement, JsValue> {
        let login_main_div = create_div(document, Some("login-main-div"))?;

        let login_fields_div = create_div(document, Some("login-fields-div"))?;
        login_fields_div.set_class_name("grid-item");

        // Username and pasword elements
        let (username_input, password_input) = username_password_elements("login_div")?;

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

        // Handler to log the user in
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

        // Return the prepared DIV
        Ok(login_main_div)
    }
}

/// Send the login request to the backend
pub fn do_login(username: String, password: String) -> Result<XmlHttpRequest, JsValue> {
    let login_request = LoginRequest {
        username: username.clone(),
        password,
    };
    let login_message = Message::from(login_request);
    make_request(
        login_message,
        move |msg: Message| login_cb(msg, username.clone()),
        || (),
    )
}

/// Callback to respond to a Login request response.  `msg` contains
/// the `LoginResponse` that has details of the successful login or a
/// failure status
fn login_cb(msg: Message, username: String) {
    {
        match msg.comm_type {
            CommType::LoginResponse => {
                let lr: LoginResponse = serde_json::from_str(msg.object.as_str()).unwrap();
                let document = get_doc();
                if lr.success {
                    // Store token and expiry time
                    let token = lr.token.unwrap();
                    let expire = lr.expire;

                    // Store the session data in the DOM
                    let head = document.body().unwrap();
                    head.set_attribute("data.token", token.as_str()).unwrap();
                    head.set_attribute("data.username", username.as_str())
                        .unwrap();
                    head.set_attribute("data.expiry", expire.to_rfc3339().as_str())
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
/// name collisions between the main login page and the side panel
/// which both display login elements
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
