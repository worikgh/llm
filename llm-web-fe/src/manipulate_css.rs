#[allow(unused_imports)]
use crate::utility::print_to_console;
#[allow(unused_imports)]
use crate::utility::print_to_console_s;
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
                    print_to_console_s(format!("{err:?} Not a CssStyleSheet"));
                    continue;
                }
            };
        // Got a CssStyleSheet
        let css_rules: CssRuleList = css_style_sheet.css_rules()?;
        for j in 0..css_rules.length() {
            // Forced unrwap OK because `j` is in a range
            let css_rule: CssRule = css_rules.get(j).unwrap();
            let css_style_rule =
                match wasm_bindgen::JsCast::dyn_into::<web_sys::CssStyleRule>(css_rule) {
                    Ok(c) => c,
                    Err(err) => {
                        print_to_console_s(format!("{err:?} Not a CssStyleRule"));
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
                        print_to_console_s(format!("{i}/{j}/{k} no style text"));
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

/// Add a style rule to the DOM.
/// Generic parameter `T` allows `value` to be `&str` or `String`
pub fn add_css_rule<T: Into<String>>(
    document: &Document,
    selector: &str,
    property: &str,
    value: T,
) -> Result<(), JsValue> {
    let value: String = value.into();
    // Check if the style element already contains CSS rules
    // print_to_console("add_css_rule 1");

    if let Some(rules) = get_css_rules(document)?.selector_rules.get(selector) {
        // The selector is registered
        if let Some(v) = rules.get(property) {
            // The property decralred for this rule
            if v == &value {
                // Rule already there
                return Ok(());
            } else {
                // Rule exists with a different value
                panic!(
                    "{}/{} has value: {}.  We want to set: {value}",
                    selector, property, v,
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
                    print_to_console_s(format!("{err:?} Not a CssStyleSheet"));
                    continue;
                }
            };
        // Got a CssStyleSheet
        let css_rules: CssRuleList = css_style_sheet.css_rules()?;
        let lim_j = css_rules.length();
        for j in 0..lim_j {
            // if css_rules.item(j).is_none() {
            //     print_to_console_s(format!("{i}/{lim_i}:{j}/{lim_j} Failed"));
            //     continue;
            // }
            css_style_sheet.delete_rule(j)?;
        }
    }

    let style_element = get_style_element(document)?;
    style_element.set_inner_html("");
    Ok(())
}
