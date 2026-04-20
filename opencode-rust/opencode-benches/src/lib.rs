use criterion::Criterion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

mod session_load;
mod session_operations;
mod storage_operations;
mod tool_execution;
mod config_and_token;
mod jsonc_parsing;
mod llm_roundtrip;
mod plugin_operations;

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

pub fn default_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::from_secs(5))
        .sample_size(100)
        .warm_up_time(std::time::Duration::from_secs(1))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub version: String,
    pub generated_at: String,
    pub total_benchmarks: usize,
    pub groups: Vec<BenchmarkGroupReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkGroupReport {
    pub name: String,
    pub benchmarks: Vec<BenchmarkSummary>,
    pub total_time_ns: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub name: String,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub p50_ns: f64,
    pub p95_ns: f64,
    pub p99_ns: f64,
}

impl BenchmarkReport {
    pub fn from_baseline(baseline: &BenchmarkBaseline) -> Self {
        let mut groups = Vec::new();
        let mut total_benchmarks = 0;

        for (group_name, results) in &baseline.results {
            let benchmarks: Vec<BenchmarkSummary> = results
                .iter()
                .map(|r| BenchmarkSummary {
                    name: r.name.clone(),
                    mean_ns: r.mean_ns,
                    std_dev_ns: r.std_dev_ns,
                    p50_ns: r.median_ns,
                    p95_ns: r.mean_ns * 1.96,
                    p99_ns: r.mean_ns * 2.576,
                })
                .collect();

            let total_time_ns: f64 = results.iter().map(|r| r.mean_ns).sum();

            total_benchmarks += results.len();
            groups.push(BenchmarkGroupReport {
                name: group_name.clone(),
                benchmarks,
                total_time_ns,
            });
        }

        Self {
            version: baseline.version.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            total_benchmarks,
            groups,
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_detect_regression() {
        let baseline = BenchmarkResult {
            name: "test_bench".to_string(),
            group: "test_group".to_string(),
            mean_ns: 1000.0,
            std_dev_ns: 50.0,
            median_ns: 980.0,
            min_ns: Some(900.0),
            max_ns: Some(1100.0),
            samples: 100,
        };

        let current = BenchmarkResult {
            name: "test_bench".to_string(),
            group: "test_group".to_string(),
            mean_ns: 1200.0,
            std_dev_ns: 60.0,
            median_ns: 1180.0,
            min_ns: Some(1000.0),
            max_ns: Some(1400.0),
            samples: 100,
        };

        let regression = baseline.detect_regression(&current);
        assert!(regression.is_some());
        let report = regression.unwrap();
        assert_eq!(report.regression_percent, 20.0);
        assert_eq!(report.severity, Severity::Moderate);
    }

    #[test]
    fn test_benchmark_result_no_regression() {
        let baseline = BenchmarkResult {
            name: "test_bench".to_string(),
            group: "test_group".to_string(),
            mean_ns: 1000.0,
            std_dev_ns: 50.0,
            median_ns: 980.0,
            min_ns: None,
            max_ns: None,
            samples: 100,
        };

        let current = BenchmarkResult {
            name: "test_bench".to_string(),
            group: "test_group".to_string(),
            mean_ns: 1050.0,
            std_dev_ns: 55.0,
            median_ns: 1030.0,
            min_ns: None,
            max_ns: None,
            samples: 100,
        };

        let regression = baseline.detect_regression(&current);
        assert!(regression.is_none());
    }

    #[test]
    fn test_benchmark_report_from_baseline() {
        let mut baseline = BenchmarkBaseline::new();
        baseline.add_result(
            "test_group",
            BenchmarkResult {
                name: "bench1".to_string(),
                group: "test_group".to_string(),
                mean_ns: 1000.0,
                std_dev_ns: 50.0,
                median_ns: 980.0,
                min_ns: None,
                max_ns: None,
                samples: 100,
            },
        );

        let report = BenchmarkReport::from_baseline(&baseline);
        assert_eq!(report.total_benchmarks, 1);
        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.groups[0].benchmarks.len(), 1);
    }

    #[test]
    fn test_baseline_compare() {
        let mut baseline = BenchmarkBaseline::new();
        baseline.add_result(
            "test_group",
            BenchmarkResult {
                name: "bench1".to_string(),
                group: "test_group".to_string(),
                mean_ns: 1000.0,
                std_dev_ns: 50.0,
                median_ns: 980.0,
                min_ns: None,
                max_ns: None,
                samples: 100,
            },
        );

        let mut current = BenchmarkBaseline::new();
        current.add_result(
            "test_group",
            BenchmarkResult {
                name: "bench1".to_string(),
                group: "test_group".to_string(),
                mean_ns: 1300.0,
                std_dev_ns: 60.0,
                median_ns: 1280.0,
                min_ns: None,
                max_ns: None,
                samples: 100,
            },
        );

        let regressions = baseline.compare(&current);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].regression_percent, 30.0);
    }
}
