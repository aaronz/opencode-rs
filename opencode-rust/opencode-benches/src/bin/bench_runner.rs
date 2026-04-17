use opencode_benches::{get_baseline_path, BenchmarkBaseline, BenchmarkResult};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--update-baseline" {
        update_baseline();
    } else if args.len() > 1 && args[1] == "--compare" {
        compare_with_baseline();
    } else if args.len() > 1 && args[1] == "--run-and-save" {
        run_and_save();
    } else {
        run_benchmarks();
    }
}

fn run_benchmarks() {
    let output = Command::new("cargo")
        .args(["bench", "--", "--noplot"])
        .current_dir(get_cargo_manifest_dir())
        .output()
        .expect("Failed to run cargo bench");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
}

fn run_and_save() {
    let results = run_cargo_bench();

    let mut baseline = BenchmarkBaseline::new();
    for (group_name, group_results) in results {
        for result in group_results {
            baseline.add_result(&group_name, result);
        }
    }

    let baseline_path = get_baseline_path();
    baseline
        .save(&baseline_path)
        .expect("Failed to save baseline");

    println!("Baseline saved to {:?}", baseline_path);
}

fn update_baseline() {
    run_and_save();
}

fn compare_with_baseline() {
    let results = run_cargo_bench();

    let mut current = BenchmarkBaseline::new();
    for (group_name, group_results) in results {
        for result in group_results {
            current.add_result(&group_name, result);
        }
    }

    let baseline_path = get_baseline_path();
    let baseline = BenchmarkBaseline::load(&baseline_path);

    match baseline {
        Some(baseline) => {
            let regressions = baseline.compare(&current);
            if !regressions.is_empty() {
                eprintln!("\n=== PERFORMANCE REGRESSION DETECTED ===\n");
                for report in &regressions {
                    eprintln!(
                        "[{:?}] {} > {}: {:.2}% regression (baseline: {:.2}ns, current: {:.2}ns)",
                        report.severity,
                        report.group,
                        report.name,
                        report.regression_percent,
                        report.baseline_mean_ns,
                        report.current_mean_ns
                    );
                }
                eprintln!("\n========================================\n");
                std::process::exit(1);
            } else {
                println!("No performance regressions detected.");
                std::process::exit(0);
            }
        }
        None => {
            eprintln!("No baseline found. Run with --update-baseline first.");
            std::process::exit(2);
        }
    }
}

fn run_cargo_bench() -> HashMap<String, Vec<BenchmarkResult>> {
    let output = Command::new("cargo")
        .args(["bench", "--", "--noplot"])
        .current_dir(get_cargo_manifest_dir())
        .output()
        .expect("Failed to run cargo bench");

    let stderr = String::from_utf8_lossy(&output.stderr);
    parse_benchmark_output(&stderr)
}

fn get_cargo_manifest_dir() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(manifest_dir).to_path_buf())
}

fn parse_benchmark_output(output: &str) -> HashMap<String, Vec<BenchmarkResult>> {
    let mut results = HashMap::new();
    let mut current_group = String::new();

    for line in output.lines() {
        if let Some(group_name) = extract_group_name(line) {
            current_group = group_name;
            results
                .entry(current_group.clone())
                .or_insert_with(Vec::new);
        } else if let Some(result) = parse_benchmark_line(line, &current_group) {
            results
                .entry(current_group.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
    }

    results
}

fn extract_group_name(line: &str) -> Option<String> {
    if line.contains("Running") || line.contains("Benchmark") {
        let start = line
            .find("session_operations")
            .or_else(|| line.find("tool_execution"))
            .or_else(|| line.find("storage_operations"))
            .or_else(|| line.find("config_and_token"))
            .or_else(|| line.find("plugin_operations"))
            .or_else(|| line.find("jsonc_parsing"));

        if let Some(idx) = start {
            let rest = &line[idx..];
            let end = rest
                .find(|c: char| c.is_whitespace())
                .or_else(|| rest.find(':'))
                .unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn parse_benchmark_line(line: &str, group: &str) -> Option<BenchmarkResult> {
    if !line.contains("time:") {
        return None;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts.first()?.split_whitespace().next()?.to_string();

    for (i, part) in parts.iter().enumerate() {
        if *part == "time:" && i + 1 < parts.len() {
            let time_str = parts[i + 1];
            if let Some(mean_ns) = parse_time_to_ns(time_str) {
                return Some(BenchmarkResult {
                    name,
                    group: group.to_string(),
                    mean_ns,
                    std_dev_ns: 0.0,
                    median_ns: mean_ns,
                    min_ns: None,
                    max_ns: None,
                    samples: 0,
                });
            }
        }
    }

    None
}

fn parse_time_to_ns(time_str: &str) -> Option<f64> {
    let time_str = time_str.trim_matches(|c: char| c.is_ascii_punctuation());

    if let Some(stripped) = time_str.strip_suffix("ns") {
        return stripped.parse().ok();
    } else if let Some(stripped) = time_str
        .strip_suffix("µs")
        .or_else(|| time_str.strip_suffix("us"))
    {
        let val: f64 = stripped.parse().ok()?;
        return Some(val * 1000.0);
    } else if let Some(stripped) = time_str.strip_suffix("ms") {
        let val: f64 = stripped.parse().ok()?;
        return Some(val * 1_000_000.0);
    } else if let Some(stripped) = time_str.strip_suffix("s") {
        let val: f64 = stripped.parse().ok()?;
        return Some(val * 1_000_000_000.0);
    }

    time_str.parse().ok()
}
