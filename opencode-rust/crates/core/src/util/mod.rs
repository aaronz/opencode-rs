mod types;
pub use types::Util;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short() {
        assert_eq!(Util::truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(Util::truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_indent() {
        assert_eq!(Util::indent("a\nb", 2), "  a\n  b");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(Util::slugify("Hello World!"), "hello-world-");
    }
}