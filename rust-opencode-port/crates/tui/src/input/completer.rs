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

impl FileCompleter {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            max_suggestions: DEFAULT_MAX_SUGGESTIONS,
        }
    }

    pub fn suggest(&self, partial: &str) -> Vec<PathBuf> {
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
            if ignored_patterns.iter().any(|pattern| candidate == *pattern) {
                continue;
            }
            let score = if needle.is_empty() {
                Some(0)
            } else {
                matcher.fuzzy_match(&candidate, needle)
            };

            if let Some(score) = score {
                scored.push((score, candidate));
            }
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        scored
            .into_iter()
            .take(self.max_suggestions)
            .map(|(_, path)| PathBuf::from(path))
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
