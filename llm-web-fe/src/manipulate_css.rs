#[allow(unused_imports)]
use crate::utility::print_to_console;
use std::collections::BTreeMap;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::CssRule;
use web_sys::CssRuleList;
use web_sys::CssStyleDeclaration;
use web_sys::StyleSheet;
use web_sys::StyleSheetList;
use web_sys::{Document, HtmlStyleElement};
fn get_style_element(document: &Document) -> Result<HtmlStyleElement, JsValue> {
    // Check if a style element already exists, otherwise create a new one
    if let Some(existing_style) = document.query_selector("style")? {
        Ok(existing_style.dyn_into::<HtmlStyleElement>()?)
    } else {
        let style_element = document
            .create_element("style")?
            .dyn_into::<HtmlStyleElement>()?;
        document.head().unwrap().append_child(&style_element)?;
        Ok(style_element)
    }
}

/// Struct for initialising CSS rules
#[derive(Debug, Clone)]
pub struct CssRules {
    pub selector_rules: BTreeMap<String, BTreeMap<String, String>>,
}

impl CssRules {
    #[allow(dead_code)]
    pub fn insert(&mut self, selector: &str, rule: &str, value: &str) -> Result<(), JsValue> {
        if !self.selector_rules.contains_key(selector) {
            self.selector_rules
                .insert(selector.to_string(), BTreeMap::new());
        }
        self.selector_rules
            .get_mut(selector)
            .unwrap()
            .insert(rule.to_string(), value.to_string());
        Ok(())
    }
}

