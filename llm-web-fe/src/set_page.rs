use crate::chat_div::ChatDiv;
use crate::filters::text_for_html;
use crate::llm_webpage::LlmWebPage;
use crate::login_div::LoginDiv;
use crate::manipulate_css::add_css_rules;
#[allow(unused_imports)]
use crate::utility::print_to_console;
use chrono::{DateTime, Utc};
use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlButtonElement};
use web_sys::{Document, HtmlElement};

/// Set up the basic page with header, footer, and body.  Called once
/// at start of programme
#[allow(dead_code)]
pub fn initialise_page() -> Result<(), JsValue> {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let body = document.body().expect("Could not access document.body");
    while let Some(child) = body.first_child() {
        let _ = body.remove_child(&child);
    }

    // Set up the three divs
    let footer_div = document.create_element("div")?;
    footer_div.set_id("footer");
    let header_div = document.create_element("div")?;
    header_div.set_id("header");
    let main_body = document.create_element("div")?;
    main_body.set_id("main_body");

    // Add a cost area to display cost
    let cost_div = document.create_element("div")?;
    cost_div.set_id("cost_div");
    header_div.append_child(&cost_div)?;

    // Add a display of time until session expires
    let timeout_div = document.create_element("div")?;
    timeout_div.set_id("timeout_div");
    timeout_div.set_inner_html("<b>&nbsp;</b>");
    header_div.append_child(&timeout_div)?;

    // Add a user area to display user
    let user_div = document.create_element("div")?;
    user_div.set_id("user_div");
    header_div.append_child(&user_div)?;

    // Add a status area
    let status_div = document.create_element("div")?;
    status_div.set_id("status_div");
    footer_div.append_child(&status_div)?;
    // Add the divs
    body.append_child(&footer_div)?;
    body.append_child(&header_div)?;
    body.append_child(&main_body)?;

    // The style.  Sizes given in integer units of percent
    let footer_height = 10;
    let header_height = 10;
    let main_body_height = 100 - (footer_height + header_height);
    let main_width = 100;
    add_css_rules(
        &document,
        "html, body",
        &[("height", "100%"), ("margin", "0"), ("padding", "0")],
    )?;
    add_css_rules(
        &document,
        "#header",
        &[
            ("height", format!("{header_height}%").as_str()),
            ("width", format!("{main_width}%").as_str()),
            ("position", "fixed"),
            ("top", "0"),
            ("left", "0"),
        ],
    )?;
    add_css_rules(
        &document,
        "#sidepanel",
        &[
            ("height", "100%"),
            ("width", "0%"),
            ("position", "fixed"),
            ("top", "0"),
            ("left", "0"),
        ],
    )?;
    add_css_rules(
        &document,
        "#footer",
        &[
            ("height", format!("{footer_height}%").as_str()),
            ("top", format!("{}%", 100 - footer_height).as_str()),
            ("width", format!("{main_width}%").as_str()),
            ("position", "fixed"),
            ("left", "0%"),
        ],
    )?;

    add_css_rules(
        &document,
        "#main_body",
        &[
            ("height", format!("{main_body_height}%").as_str()),
            ("width", format!("{main_width}%").as_str()),
            ("position", "fixed"),
            ("top", format!("{header_height}%").as_str()),
            ("border", "1px solid black"),
            ("left", "0%"),
            ("bottom", format!("{}%", 100 - footer_height).as_str()),
        ],
    )?;

    add_css_rules(
        &document,
        "#cost_div",
        &[("background-color", "#f2fbfa"), ("float", "right")],
    )?;
    add_css_rules(&document, "#footer", &[("border", "1px solid black")])?;
    add_css_rules(
        &document,
        "#header",
        &[("background", "#f8eaea"), ("border", "1px solid black")],
    )?;
    add_css_rules(
        &document,
        "#side_panel_headers_div",
        &[
            ("border-radius", "1em"),
            ("font-size", "small"),
            ("margin", "1em"),
            ("padding", "1em"),
        ],
    )?;
    add_css_rules(
        &document,
        "#side_panel_headers_div",
        &[
            ("display", "inline-block"),
            ("flex-direction", "column"),
            ("align-items", "center"),
            // ("display", "flex"),
        ],
    )?;
    add_css_rules(
        &document,
        "#side_panel_username_input",
        &[("display", "flex"), ("border", "1px solid black")],
    )?;
    add_css_rules(
        &document,
        "#timeout_div",
        &[
            ("float", "right"),
            ("font-family", "sans-serif"),
            ("background-color", "#f2fbfa"),
        ],
    )?;
    add_css_rules(
        &document,
        "#user_div",
        &[("background", "#f9f2d1"), ("float", "left")],
    )?;
    add_css_rules(
        &document,
        "#side_panel_headers_div",
        &[("background", "#ffffef"), ("font-family", "sans-serif")],
    )?;

    start_session_timer()?;

    Ok(())
}

