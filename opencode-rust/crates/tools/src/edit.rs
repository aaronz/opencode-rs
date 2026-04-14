#![allow(
    clippy::redundant_closure,
    clippy::needless_range_loop,
    clippy::let_underscore_future
)]

use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::path::PathBuf;

pub struct EditTool;

#[derive(Deserialize)]
struct EditArgs {
    #[serde(rename = "filePath")]
    file_path: String,
    #[serde(rename = "oldString")]
    old_string: String,
    #[serde(rename = "newString")]
    new_string: String,
    #[serde(rename = "replaceAll")]
    replace_all: Option<bool>,
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edit files with fuzzy string matching"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(EditTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: EditArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.file_path);

        if !path.exists() {
            return Ok(ToolResult::err(format!(
                "File not found: {}",
                args.file_path
            )));
        }

        let stats = std::fs::metadata(&path)?;
        if stats.is_dir() {
            return Ok(ToolResult::err(format!(
                "Path is a directory, not a file: {}",
                args.file_path
            )));
        }

        if args.old_string == args.new_string {
            return Ok(ToolResult::err(
                "No changes to apply: oldString and newString are identical.".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&path).map_err(|e| OpenCodeError::Io(e))?;

        // Detect line ending
        let ending = if content.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };

        // Normalize line endings in content and search strings
        let normalized_content = content.replace("\r\n", "\n");
        let normalized_old = args.old_string.replace("\r\n", "\n");
        let normalized_new = args.new_string.replace("\r\n", "\n");

        // Try to find and replace
        match replace(
            &normalized_content,
            &normalized_old,
            &normalized_new,
            args.replace_all.unwrap_or(false),
        ) {
            Ok(new_content) => {
                // Convert back to original line ending
                let final_content = if ending == "\r\n" {
                    new_content.replace("\n", "\r\n")
                } else {
                    new_content
                };

                std::fs::write(&path, &final_content).map_err(|e| OpenCodeError::Io(e))?;

                // Generate diff
                let diff = generate_diff(&args.file_path, &content, &final_content);

                let title = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&args.file_path)
                    .to_string();

                Ok(ToolResult::ok("Edit applied successfully.".to_string())
                    .with_title(title)
                    .with_metadata(serde_json::json!({
                        "diff": diff,
                    })))
            }
            Err(e) => Ok(ToolResult::err(e)),
        }
    }
}

