#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn find_files_with_unwrap(root: &PathBuf) -> Vec<(PathBuf, Vec<String>)> {
        let mut results = Vec::new();
        let crates_dir = root.join("crates");

        if !crates_dir.exists() {
            return results;
        }

        for entry in std::fs::read_dir(crates_dir).unwrap() {
            let entry = entry.unwrap();
            let crate_path = entry.path();

            if !crate_path.is_dir() {
                continue;
            }

            let src_dir = crate_path.join("src");
            if !src_dir.exists() {
                continue;
            }

            find_unwrap_in_dir(&src_dir, &crate_path, &mut results);
        }

        results
    }

    fn find_unwrap_in_dir(
        dir: &PathBuf,
        crate_path: &PathBuf,
        results: &mut Vec<(PathBuf, Vec<String>)>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                find_unwrap_in_dir(&path, crate_path, results);
                continue;
            }

            let Some(ext) = path.extension() else {
                continue;
            };
            if ext != "rs" {
                continue;
            }

            if is_test_file(&path) {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut line_numbers = Vec::new();
            for (idx, line) in content.lines().enumerate() {
                if line.contains(".unwrap()") && !is_inside_cfg_test(&content, idx) {
                    line_numbers.push(format!("{}:{}", idx + 1, line.trim()));
                }
            }

            if !line_numbers.is_empty() {
                results.push((path, line_numbers));
            }
        }
    }

    fn is_test_file(path: &PathBuf) -> bool {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        filename.ends_with("_test.rs")
            || filename.ends_with("_tests.rs")
            || path.to_string_lossy().contains("/tests/")
            || path.to_string_lossy().contains("/test/")
    }

    fn is_inside_cfg_test(content: &str, line_idx: usize) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        if line_idx >= lines.len() {
            return false;
        }

        let mut in_cfg_test = false;
        let mut brace_count: isize = 0;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.contains("#[cfg(test)]") {
                in_cfg_test = true;
                brace_count = 0;
                continue;
            }

            if in_cfg_test {
                brace_count += line.chars().filter(|&c| c == '{').count() as isize;
                brace_count -= line.chars().filter(|&c| c == '}').count() as isize;

                if brace_count < 0 {
                    in_cfg_test = false;
                    continue;
                }

                if idx == line_idx {
                    return true;
                }
            }
        }

        false
    }

    fn categorize_risk(path: &PathBuf, line: &str) -> &'static str {
        let path_str = path.to_string_lossy();

        if path_str.contains("/routes/") || path_str.contains("/server/") {
            return "HIGH";
        }
        if path_str.contains("/agent/") || path_str.contains("/llm/") {
            return "HIGH";
        }
        if path_str.contains("/tools/") && (line.contains("execute") || line.contains("Tool")) {
            return "HIGH";
        }
        if path_str.contains("/auth/") || path_str.contains("/permission/") {
            return "HIGH";
        }
        if line.contains("expect(") {
            return "MEDIUM";
        }
        "LOW"
    }

    #[test]
    fn no_unwrap_in_production_code() {
        let root = workspace_root();
        let findings = find_files_with_unwrap(&root);

        if findings.is_empty() {
            return;
        }

        let mut high_risk = Vec::new();
        let mut medium_risk = Vec::new();
        let mut low_risk = Vec::new();

        for (path, lines) in &findings {
            for line in lines {
                let risk = categorize_risk(path, line);
                match risk {
                    "HIGH" => high_risk.push((path.clone(), line.clone())),
                    "MEDIUM" => medium_risk.push((path.clone(), line.clone())),
                    _ => low_risk.push((path.clone(), line.clone())),
                }
            }
        }

        let mut message = String::new();
        message.push_str(&format!(
            "Found {} files with .unwrap() in production code:\n\n",
            findings.len()
        ));

        if !high_risk.is_empty() {
            message.push_str(&format!(
                "=== HIGH RISK ({} occurrences) ===\n",
                high_risk.len()
            ));
            for (path, line) in &high_risk {
                message.push_str(&format!("  {}: {}\n", path.display(), line));
            }
            message.push('\n');
        }

        if !medium_risk.is_empty() {
            message.push_str(&format!(
                "=== MEDIUM RISK ({} occurrences) ===\n",
                medium_risk.len()
            ));
            for (path, line) in &medium_risk {
                message.push_str(&format!("  {}: {}\n", path.display(), line));
            }
            message.push('\n');
        }

        if !low_risk.is_empty() {
            message.push_str(&format!(
                "=== LOW RISK ({} occurrences) ===\n",
                low_risk.len()
            ));
            for (path, line) in &low_risk {
                message.push_str(&format!("  {}: {}\n", path.display(), line));
            }
            message.push('\n');
        }

        message.push_str("Production code must not use .unwrap() - use proper error propagation with '?' or expect with context");

        panic!("{}", message);
    }
}
