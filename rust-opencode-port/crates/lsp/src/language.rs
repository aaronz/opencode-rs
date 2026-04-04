use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Unknown,
}

impl Language {
    pub fn detect(path: &Path) -> Self {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "rs" => Language::Rust,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" => Language::JavaScript,
            "py" => Language::Python,
            "go" => Language::Go,
            _ => Language::Unknown,
        }
    }

    pub fn detect_from_root(root: &Path) -> Vec<Self> {
        let mut languages = Vec::new();

        if root.join("Cargo.toml").exists() {
            languages.push(Language::Rust);
        }
        if root.join("tsconfig.json").exists() {
            languages.push(Language::TypeScript);
        }
        if root.join("package.json").exists() {
            if !languages.contains(&Language::TypeScript) {
                languages.push(Language::JavaScript);
            }
        }
        if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            languages.push(Language::Python);
        }
        if root.join("go.mod").exists() {
            languages.push(Language::Go);
        }

        if languages.is_empty() {
            languages.push(Language::Unknown);
        }

        languages
    }

    pub fn server_command(&self) -> Option<&str> {
        match self {
            Language::Rust => Some("rust-analyzer"),
            Language::TypeScript | Language::JavaScript => {
                Some("typescript-language-server --stdio")
            }
            Language::Python => Some("pyright-langserver --stdio"),
            Language::Go => Some("gopls"),
            Language::Unknown => None,
        }
    }

    pub fn file_extensions(&self) -> &[&str] {
        match self {
            Language::Rust => &["rs"],
            Language::TypeScript => &["ts", "tsx"],
            Language::JavaScript => &["js", "jsx"],
            Language::Python => &["py"],
            Language::Go => &["go"],
            Language::Unknown => &[],
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn maps_required_extensions() {
        assert_eq!(Language::detect(Path::new("main.rs")), Language::Rust);
        assert_eq!(Language::detect(Path::new("app.ts")), Language::TypeScript);
        assert_eq!(Language::detect(Path::new("app.tsx")), Language::TypeScript);
        assert_eq!(Language::detect(Path::new("script.py")), Language::Python);
        assert_eq!(Language::detect(Path::new("main.go")), Language::Go);
    }

    #[test]
    fn uses_expected_server_commands() {
        assert_eq!(Language::Rust.server_command(), Some("rust-analyzer"));
        assert_eq!(
            Language::TypeScript.server_command(),
            Some("typescript-language-server --stdio")
        );
        assert_eq!(
            Language::Python.server_command(),
            Some("pyright-langserver --stdio")
        );
        assert_eq!(Language::Go.server_command(), Some("gopls"));
    }
}
