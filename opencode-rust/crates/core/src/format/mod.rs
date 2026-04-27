mod types;

pub(crate) use types::FormatUtils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(FormatUtils::format_size(500), "500 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(FormatUtils::format_size(1024), "1.00 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(FormatUtils::format_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(FormatUtils::format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_size_zero() {
        assert_eq!(FormatUtils::format_size(0), "0 B");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(FormatUtils::format_duration(45), "45s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(FormatUtils::format_duration(90), "1m 30s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(FormatUtils::format_duration(3661), "1h 1m");
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(FormatUtils::format_duration(0), "0s");
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(FormatUtils::truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(FormatUtils::truncate("hello world", 5), "he...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(FormatUtils::truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_empty() {
        assert_eq!(FormatUtils::truncate("", 5), "");
    }

    #[test]
    fn test_indent() {
        assert_eq!(FormatUtils::indent("line1\nline2", 2), "  line1\n  line2");
    }

    #[test]
    fn test_indent_empty() {
        assert_eq!(FormatUtils::indent("", 4), "");
    }

    #[test]
    fn test_indent_single_line() {
        assert_eq!(FormatUtils::indent("hello", 4), "    hello");
    }

    #[test]
    fn test_code_block() {
        let result = FormatUtils::code_block("rust", "fn main() {}");
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_code_block_empty() {
        let result = FormatUtils::code_block("python", "");
        assert!(result.contains("```python"));
        assert!(result.contains("\n\n```"));
    }
}