fn replace(
    content: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> Result<String, String> {
    let mut not_found = true;

    // Try replacers in order
    for replacer in &[
        simple_replacer,
        line_trimmed_replacer,
        block_anchor_replacer,
        whitespace_normalized_replacer,
        indentation_flexible_replacer,
        trimmed_boundary_replacer,
    ] {
        for search in replacer(content, old_string) {
            let index = content.find(&search);
            if index.is_none() {
                continue;
            }
            not_found = false;

            if replace_all {
                return Ok(content.replace(&search, new_string));
            }

            let last_index = content.rfind(&search);
            if index != last_index {
                continue;
            }

            let idx = index.unwrap();
            let (start, end) = content.split_at(idx);
            return Ok(format!("{}{}{}", start, new_string, &end[search.len()..]));
        }
    }

    if not_found {
        Err("Could not find oldString in the file. It must match exactly, including whitespace, indentation, and line endings.".to_string())
    } else {
        Err("Found multiple matches for oldString. Provide more surrounding context to make the match unique.".to_string())
    }
}

// Replacer functions
#[allow(dead_code)]
type Replacer = fn(&str, &str) -> Vec<String>;

fn simple_replacer(content: &str, find: &str) -> Vec<String> {
    if content.contains(find) {
        vec![find.to_string()]
    } else {
        vec![]
    }
}

fn line_trimmed_replacer(content: &str, find: &str) -> Vec<String> {
    let mut results = Vec::new();
    let original_lines: Vec<&str> = content.lines().collect();
    let search_lines: Vec<&str> = find.lines().collect();

    let mut search_lines = search_lines.to_vec();
    if search_lines.last() == Some(&"") {
        search_lines.pop();
    }

    for i in 0..=original_lines.len().saturating_sub(search_lines.len()) {
        let mut matches = true;
        for j in 0..search_lines.len() {
            let original_trimmed = original_lines[i + j].trim();
            let search_trimmed = search_lines[j].trim();
            if original_trimmed != search_trimmed {
                matches = false;
                break;
            }
        }
        if matches {
            let match_start = original_lines[..i]
                .iter()
                .map(|l| l.len() + 1)
                .sum::<usize>();
            let mut match_end = match_start;
            for j in 0..search_lines.len() {
                match_end += original_lines[i + j].len();
                if j < search_lines.len() - 1 {
                    match_end += 1;
                }
            }
            results.push(content[match_start..match_end].to_string());
        }
    }
    results
}

fn block_anchor_replacer(content: &str, find: &str) -> Vec<String> {
    let mut results = Vec::new();
    let original_lines: Vec<&str> = content.lines().collect();
    let search_lines: Vec<&str> = find.lines().collect();

    if search_lines.len() < 3 {
        return results;
    }

    let mut search_lines = search_lines.to_vec();
    if search_lines.last() == Some(&"") {
        search_lines.pop();
    }

    let first_line_search = search_lines[0].trim();
    let last_line_search = search_lines[search_lines.len() - 1].trim();
    let search_block_size = search_lines.len();

    // Find candidates
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for i in 0..original_lines.len() {
        if original_lines[i].trim() != first_line_search {
            continue;
        }
        for j in i + 2..original_lines.len() {
            if original_lines[j].trim() == last_line_search {
                candidates.push((i, j));
                break;
            }
        }
    }

    if candidates.is_empty() {
        return results;
    }

    if candidates.len() == 1 {
        let (start_line, end_line) = candidates[0];
        let actual_block_size = end_line - start_line + 1;

        let mut similarity = 0.0;
        let lines_to_check = std::cmp::min(search_block_size - 2, actual_block_size - 2);

        if lines_to_check > 0 {
            for j in 1..search_block_size - 1 {
                if j >= actual_block_size - 1 {
                    break;
                }
                let original_line = original_lines[start_line + j].trim();
                let search_line = search_lines[j].trim();
                let max_len = std::cmp::max(original_line.len(), search_line.len());
                if max_len == 0 {
                    continue;
                }
                let distance = levenshtein(original_line, search_line);
                similarity += 1.0 - (distance as f64 / max_len as f64);
            }
            similarity /= lines_to_check as f64;
        } else {
            similarity = 1.0;
        }

        if similarity >= 0.0 {
            let mut match_start = 0usize;
            for k in 0..start_line {
                match_start += original_lines[k].len() + 1;
            }
            let mut match_end = match_start;
            for k in start_line..=end_line {
                match_end += original_lines[k].len();
                if k < end_line {
                    match_end += 1;
                }
            }
            results.push(content[match_start..match_end].to_string());
        }
        return results;
    }

    // Multiple candidates
    let mut best_match: Option<(usize, usize)> = None;
    let mut max_similarity = -1.0;

    for candidate in &candidates {
        let (start_line, end_line) = candidate;
        let actual_block_size = end_line - start_line + 1;

        let mut similarity = 0.0;
        let lines_to_check = std::cmp::min(search_block_size - 2, actual_block_size - 2);

        if lines_to_check > 0 {
            for j in 1..search_block_size - 1 {
                if j >= actual_block_size - 1 {
                    break;
                }
                let original_line = original_lines[start_line + j].trim();
                let search_line = search_lines[j].trim();
                let max_len = std::cmp::max(original_line.len(), search_line.len());
                if max_len == 0 {
                    continue;
                }
                let distance = levenshtein(original_line, search_line);
                similarity += 1.0 - (distance as f64 / max_len as f64);
            }
            similarity /= lines_to_check as f64;
        } else {
            similarity = 1.0;
        }

        if similarity > max_similarity {
            max_similarity = similarity;
            best_match = Some((*start_line, *end_line));
        }
    }

    if max_similarity >= 0.3 {
        if let Some((start_line, end_line)) = best_match {
            let mut match_start = 0usize;
            for k in 0..start_line {
                match_start += original_lines[k].len() + 1;
            }
            let mut match_end = match_start;
            for k in start_line..=end_line {
                match_end += original_lines[k].len();
                if k < end_line {
                    match_end += 1;
                }
            }
            results.push(content[match_start..match_end].to_string());
        }
    }

    results
}

fn whitespace_normalized_replacer(content: &str, find: &str) -> Vec<String> {
    let mut results = Vec::new();
    let normalize = |text: &str| {
        text.replace_whitespace()
            .collect::<String>()
            .trim()
            .to_string()
    };
    let normalized_find = normalize(find);

    for line in content.lines() {
        if normalize(line) == normalized_find {
            results.push(line.to_string());
        }
    }

    let lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = find.lines().collect();

    if find_lines.len() > 1 {
        for i in 0..=lines.len().saturating_sub(find_lines.len()) {
            let block = lines[i..i + find_lines.len()].join("\n");
            if normalize(&block) == normalized_find {
                results.push(block);
            }
        }
    }

    results
}

fn indentation_flexible_replacer(content: &str, find: &str) -> Vec<String> {
    let mut results = Vec::new();

    let remove_indentation = |text: &str| -> String {
        let lines: Vec<&str> = text.lines().collect();
        let non_empty: Vec<&str> = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .copied()
            .collect();
        if non_empty.is_empty() {
            return text.to_string();
        }
        let min_indent = non_empty
            .iter()
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        lines
            .iter()
            .map(|l| {
                if l.trim().is_empty() {
                    l.to_string()
                } else {
                    l[min_indent..].to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let normalized_find = remove_indentation(find);
    let content_lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = find.lines().collect();

    for i in 0..=content_lines.len().saturating_sub(find_lines.len()) {
        let block: String = content_lines[i..i + find_lines.len()].join("\n");
        if remove_indentation(&block) == normalized_find {
            results.push(block);
        }
    }

    results
}

fn trimmed_boundary_replacer(content: &str, find: &str) -> Vec<String> {
    let mut results = Vec::new();
    let trimmed_find = find.trim();

    if content.contains(trimmed_find) {
        results.push(trimmed_find.to_string());
    }

    // Block matching
    let lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = find.lines().collect();

    for i in 0..=lines.len().saturating_sub(find_lines.len()) {
        let block = lines[i..i + find_lines.len()].join("\n");
        if block.trim() == trimmed_find {
            results.push(block);
        }
    }

    results
}

fn levenshtein(a: &str, b: &str) -> usize {
    if a.is_empty() || b.is_empty() {
        return a.len().max(b.len());
    }

    let mut matrix: Vec<Vec<usize>> = Vec::with_capacity(a.len() + 1);
    for i in 0..=a.len() {
        matrix.push((0..=b.len()).map(|j| if j == 0 { i } else { 0 }).collect());
    }
    for j in 0..=b.len() {
        matrix[0][j] = j;
    }

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a.len()][b.len()]
}

fn generate_diff(file_path: &str, old_content: &str, new_content: &str) -> String {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let mut diff = String::new();
    diff.push_str(&format!("--- {}\n", file_path));
    diff.push_str(&format!("+++ {}\n", file_path));

    let max_len = std::cmp::max(old_lines.len(), new_lines.len());
    let mut changes = false;

    for i in 0..max_len {
        let old_line = old_lines.get(i).copied();
        let new_line = new_lines.get(i).copied();

        match (old_line, new_line) {
            (Some(ol), Some(nl)) if ol == nl => {
                diff.push_str(&format!(" {}\n", ol));
            }
            (Some(ol), Some(nl)) => {
                diff.push_str(&format!("-{}\n", ol));
                diff.push_str(&format!("+{}\n", nl));
                changes = true;
            }
            (Some(ol), None) => {
                diff.push_str(&format!("-{}\n", ol));
                changes = true;
            }
            (None, Some(nl)) => {
                diff.push_str(&format!("+{}\n", nl));
                changes = true;
            }
            (None, None) => {}
        }
    }

    if !changes && old_content != new_content {
        // Simple comparison
        return format!("@@ -1,{} +1,{} @@\n", old_lines.len(), new_lines.len());
    }

    diff
}

// Extension trait for whitespace normalization
trait WhitespaceExt {
    fn replace_whitespace(&self) -> ReplaceWhitespace<'_>;
}

impl WhitespaceExt for str {
    fn replace_whitespace(&self) -> ReplaceWhitespace<'_> {
        ReplaceWhitespace(self)
    }
}

struct ReplaceWhitespace<'a>(&'a str);

impl Iterator for ReplaceWhitespace<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.0.chars().next() {
            self.0 = &self.0[c.len_utf8()..];
            if c.is_whitespace() {
                return Some(' ');
            }
            return Some(c);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}
