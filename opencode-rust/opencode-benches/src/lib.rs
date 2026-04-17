use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub group: String,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub median_ns: f64,
    pub min_ns: Option<f64>,
    pub max_ns: Option<f64>,
    pub samples: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBaseline {
    pub version: String,
    pub results: HashMap<String, Vec<BenchmarkResult>>,
    pub created_at: String,
}

impl BenchmarkBaseline {
    pub fn new() -> Self {
        Self {
            version: get_version(),
            results: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn load(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    pub fn add_result(&mut self, group: &str, result: BenchmarkResult) {
        self.results
            .entry(group.to_string())
            .or_default()
            .push(result);
    }

    pub fn compare(&self, other: &BenchmarkBaseline) -> Vec<RegressionReport> {
        let mut reports = Vec::new();

        for (group, baseline_results) in &self.results {
            if let Some(current_results) = other.results.get(group) {
                for baseline in baseline_results {
                    if let Some(current) = current_results.iter().find(|r| r.name == baseline.name)
                    {
                        if let Some(report) = baseline.detect_regression(current) {
                            reports.push(report);
                        }
                    }
                }
            }
        }

        reports
    }
}

impl BenchmarkResult {
    pub fn detect_regression(&self, current: &BenchmarkResult) -> Option<RegressionReport> {
        const REGRESSION_THRESHOLD_PERCENT: f64 = 10.0;

        let mean_diff_percent = ((current.mean_ns - self.mean_ns) / self.mean_ns) * 100.0;

        if mean_diff_percent > REGRESSION_THRESHOLD_PERCENT {
            Some(RegressionReport {
                group: self.group.clone(),
                name: self.name.clone(),
                baseline_mean_ns: self.mean_ns,
                current_mean_ns: current.mean_ns,
                regression_percent: mean_diff_percent,
                severity: if mean_diff_percent > 25.0 {
                    Severity::Major
                } else if mean_diff_percent > 15.0 {
                    Severity::Moderate
                } else {
                    Severity::Minor
                },
            })
        } else {
            None
        }
    }
}

impl Default for BenchmarkBaseline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    pub group: String,
    pub name: String,
    pub baseline_mean_ns: f64,
    pub current_mean_ns: f64,
    pub regression_percent: f64,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Minor,
    Moderate,
    Major,
}

fn get_version() -> String {
    option_env!("CARGO_PKG_VERSION")
        .unwrap_or("0.0.0")
        .to_string()
}

pub fn get_baseline_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("baseline")
        .join("baseline.json")
}
