use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedInput {
    FileReference { paths: Vec<String> },
    ShellCommand { cmd: String },
    SlashCommand { name: String, args: Vec<String> },
    PlainText { text: String },
}

#[derive(Debug, Clone, Default)]
pub struct InputParser;

impl InputParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Vec<ParsedInput> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return vec![ParsedInput::PlainText {
                text: input.to_string(),
            }];
        }

        if trimmed.starts_with('/') {
            return vec![self.parse_slash(trimmed)];
        }

        if let Some(shell) = self.extract_shell_command(input) {
            return vec![ParsedInput::ShellCommand { cmd: shell }];
        }

        let mut file_paths = Vec::new();
        let mut plain_tokens = Vec::new();

        for token in input.split_whitespace() {
            if let Some(path) = token.strip_prefix('@') {
                if !path.is_empty() {
                    file_paths.push(path.to_string());
                    continue;
                }
            }
            plain_tokens.push(token.to_string());
        }

        let mut parsed = Vec::new();
        if !file_paths.is_empty() {
            parsed.push(ParsedInput::FileReference { paths: file_paths });
        }

        let plain = plain_tokens.join(" ");
        if !plain.is_empty() || parsed.is_empty() {
            parsed.push(ParsedInput::PlainText { text: plain });
        }

        parsed
    }

    pub fn complete_at(&self, input: &str, cwd: &Path) -> Vec<String> {
        let Some(fragment) = current_at_fragment(input) else {
            return Vec::new();
        };

        let (base_dir, prefix) = split_completion_target(&fragment, cwd);
        let mut matches = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

                if !name.starts_with(&prefix) {
                    continue;
                }

                let candidate = if fragment.contains('/') {
                    let parent = fragment
                        .rsplit_once('/')
                        .map(|(parent, _)| parent)
                        .unwrap_or("");
                    if parent.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}/{}", parent, name)
                    }
                } else {
                    name.to_string()
                };

                if path.is_dir() {
                    matches.push(format!("{}/", candidate));
                } else {
                    matches.push(candidate);
                }
            }
        }

        matches.sort();
        matches
    }

    pub fn complete_slash(&self, input: &str, commands: &[&str]) -> Vec<String> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('/') {
            return Vec::new();
        }

        let prefix = trimmed
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase();

        let mut matches = commands
            .iter()
            .filter(|cmd| cmd.starts_with(&prefix))
            .map(|cmd| format!("/{}", cmd))
            .collect::<Vec<_>>();
        matches.sort();
        matches
    }

    fn parse_slash(&self, trimmed: &str) -> ParsedInput {
        let without = trimmed.trim_start_matches('/');
        let mut parts = without.split_whitespace();
        let name = parts.next().unwrap_or_default().to_lowercase();
        let args = parts.map(str::to_string).collect::<Vec<_>>();

        ParsedInput::SlashCommand { name, args }
    }

    fn extract_shell_command(&self, input: &str) -> Option<String> {
        let chars = input.char_indices().collect::<Vec<_>>();
        for (idx, ch) in chars {
            if ch != '!' {
                continue;
            }

            let is_start = idx == 0;
            let preceded_by_whitespace = !is_start
                && input[..idx]
                    .chars()
                    .last()
                    .map(|c| c.is_whitespace())
                    .unwrap_or(false);

            if is_start || preceded_by_whitespace {
                let cmd = input[idx + 1..].trim().to_string();
                return Some(cmd);
            }
        }
        None
    }
}

fn current_at_fragment(input: &str) -> Option<String> {
    let token = input.split_whitespace().last()?;
    token.strip_prefix('@').map(str::to_string)
}

fn split_completion_target(fragment: &str, cwd: &Path) -> (PathBuf, String) {
    if let Some((parent, prefix)) = fragment.rsplit_once('/') {
        let dir = if parent.is_empty() {
            cwd.to_path_buf()
        } else {
            cwd.join(parent)
        };
        (dir, prefix.to_string())
    } else {
        (cwd.to_path_buf(), fragment.to_string())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn parse_plain_text() {
        let parser = InputParser::new();
        assert_eq!(
            parser.parse("summarize this"),
            vec![ParsedInput::PlainText {
                text: "summarize this".to_string()
            }]
        );
    }

    #[test]
    fn parse_multiple_file_refs_with_text() {
        let parser = InputParser::new();
        assert_eq!(
            parser.parse("@file1 @src/lib.rs summarize this code"),
            vec![
                ParsedInput::FileReference {
                    paths: vec!["file1".to_string(), "src/lib.rs".to_string()]
                },
                ParsedInput::PlainText {
                    text: "summarize this code".to_string()
                }
            ]
        );
    }

    #[test]
    fn parse_shell_start_of_line() {
        let parser = InputParser::new();
        assert_eq!(
            parser.parse("!cargo test"),
            vec![ParsedInput::ShellCommand {
                cmd: "cargo test".to_string()
            }]
        );
    }

    #[test]
    fn parse_shell_after_whitespace() {
        let parser = InputParser::new();
        assert_eq!(
            parser.parse("please run !ls -la"),
            vec![ParsedInput::ShellCommand {
                cmd: "ls -la".to_string()
            }]
        );
    }

    #[test]
    fn parse_slash_command() {
        let parser = InputParser::new();
        assert_eq!(
            parser.parse("/models openai"),
            vec![ParsedInput::SlashCommand {
                name: "models".to_string(),
                args: vec!["openai".to_string()]
            }]
        );
    }

    #[test]
    fn complete_at_returns_matches() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();

        let parser = InputParser::new();
        let suggestions = parser.complete_at("@m", dir.path());
        assert!(suggestions.contains(&"main.rs".to_string()));
    }

    #[test]
    fn complete_slash_returns_matching_commands() {
        let parser = InputParser::new();
        let commands = ["help", "models", "agents", "clear"];
        let suggestions = parser.complete_slash("/mo", &commands);
        assert_eq!(suggestions, vec!["/models".to_string()]);
    }
}
