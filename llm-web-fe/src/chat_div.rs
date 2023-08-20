use crate::cost_div::cost_div;
use crate::manipulate_css::add_css_rule;
use crate::manipulate_css::clear_css;
#[allow(unused_imports)]
use crate::utility::print_to_console;
#[allow(unused_imports)]
use crate::utility::print_to_console_s;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlInputElement};
/// Screen fo the `chat` model interface
pub fn chat_div(document: &Document) -> Result<Element, JsValue> {
    // The container DIV that arranges the page
    let grid_container = document
        .create_element("div")
        .expect("Could not create DIV element");
    grid_container.set_class_name("grid-container");

    // Display uses budget
    let cost_div = cost_div(document);

    // The response from LLM
    let response_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    response_div.set_id("response-div");
    let response_inp: HtmlInputElement = document
        .create_element("input")
        .map_err(|err| format!("Error creating input element: {:?}", err))?
        .dyn_into::<HtmlInputElement>()
        .map_err(|err| format!("Error casting to HtmlInputElement: {:?}", err))?;
    response_inp.set_value("response");
    response_inp.set_type("text");
    response_div.append_child(&response_inp)?;

    // The entry for the prompt
    let prompt_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    prompt_div.set_id("prompt-div");

    // The button menu
    let button_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    button_div.set_id("button-div");

    // The side_panel menu
    let side_panel_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    side_panel_div.set_id("side-panel-div");

    // The status bar
    let status_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    status_div.set_id("status-div");

    // Put the page together
    grid_container.append_child(&cost_div).unwrap();
    grid_container.append_child(&response_div).unwrap();
    grid_container.append_child(&prompt_div).unwrap();
    grid_container.append_child(&button_div).unwrap();
    grid_container.append_child(&side_panel_div).unwrap();
    grid_container.append_child(&status_div).unwrap();

    // Prepare variables to control page layout

    // Column and row count
    let col = 160;
    let row = 100;

    // Arrange Page:

    // *  bar runs along the complete bottom of the page and is
    // short

    // * Side Panel takes left part of screen from top of screen down
    // to top of the status bar.

    // * The right portion is divided in three, vertically:

    // 1 At the top/right is the cost display, it is small

    // 2 Below that and taking up most of the remaining space is the
    //   display of results.

    // 3 A row of buttons along the bottom, above the status bar

    // Status bar at bottom of page.  Takes 10% of height and full width
    let status_w = col;
    let status_h = (row * 10) / 100;
    let status_t = row - status_h + 1;
    let status_l = 1;

    // Side panel starts at top (1) left (1) and its height is the
    // screen height minus the status bar.The side panel width (span)
    // is to 4/16 of screen width
    let side_panel_t = 1;
    let side_panel_l = 1;
    let side_panel_h = row - status_h;
    let side_panel_w = col * 4 / 16;

    // The cost div is 3/16 of width and on the right, and 10% of height
    let cost_w = (3 * col) / 16;
    let cost_l = col - cost_w + 1;
    let cost_t = 1;
    let cost_h = (row * 10) / 100;

    // The response, prompt, and button panels all have the same left
    // margin and width
    let main_l = side_panel_l + side_panel_w;
    let main_w = col - side_panel_w;

    // The button menu has bottom on top of status menu, is 10% of
    // height, left is right of side panel and extends to edge on
    // right
    let button_t = row - status_h - (row * 10) / 100 + 1;
    let button_h = (row * 10) / 100;
    let button_l = main_l;
    let button_w = main_w;

    // Prompt div height is 10% of total, start div_height above
    // button_t
    let prompt_h = (row * 10) / 100;
    let prompt_t = button_t - prompt_h;
    let prompt_l = main_l;
    let prompt_w = main_w;

    // Response top is below cost, to the right of the side panel,
    // takes all the space left vertically and extends to the right of
    // the screen
    let response_t = cost_t + cost_h;
    let response_l = main_l;
    let response_h = row - (status_h + prompt_h + cost_h);
    let response_w = main_w;

    // Inject the style into the DOM.
    clear_css(document)?;

    add_css_rule(document, "html, body", "height", "100%".to_string())?;
    add_css_rule(document, "html, body", "margin", "0".to_string())?;
    add_css_rule(document, "html, body", "padding", "0".to_string())?;

    add_css_rule(document, ".grid-container", "display", "grid".to_string())?;
    add_css_rule(document, ".grid-container", "height", "100vh".to_string())?;
    add_css_rule(document, ".grid-container", "width", "100vw".to_string())?;
    add_css_rule(document, ".grid-container", "padding", "0".to_string())?;
    add_css_rule(document, ".grid-container", "margin", "0".to_string())?;
    add_css_rule(document, ".grid-container", "overflow", "auto".to_string())?;

    add_css_rule(
        document,
        ".grid-container",
        "grid-template-columns",
        format!("repeat({col}, 1fr)"),
    )?;
    add_css_rule(
        document,
        ".grid-container",
        "grid-template-rows",
        format!("repeat({row}, 1fr)"),
    )?;
    add_css_rule(document, ".grid-container", "gap", ".1em".to_string())?;

    add_css_rule(
        document,
        "#cost-div",
        "grid-column",
        format!("{cost_l} / span {cost_w}"),
    )?;
    add_css_rule(
        document,
        "#cost-div",
        "grid-row",
        format!("{cost_t} / span {cost_h}"),
    )?;
    add_css_rule(
        document,
        "#cost-div",
        "border",
        "thick double #ffeeff".to_string(),
    )?;

    add_css_rule(
        document,
        "#response-div",
        "grid-column",
        format!("{response_l} / span {response_w}"),
    )?;
    add_css_rule(
        document,
        "#response-div",
        "grid-row",
        format!("{response_t} / span {response_h}"),
    )?;
    add_css_rule(
        document,
        "#response-div",
        "border",
        "thick double #00ff00".to_string(),
    )?;

    add_css_rule(
        document,
        "#prompt-div",
        "grid-column",
        format!("{prompt_l} / span {prompt_w}"),
    )?;
    add_css_rule(
        document,
        "#prompt-div",
        "grid-row",
        format!("{prompt_t} / span {prompt_h}"),
    )?;
    add_css_rule(
        document,
        "#prompt-div",
        "border",
        "thick double #ff00ff".to_string(),
    )?;

    add_css_rule(
        document,
        "#button-div",
        "grid-column",
        format!("{button_l} / span {button_w}"),
    )?;
    add_css_rule(
        document,
        "#button-div",
        "grid-row",
        format!("{button_t} / span {button_h}"),
    )?;
    add_css_rule(
        document,
        "#button-div",
        "border",
        "thick double #ffff00".to_string(),
    )?;

    add_css_rule(
        document,
        "#side-panel-div",
        "grid-column",
        format!("{side_panel_l} / span {side_panel_w}"),
    )?;
    add_css_rule(
        document,
        "#side-panel-div",
        "grid-row",
        format!("{side_panel_t} / span {side_panel_h}"),
    )?;
    add_css_rule(
        document,
        "#side-panel-div",
        "border",
        "thick double #ffff00".to_string(),
    )?;

    add_css_rule(
        document,
        "#status-div",
        "grid-column",
        format!("{status_l} / span {status_w}"),
    )?;
    add_css_rule(
        document,
        "#status-div",
        "grid-row",
        format!("{status_t} / span {status_h}"),
    )?;
    add_css_rule(
        document,
        "#status-div",
        "border",
        "thick double #ffff00".to_string(),
    )?;
    response_div.set_inner_html(
        format!("response t,l/WxH {response_t},{response_l}/{response_w}x{response_h}").as_str(),
    );
    prompt_div.set_inner_html(
        format!("prompt t,l/WxH {prompt_t},{prompt_l}/{prompt_w}x{prompt_h}").as_str(),
    );
    button_div.set_inner_html(
        format!("button t,l/WxH {button_t},{button_l}/{button_w}x{button_h}").as_str(),
    );
    status_div.set_inner_html(
        format!("status t,l/WxH {status_t},{status_l}/{status_w}x{status_h}").as_str(),
    );
    side_panel_div.set_inner_html(
        format!("side_panel t,l/WxH {side_panel_t},{side_panel_l}/{side_panel_w}x{side_panel_h}")
            .as_str(),
    );

    Ok(grid_container)
}
