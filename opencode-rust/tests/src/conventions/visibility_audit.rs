use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).parent().unwrap().to_path_buf()
}

fn get_crate_names(root: &PathBuf) -> Vec<(String, PathBuf)> {
    let crates_dir = root.join("crates");
    let mut crates = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("src/lib.rs").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    crates.push((name.to_string(), path));
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("src/lib.rs").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == "ratatui-testing" || name == "opencode-benches" {
                        crates.push((name.to_string(), path));
                    }
                }
            }
        }
    }

    crates
}

fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    let mut chars = name.chars().peekable();
    let first = chars.next().unwrap();
    if !first.is_lowercase() && first != '_' {
        return false;
    }
    for c in chars {
        if !c.is_lowercase() && !c.is_numeric() && c != '_' {
            return false;
        }
    }
    true
}

fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }

    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return true;
    }

    if !chars[0].is_uppercase() {
        return false;
    }

    for c in &chars[1..] {
        if *c == '_' {
            return false;
        }
    }

    true
}

fn is_screaming_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars().peekable();
    let first = chars.next().unwrap();
    if !first.is_uppercase() && first != '_' {
        return false;
    }
    for c in chars {
        if !c.is_uppercase() && !c.is_numeric() && c != '_' {
            return false;
        }
    }
    true
}

fn is_macro_line(line: &str) -> bool {
    line.contains("$name")
        || line.contains("$prefix")
        || line.contains("$vis")
        || line.contains("$id")
        || line.contains("$ty")
        || line.contains("$t")
}

fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if is_macro_line(trimmed) {
        return None;
    }

    let prefixes = [
        "pub(crate) fn ",
        "pub fn ",
        "pub(super) fn ",
        "pub(super) async fn ",
        "pub(crate) async fn ",
        "pub async fn ",
        "async fn ",
        "fn ",
    ];

    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| c == '<' || c == '(' || c == ' ')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn extract_struct_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if is_macro_line(trimmed) {
        return None;
    }

    let prefixes = ["pub(crate) struct ", "pub struct ", "pub(super) struct "];

    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| c == '<' || c == '(' || c == ' ' || c == ';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn extract_enum_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if is_macro_line(trimmed) {
        return None;
    }

    let prefixes = ["pub(crate) enum ", "pub enum ", "pub(super) enum "];

    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| c == '<' || c == '(' || c == ' ' || c == ';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn extract_trait_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if is_macro_line(trimmed) {
        return None;
    }

    let prefixes = ["pub(crate) trait ", "pub trait ", "pub(super) trait "];

    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| c == '<' || c == '(' || c == ' ' || c == ';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn extract_const_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if is_macro_line(trimmed) {
        return None;
    }

    let prefixes = ["pub const ", "pub(crate) const ", "pub(super) const "];

    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| c == ' ' || c == ':' || c == '=')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn find_files(src_dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                files.extend(find_files(&path));
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    files.push(path);
                }
            }
        }
    }

    files
}

