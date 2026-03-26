pub struct Util;

impl Util {
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    pub fn indent(s: &str, spaces: usize) -> String {
        let indent = " ".repeat(spaces);
        s.lines()
            .map(|l| format!("{}{}", indent, l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn slugify(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect()
    }
}
