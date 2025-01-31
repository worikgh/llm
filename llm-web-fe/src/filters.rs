/// Change text so it can be displayed in a HTML element
/// `input` Plain text to convert
/// `result` Html version of `input`
pub fn text_for_html(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            ' ' => result.push_str("&nbsp;"),
            '\n' => result.push_str("<br/>"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),

            // TAB is a gross approximation.
            '\t' => result.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            _ => result.push(c),
        }
    }
    result
}
