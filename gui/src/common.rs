use unicode_width::UnicodeWidthChar;

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }

    let truncated: String = s.chars().take(max_len).collect();

    format!("{}...", truncated)
}

pub fn truncate_string_unicode(s: &str, max_len: usize) -> String {
    let mut current_len = 0;
    let mut result = String::new();

    for c in s.chars() {
        let char_width = UnicodeWidthChar::width(c).unwrap_or(0);

        if current_len + char_width > max_len {
            result.push_str("...");
            break;
        }

        result.push(c);
        current_len += char_width;
    }

    result
}
