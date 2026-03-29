use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct DiffCell {
    pub x: u16,
    pub y: u16,
    pub expected: char,
    pub actual: char,
}

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub diffs: Vec<DiffCell>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub total_cells: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub similarity: f64,
}

pub struct BufferDiff;

impl BufferDiff {
    pub fn compare_content(exp: &[char], act: &[char], width: u16, height: u16) -> DiffResult {
        let mut diffs = Vec::new();
        let mut changed = 0;
        let mut unchanged = 0;

        for i in 0..(width as usize * height as usize) {
            let exp_char = exp.get(i).copied().unwrap_or(' ');
            let act_char = act.get(i).copied().unwrap_or(' ');

            if exp_char == act_char {
                unchanged += 1;
            } else {
                changed += 1;
                let x = (i % width as usize) as u16;
                let y = (i / width as usize) as u16;
                diffs.push(DiffCell {
                    x,
                    y,
                    expected: exp_char,
                    actual: act_char,
                });
            }
        }

        let total = (width * height) as usize;
        let similarity = if total > 0 {
            (unchanged as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        DiffResult {
            diffs,
            stats: DiffStats {
                total_cells: total,
                changed,
                unchanged,
                similarity,
            },
        }
    }

    pub fn format_inline(result: &DiffResult) -> String {
        let mut output = String::new();
        for diff in &result.diffs {
            writeln!(
                output,
                "({}:{}) '{}' -> '{}'",
                diff.x, diff.y, diff.expected, diff.actual
            )
            .unwrap();
        }
        if result.diffs.is_empty() {
            output.push_str("No differences found.\n");
        }
        output
    }

    pub fn has_differences(result: &DiffResult) -> bool {
        !result.diffs.is_empty()
    }

    pub fn similarity_percentage(result: &DiffResult) -> f64 {
        result.stats.similarity
    }
}