/// The Pages that constitute this web app
#[derive(Debug)]
pub enum Pages {
    ChatDiv,
    LoginDiv,
}
/// Change the "main_body" DIV's content
/// `page` indicates what with
pub fn set_page(page: Pages) -> Result<(), JsValue> {
    // Get the main document
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let body = document.body().expect("Could not access document.body");

    let e = match page {
        Pages::ChatDiv => ChatDiv::initialise_page(&document)?,
        Pages::LoginDiv => LoginDiv::initialise_page(&document)?,
    };
    if let Some(main_body) = document.get_element_by_id("main_body") {
        main_body.set_inner_html("");
        main_body.append_child(&e)?;
        body.append_child(&main_body)?;
    } else {
        print_to_console("No `main_body` in page.  Has not been initialised");
        panic!("Died");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn set_focus_on_element(element_id: &str) {
    let document: &Document = &get_doc();
    if let Some(element) = document.get_element_by_id(element_id) {
        if let Some(input) = element.dyn_ref::<HtmlElement>() {
            input.focus().unwrap();
        } else {
            print_to_console(format!(
                "Failed to set focus. Found {element_id} but is not a HtmlElement.  {element:?}"
            ));
        }
    } else {
        print_to_console(format!(
            "Failed to set focus.  Could not find: {element_id}"
        ));
    }
}

#[allow(dead_code)]
pub fn set_status(status: &str) {
    let document: &Document = &get_doc();
    let status = &text_for_html(status);
    if let Some(status_element) = document.get_element_by_id("status_div") {
        status_element.set_inner_html(status);
    } else {
        print_to_console(format!("Status (No status-div): {status}"));
    }
}

/// Update the cost display
pub fn update_cost_display(document: &Document, credit: f64) {
    let cost_div = document.get_element_by_id("cost_div").unwrap();
    let cost_string = format!("Credit: {credit:.2}\u{00A2}");
    cost_div.set_inner_html(cost_string.as_str());
}

pub fn start_session_timer() -> Result<(), JsValue> {
    let t = Interval::new(1_000, move || {
        update_timeout_display().unwrap();
    });
    t.forget();
    Ok(())
}
fn update_timeout_display() -> Result<(), JsValue> {
    let document: Document = get_doc();
    if let Some(s) = document
        .body()
        .ok_or("send_prompt: Cannot get <body>")?
        .get_attribute("data.expiry")
    {
        let dt = DateTime::parse_from_rfc3339(s.as_str()).expect("Valid rfc3339 time");
        let dt = dt.with_timezone(&Utc);
        let now = Utc::now();
        let delta_t = dt.signed_duration_since(now);

        let timeout_div = document
            .get_element_by_id("timeout_div")
            .ok_or("Failed to get timeout_div")?;

        let timeout = format!(
            "<span id=\"timeout_span\"> Time Remaining: {} </span>",
            if delta_t.num_hours() > 3 {
                format!("{}h", delta_t.num_hours() + 1)
            } else if delta_t.num_hours() > 0 {
                format!("{}h {}m", delta_t.num_hours(), delta_t.num_minutes() % 60)
            } else {
                format!("{}m {}s", delta_t.num_minutes(), delta_t.num_seconds() % 60)
            }
        );
        timeout_div.set_inner_html(timeout.as_str());
    }
    Ok(())
}

/// Display logged in user in header
pub fn update_user_display() {
    let document = get_doc();
    if let Some(t) = document.body().unwrap().get_attribute("data.username") {
        let user_div = document.get_element_by_id("user_div").unwrap();
        user_div.set_inner_html(t.as_str());
    }
}

/// Helper function to make a button.
/// `id`
/// `display` is string to display
pub fn new_button(
    document: &Document,
    id: &str,
    display: &str,
) -> Result<HtmlButtonElement, JsValue> {
    let result: HtmlButtonElement = document
        .create_element("button")
        .map_err(|err| format!("Error creating button element: {:?}", err))?
        .dyn_into::<HtmlButtonElement>()
        .map_err(|err| format!("Error casting to HtmlButtonElement: {:?}", err))?;

    result.set_id(id);
    result.set_inner_text(display);

    Ok(result)
}

/// Helper function to get the Document
pub fn get_doc() -> Document {
    window()
        .and_then(|win| win.document())
        .expect("Failed to get document")
}
