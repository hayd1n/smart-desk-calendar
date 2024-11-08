pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }

    let truncated: String = s.chars().take(max_len).collect();

    format!("{}...", truncated)
}
