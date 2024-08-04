// Utility functons for llm-web-fe

use wasm_bindgen::prelude::*;
use web_sys::console;
use chrono::Duration;

#[wasm_bindgen]
pub fn _print_to_console(message: &str) {
    console::log_1(&message.into());
}

pub fn print_to_console<T: Into<String>>(message: T) {
    let message = message.into();
    _print_to_console(message.as_str());
}

pub fn format_with_commas(n: i64) -> String {
    let str_num = n.to_string();
    let mut result = String::new();
    let int_len = str_num.len();
    
    for (i, c) in str_num.chars().enumerate() {
        if (int_len - i) % 3 == 0 && i != 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.num_seconds();
    
    if seconds >= 3 * 60 * 60  {
        let hours = (30*60 + seconds) / 3600;
        format!("{}h", hours)
    }else if seconds >= 3600 {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else if seconds >= 60 {
        let minutes = seconds / 60;
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}
