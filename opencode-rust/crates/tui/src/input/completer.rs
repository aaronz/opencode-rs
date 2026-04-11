use std::path::{Path, PathBuf};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ignore::WalkBuilder;

use crate::command::CommandRegistry;

const DEFAULT_MAX_SUGGESTIONS: usize = 20;

#[derive(Debug, Clone)]
pub struct FileCompleter {
    root: PathBuf,
    max_suggestions: usize,
}

#[derive(Debug, Clone)]
pub struct FileSuggestion {
    pub path: PathBuf,
    pub icon: String,
}

fn get_file_icon(path: &Path) -> String {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match extension {
        "rs" | "go" | "ts" | "tsx" | "js" | "jsx" | "py" | "rb" | "java" | "c" | "cpp" | "h" => {
            "📄".to_string()
        }
        "md" | "txt" | "doc" | "pdf" => "📝".to_string(),
        "json" | "toml" | "yaml" | "yml" | "xml" | "config" => "⚙️".to_string(),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" => "🖼️".to_string(),
        "zip" | "tar" | "gz" | "rar" | "7z" => "📦".to_string(),
        "sh" | "bash" | "zsh" | "fish" => "🔧".to_string(),
        "html" | "css" | "scss" | "sass" => "🌐".to_string(),
        _ => "📄".to_string(),
    }
}

fn calculate_enhanced_score(candidate: &str, needle: &str, base_score: i64) -> i64 {
    if needle.is_empty() {
        return base_score;
    }

    let mut score = base_score;
    let needle_lower = needle.to_lowercase();
    let candidate_lower = candidate.to_lowercase();

    if let Some(filename) = candidate.rsplit('/').next() {
        let filename_lower = filename.to_lowercase();
        if filename.starts_with(needle) {
            score += 1000;
        } else if let Some(first_char) = needle.chars().next() {
            if filename_lower.starts_with(&first_char.to_string()) {
                for (i, _) in filename_lower.char_indices() {
                    if filename_lower[i..].starts_with(&needle_lower) {
                        score += 500;
                        break;
                    }
                }
            }
        }

        let needle_chars: Vec<char> = needle_lower.chars().collect();
        let mut match_count = 0;
        let mut last_match_pos = 0;
        for (i, c) in filename_lower.char_indices() {
            if match_count < needle_chars.len() && c == needle_chars[match_count] {
                if match_count == 0 || i == last_match_pos + 1 {
                    match_count += 1;
                    last_match_pos = i;
                }
            }
        }
        if match_count == needle_chars.len() {
            score += 200;
        }
    }

    let depth = candidate.chars().filter(|&c| c == '/').count() as i64;
    score += depth * -5;

    if let Some(pos) = candidate_lower.find(&needle_lower) {
        score += (50 - pos as i64).max(0) / 5;
    }

    score
}

impl FileCompleter {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            max_suggestions: DEFAULT_MAX_SUGGESTIONS,
        }
    }

    pub fn suggest(&self, partial: &str) -> Vec<PathBuf> {
        self.suggest_with_icons(partial)
            .into_iter()
            .map(|s| s.path)
            .collect()
    }

    pub fn suggest_with_icons(&self, partial: &str) -> Vec<FileSuggestion> {
        let matcher = SkimMatcherV2::default();
        let needle = partial.trim_start_matches('@');
        let ignored_patterns = load_root_gitignore_patterns(&self.root);

        let mut scored = Vec::new();
        for entry in WalkBuilder::new(&self.root)
            .standard_filters(true)
            .hidden(false)
            .build()
            .flatten()
        {
            if !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
            {
                continue;
            }

            let Ok(relative) = entry.path().strip_prefix(&self.root) else {
                continue;
            };
            let candidate = relative.to_string_lossy().replace('\\', "/");
            if ignored_patterns.contains(&candidate) {
                continue;
            }
            let score = if needle.is_empty() {
                Some(0)
            } else {
                matcher.fuzzy_match(&candidate, needle)
            };

            if let Some(base_score) = score {
                let enhanced_score = calculate_enhanced_score(&candidate, needle, base_score);
                scored.push((enhanced_score, candidate, entry.path().to_path_buf()));
            }
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.len().cmp(&b.1.len())));
        scored
            .into_iter()
            .take(self.max_suggestions)
            .map(|(_, path, full_path)| FileSuggestion {
                path: PathBuf::from(path),
                icon: get_file_icon(&full_path),
            })
            .collect()
    }
}

fn load_root_gitignore_patterns(root: &Path) -> Vec<String> {
    let gitignore = root.join(".gitignore");
    let Ok(content) = std::fs::read_to_string(gitignore) else {
        return Vec::new();
    };

    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone)]
pub struct CommandCompleter {
    commands: Vec<String>,
}

impl CommandCompleter {
    pub fn from_registry(registry: &CommandRegistry) -> Self {
        let mut commands = registry
            .all()
            .iter()
            .map(|command| command.name.clone())
            .collect::<Vec<_>>();
        commands.sort();
        commands.dedup();
        Self { commands }
    }

    pub fn suggest(&self, partial: &str) -> Vec<String> {
        let prefix = partial.trim_start_matches('/').to_lowercase();
        let mut matches = self
            .commands
            .iter()
            .filter(|command| command.starts_with(&prefix))
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        matches
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn file_completer_fuzzy_matches_paths() {
        let temp = tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join("src")).expect("create src");
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").expect("write main");
        fs::write(temp.path().join("src/parser.rs"), "").expect("write parser");

        let completer = FileCompleter::new(temp.path());
        let suggestions = completer.suggest("main");
        assert!(suggestions
            .iter()
            .any(|path| path.to_string_lossy().contains("src/main.rs")));
    }

    #[test]
    fn file_completer_respects_gitignore() {
        let temp = tempdir().expect("tempdir");
        fs::write(temp.path().join(".gitignore"), "ignored.txt\n").expect("write gitignore");
        fs::write(temp.path().join("ignored.txt"), "nope").expect("write ignored");
        fs::write(temp.path().join("visible.txt"), "ok").expect("write visible");

        let completer = FileCompleter::new(temp.path());
        let suggestions = completer.suggest("txt");
        assert!(suggestions
            .iter()
            .all(|path| path.to_string_lossy() != "ignored.txt"));
        assert!(suggestions
            .iter()
            .any(|path| path.to_string_lossy() == "visible.txt"));
    }

    #[test]
    fn command_completer_partial_match() {
        let registry = CommandRegistry::new();
        let completer = CommandCompleter::from_registry(&registry);
        let suggestions = completer.suggest("mod");
        assert_eq!(suggestions, vec!["models".to_string()]);
    }
}