#[test]
fn test_naming_conventions_functions_use_snake_case() {
    let root = workspace_root();
    let crates = get_crate_names(&root);
    let mut violations = Vec::new();

    for (crate_name, crate_path) in crates {
        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            continue;
        }

        for path in find_files(&src_dir) {
            if is_test_file(&path) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                if let Some(fn_name) = extract_function_name(line) {
                    if fn_name.starts_with("test_")
                        || fn_name.starts_with("bench_")
                        || fn_name.starts_with("impl_")
                    {
                        continue;
                    }
                    if fn_name.contains('<') || fn_name.contains('>') {
                        continue;
                    }
                    if !is_snake_case(&fn_name) && !fn_name.starts_with('_') {
                        violations.push(format!(
                            "{}:{}: function '{}' should use snake_case",
                            path.display(),
                            idx + 1,
                            fn_name
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = format!(
            "Found {} function naming violations:\n{}\n\
            Functions must use snake_case (e.g., 'fn get_session', not 'fn getSession' or 'fn GetSession')",
            violations.len(),
            violations.iter().take(50).cloned().collect::<Vec<_>>().join("\n")
        );
        panic!("{}", msg);
    }
}

#[test]
fn test_naming_conventions_types_use_camel_case() {
    let root = workspace_root();
    let crates = get_crate_names(&root);
    let mut violations = Vec::new();

    for (crate_name, crate_path) in crates {
        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            continue;
        }

        for path in find_files(&src_dir) {
            if is_test_file(&path) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                if let Some(struct_name) = extract_struct_name(line) {
                    if !struct_name.is_empty()
                        && !is_pascal_case(&struct_name)
                        && !struct_name.starts_with('_')
                    {
                        violations.push(format!(
                            "{}:{}: struct '{}' should use PascalCase",
                            path.display(),
                            idx + 1,
                            struct_name
                        ));
                    }
                }
                if let Some(enum_name) = extract_enum_name(line) {
                    if !enum_name.is_empty()
                        && !is_pascal_case(&enum_name)
                        && !enum_name.starts_with('_')
                    {
                        violations.push(format!(
                            "{}:{}: enum '{}' should use PascalCase",
                            path.display(),
                            idx + 1,
                            enum_name
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = format!(
            "Found {} type naming violations:\n{}\n\
            Types (structs, enums) must use PascalCase (e.g., 'struct SessionManager', not 'struct session_manager')",
            violations.len(),
            violations.iter().take(50).cloned().collect::<Vec<_>>().join("\n")
        );
        panic!("{}", msg);
    }
}

#[test]
fn test_naming_conventions_traits_use_camel_case() {
    let root = workspace_root();
    let crates = get_crate_names(&root);
    let mut violations = Vec::new();

    for (crate_name, crate_path) in crates {
        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            continue;
        }

        for path in find_files(&src_dir) {
            if is_test_file(&path) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                if let Some(trait_name) = extract_trait_name(line) {
                    if !trait_name.is_empty()
                        && !is_pascal_case(&trait_name)
                        && !trait_name.starts_with('_')
                    {
                        violations.push(format!(
                            "{}:{}: trait '{}' should use PascalCase",
                            path.display(),
                            idx + 1,
                            trait_name
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = format!(
            "Found {} trait naming violations:\n{}\n\
            Traits must use PascalCase (e.g., 'trait ToolExecutor', not 'trait tool_executor')",
            violations.len(),
            violations
                .iter()
                .take(50)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
        panic!("{}", msg);
    }
}

#[test]
fn test_constants_use_screaming_snake_case() {
    let root = workspace_root();
    let crates = get_crate_names(&root);
    let mut violations = Vec::new();

    for (crate_name, crate_path) in crates {
        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            continue;
        }

        for path in find_files(&src_dir) {
            if is_test_file(&path) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                if let Some(const_name) = extract_const_name(line) {
                    if !const_name.is_empty()
                        && !is_screaming_snake_case(&const_name)
                        && !const_name.starts_with('_')
                    {
                        violations.push(format!(
                            "{}:{}: constant '{}' should use SCREAMING_SNAKE_CASE",
                            path.display(),
                            idx + 1,
                            const_name
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = format!(
            "Found {} constant naming violations:\n{}\n\
            Constants must use SCREAMING_SNAKE_CASE (e.g., 'MAX_TOKEN_BUDGET', not 'maxTokenBudget')",
            violations.len(),
            violations.iter().take(50).cloned().collect::<Vec<_>>().join("\n")
        );
        panic!("{}", msg);
    }
}

fn is_test_file(path: &PathBuf) -> bool {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    filename.ends_with("_test.rs")
        || filename.ends_with("_tests.rs")
        || path.to_string_lossy().contains("/tests/")
        || path.to_string_lossy().contains("/test/")
}

#[test]
fn test_lib_rs_exports_are_appropriate() {
    let root = workspace_root();
    let crates = get_crate_names(&root);

    for (crate_name, crate_path) in crates {
        let lib_rs = crate_path.join("src/lib.rs");
        if !lib_rs.exists() {
            continue;
        }

        let content = fs::read_to_string(&lib_rs).unwrap_or_default();

        if content.contains("pub use") {
            let pub_use_count = content
                .lines()
                .filter(|l| l.trim().starts_with("pub use"))
                .count();
            let pub_mod_count = content
                .lines()
                .filter(|l| l.trim().starts_with("pub mod"))
                .count();

            assert!(
                pub_use_count > 0 || pub_mod_count > 0,
                "Crate '{}' has public exports but no 'pub use' or 'pub mod' statements",
                crate_name
            );
        }
    }
}

#[test]
fn test_no_private_items_in_public_api() {
    let root = workspace_root();
    let crates = get_crate_names(&root);
    let mut violations = Vec::new();

    for (crate_name, crate_path) in crates {
        let lib_rs = crate_path.join("src/lib.rs");
        if !lib_rs.exists() {
            continue;
        }

        let content = fs::read_to_string(&lib_rs).unwrap_or_default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub use ") {
                let rest = trimmed.trim_start_matches("pub use ");
                let parts: Vec<&str> = rest.split("::").collect();
                if let Some(item_name) = parts.last() {
                    let name = item_name.trim_end_matches(';').trim();
                    if name.starts_with('_') {
                        violations.push(format!(
                            "{}: public export '{}' starts with underscore (private)",
                            crate_name, name
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = format!(
            "Found {} private item in public API violations:\n{}\n",
            violations.len(),
            violations.join("\n")
        );
        panic!("{}", msg);
    }
}