impl fmt::Display for CssRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (selector, rules) in &self.selector_rules {
            writeln!(f, "{} {{", selector)?;
            for (property, value) in rules {
                writeln!(f, "  {}: {};", property, value)?;
            }
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

/// Read the style sheets from the Document and collect all the
/// selectors and have a set of (property/value) pairs for each
/// selector
#[allow(dead_code)]
pub fn get_css_rules(document: &Document) -> Result<CssRules, JsValue> {
    let mut result: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    let style_sheets: StyleSheetList = document.style_sheets();

    for i in 0..style_sheets.length() {
        // For each style sheet.  Forced unwrap OK because `i` is
        // confined to a range

        let style_sheet: StyleSheet = style_sheets.get(i).unwrap();
        let css_style_sheet =
            match wasm_bindgen::JsCast::dyn_into::<web_sys::CssStyleSheet>(style_sheet) {
                Ok(css) => css,
                Err(err) => {
                    print_to_console(format!("{err:?} Not a CssStyleSheet"));
                    continue;
                }
            };
        // Got a CssStyleSheet
        let href = css_style_sheet.href()?.unwrap_or("no title".to_string());
        if href.contains("google_ads_iframe") {
            // Wierd rules causing problems.  WHere do these come from?
            continue;
        }

        let css_rules: CssRuleList = css_style_sheet.css_rules()?;
        for j in 0..css_rules.length() {
            // Forced unrwap OK because `j` is in a range
            let css_rule: CssRule = css_rules.get(j).unwrap();
            let css_style_rule =
                match wasm_bindgen::JsCast::dyn_into::<web_sys::CssStyleRule>(css_rule) {
                    Ok(c) => c,
                    Err(err) => {
                        print_to_console(format!("{err:?} Not a CssStyleRule"));
                        continue;
                    }
                };

            let selector = css_style_rule.selector_text();
            let scc_style_dec: CssStyleDeclaration = css_style_rule.style();

            // Make sure the rules are initialised
            if !result.contains_key(&selector) {
                result.insert(selector.clone(), BTreeMap::new());
            }

            for k in 0..scc_style_dec.length() {
                let property_name: String = match scc_style_dec.get(k) {
                    Some(s) => s,
                    None => {
                        print_to_console(format!("{i}/{j}/{k} no style text"));
                        continue;
                    }
                };
                let value: String =
                    scc_style_dec.get_property_value(property_name.clone().as_str())?;
                // At this point got the selector, the property name,,
                // and value of the CSS rule
                let v = result.get_mut(&selector).unwrap();
                v.insert(property_name, value);
            }
        }
    }

    Ok(CssRules {
        selector_rules: result,
    })
}

/// Add in the rules in the passed container
#[allow(dead_code)]
pub fn set_css_rules(document: &Document, css_rules: &CssRules) -> Result<(), JsValue> {
    let style_sheets: StyleSheetList = document.style_sheets();
    let lim_i = style_sheets.length();
    assert!(lim_i > 0);
    let style_sheet: StyleSheet = style_sheets.get(0).unwrap();
    let css_style_sheet =
        wasm_bindgen::JsCast::dyn_into::<web_sys::CssStyleSheet>(style_sheet).unwrap();

    for (selector, v) in css_rules.selector_rules.iter() {
        for (rule, value) in v.iter() {
            let rule = format!("{selector}{{{rule}:{value}}}");
            css_style_sheet.insert_rule(rule.as_str())?;
        }
    }
    Ok(())
}

/// Add style rules to the DOM.
/// Generic parameter `T` allows `value` to be `&str` or `String`
pub fn add_css_rules(
    document: &Document,
    selector: &str,
    properties_values: &[(&str, &str)],
) -> Result<(), JsValue> {
    for pv in properties_values.iter() {
        let (p, v) = pv;
        add_css_rule(document, selector, p, v)?;
    }
    Ok(())
}

/// Add a style rule to the DOM.
/// Generic parameter `T` allows `value` to be `&str` or `String`
fn add_css_rule(
    document: &Document,
    selector: &str,
    property: &str,
    value: &str,
) -> Result<(), JsValue> {
    let value: String = value.into();
    let selector: String = selector.into();
    let property: String = property.into();
    let value: String = value;
    // Check if the style element already contains CSS rules

    if let Some(rules) = get_css_rules(document)?
        .selector_rules
        .get(selector.as_str())
    {
        // The selector is registered

        if let Some(v) = rules.get(property.as_str()) {
            // The property declared for this rule
            if v == &value {
                // Rule already there
                return Ok(());
            } else {
                // Rule exists with a different value
                panic!(
                    "{selector}/{property} has value: {v}.  We want to set: {value}",
                );
            }
        }
    }

    let style_element: HtmlStyleElement = get_style_element(document)?;
    let existing_css = style_element.inner_html();
    let css_rule = format!("{} {{ {}: {} }}\n", selector, property, value); //

    if existing_css.is_empty() {
        // If no rules present, set the CSS rule
        style_element.set_inner_html(&css_rule);
    } else {
        // Append the new rule to the existing CSS rules
        style_element.set_inner_html(&format!("{}{}", existing_css, css_rule));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn clear_css(document: &Document) -> Result<(), JsValue> {
    // See: https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/deleteRule
    // print_to_console("clear_css 1");
    let style_sheets: StyleSheetList = document.style_sheets();
    let lim_i = style_sheets.length();
    for i in 0..lim_i {
        // For each style sheet.  Forced unwrap OK because `i` is
        // confined to a range

        let style_sheet: StyleSheet = style_sheets.get(i).unwrap();

        let css_style_sheet =
            match wasm_bindgen::JsCast::dyn_into::<web_sys::CssStyleSheet>(style_sheet) {
                Ok(css) => css,
                Err(err) => {
                    print_to_console(format!("{err:?} Not a CssStyleSheet"));
                    continue;
                }
            };
        // Got a CssStyleSheet
        let css_rules: CssRuleList = css_style_sheet.css_rules()?;
        let lim_j = css_rules.length();
        for j in 0..lim_j {
            match css_style_sheet.delete_rule(j) {
                Ok(()) => (),
                Err(err) => print_to_console(format!(
                    "Cannot delete rule {j} of {lim_j}: {}:{}",
                    err.as_string().unwrap_or("<UNKNOWN>".to_string()),
                    err.js_typeof().as_string().unwrap_or("".to_string()),
                )),
            };
        }
    }
    let style_element = get_style_element(document)?;
    style_element.set_inner_html("");
    Ok(())
}
