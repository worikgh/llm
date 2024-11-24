use crate::chat_div::ChatDiv;
use crate::filters::text_for_html;
use crate::llm_webpage::LlmWebPage;
use crate::login_div::LoginDiv;
use crate::manipulate_css::add_css_rule;
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
    add_css_rule(&document, "html, body", "height", "100%")?;
    add_css_rule(&document, "html, body", "margin", "0")?;
    add_css_rule(&document, "html, body", "padding", "0")?;
    add_css_rule(&document, "#header", "height", format!("{header_height}%"))?;
    add_css_rule(&document, "#header", "width", format!("{main_width}%"))?;
    add_css_rule(&document, "#header", "position", "fixed")?;
    add_css_rule(&document, "#header", "top", "0")?;
    add_css_rule(&document, "#header", "left", "0")?;
    add_css_rule(&document, "#sidepanel", "height", "100%")?;
    add_css_rule(&document, "#sidepanel", "width", "0%")?;
    add_css_rule(&document, "#sidepanel", "position", "fixed")?;
    add_css_rule(&document, "#sidepanel", "top", "0")?;
    add_css_rule(&document, "#sidepanel", "left", "0")?;
    add_css_rule(&document, "#footer", "height", format!("{footer_height}%"))?;
    add_css_rule(&document, "#footer", "width", format!("{main_width}%"))?;
    add_css_rule(&document, "#footer", "position", "fixed")?;
    add_css_rule(
        &document,
        "#footer",
        "top",
        format!("{}%", 100 - footer_height),
    )?;
    add_css_rule(&document, "#footer", "left", "0%")?;

    add_css_rule(
        &document,
        "#main_body",
        "height",
        format!("{main_body_height}%"),
    )?;
    add_css_rule(&document, "#main_body", "width", format!("{main_width}%"))?;
    add_css_rule(&document, "#main_body", "position", "fixed")?;
    add_css_rule(&document, "#main_body", "top", format!("{header_height}%"))?;
    add_css_rule(&document, "#main_body", "left", "0%")?;
    add_css_rule(
        &document,
        "#main_body",
        "bottom",
        format!("{}%", 100 - footer_height),
    )?;
    add_css_rule(&document, "#sidepanel", "border", "1px solid black")?;
    add_css_rule(&document, "#footer", "border", "1px solid black")?;

    add_css_rule(&document, "#header", "border", "1px solid black")?;
    add_css_rule(&document, "#header", "background", "#f8eaea")?;
    add_css_rule(&document, "#main_body", "border", "1px solid black")?;

    add_css_rule(&document, "#timeout_div", "float", "right")?;
    add_css_rule(&document, "#timout_div", "background-color", "#f2fbfa")?;
    add_css_rule(&document, "#cost_div", "float", "right")?;
    add_css_rule(&document, "#cost_div", "background-color", "#f2fbfa")?;
    add_css_rule(&document, "#user_div", "float", "left")?;
    add_css_rule(&document, "#user_div", "background", "#f9f2d1")?;

    add_css_rule(&document, "#side_panel_username_input", "display", "flex")?;
    add_css_rule(&document, "#side_panel_login_div", "display", "flex")?;
    add_css_rule(
        &document,
        "#side_panel_login_div",
        "flex-direction",
        "column",
    )?;
    add_css_rule(&document, "#side_panel_login_div", "align-items", "center")?;

    add_css_rule(
        &document,
        "#side_panel_headers_div",
        "display",
        "inline-block",
    )?;
    add_css_rule(
        &document,
        "#side_panel_headers_div",
        "background",
        "#ffffef",
    )?;
    add_css_rule(&document, "#side_panel_headers_div", "border-radius", "1em")?;
    add_css_rule(&document, "#side_panel_headers_div", "padding", "1em")?;
    add_css_rule(&document, "#side_panel_headers_div", "margin", "1em")?;
    add_css_rule(&document, "#side_panel_headers_div", "font-size", "small")?;
    add_css_rule(
        &document,
        "#side_panel_headers_div",
        "font-family",
        "sans-serif",
    )?;
    add_css_rule(&document, "#timeout_div", "font-family", "sans-serif")?;
    add_css_rule(&document, "#timeout_div", "margin-right", "")?;

    add_css_rule(&document, "#timeout_div", "", "")?;
    add_css_rule(&document, "#timeout_div", "", "")?;
    // # {
    //   float: right;
    //   margin-right: 2em;
    //   background: aliceblue;
    //   border: .25em solid #e4eaf0;
    //   border-radius: 7pt;
    // }

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
fn update_timeout_display() -> Result<(), JsValue>{
    let document: Document = get_doc();
    if let Some(s) = document
	.body()
	.ok_or("send_prompt: Cannot get <body>")?
	.get_attribute("data.expiry"){
	    let dt = DateTime::parse_from_rfc3339(s.as_str()).expect("Valid rfc3339 time");
	    let dt = dt.with_timezone(&Utc);
	    let now = Utc::now();
	    let delta_t = dt.signed_duration_since(now);

	    let timeout_div = document.get_element_by_id("timeout_div").ok_or("Failed to get timeout_div")?;
	    let timeout = if delta_t.num_hours() > 3 {
		format!("{}h", delta_t.num_hours() + 1)
	    }else if delta_t.num_hours() > 0 {
		format!("{}h {}m", delta_t.num_hours(), delta_t.num_minutes() % 60)
	    }else{
		format!("{}m {}s", delta_t.num_minutes(), delta_t.num_seconds() % 60)
	    };
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
