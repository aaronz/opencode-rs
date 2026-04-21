use std::path::{Path, PathBuf};

pub struct Normalizer;

impl Normalizer {
    pub fn new() -> Self {
        Self
    }

    pub fn normalize(&self, path: &Path) -> PathBuf {
        let mut result = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    result.pop();
                }
                std::path::Component::CurDir => {}
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    result.push(component.as_os_str());
                }
                std::path::Component::Normal(s) => {
                    result.push(s);
                }
            }
        }
        result
    }

    pub fn resolve_path(&self, base: &Path, relative: &Path) -> PathBuf {
        if relative.is_absolute() {
            relative.to_path_buf()
        } else {
            self.normalize(&base.join(relative))
        }
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_collapse_dots() {
        let normalizer = Normalizer::new();
        let p = normalizer.normalize(Path::new("/a/b/../c/./d"));
        assert_eq!(p, Path::new("/a/c/d"));
    }

    #[test]
    fn test_normalize_resolve_relative() {
        let normalizer = Normalizer::new();
        let base = Path::new("/foo/bar");
        let relative = Path::new("../baz");
        let resolved = normalizer.resolve_path(base, relative);
        assert_eq!(resolved, Path::new("/foo/baz"));
    }

    #[test]
    fn test_resolve_path_already_absolute() {
        let normalizer = Normalizer::new();
        let base = Path::new("/foo/bar");
        let absolute = Path::new("/baz/qux");
        let resolved = normalizer.resolve_path(base, absolute);
        assert_eq!(resolved, Path::new("/baz/qux"));
    }
}