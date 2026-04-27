use std::fs;
use std::path::PathBuf;

#[test]
fn test_benchmark_regression_detection() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let benches_dir = manifest_dir.parent().unwrap().join("opencode-benches");
    let baseline_path = benches_dir
        .join("benches")
        .join("baseline")
        .join("baseline.json");

    if !baseline_path.exists() {
        eprintln!(
            "No baseline found at {:?}. Run 'cargo bench -- --update-baseline' to create baseline.",
            baseline_path
        );
        eprintln!("Skipping regression test.");
        return;
    }

    let baseline_content =
        fs::read_to_string(&baseline_path).expect("Failed to read baseline file");
    let baseline: serde_json::Value =
        serde_json::from_str(&baseline_content).expect("Failed to parse baseline JSON");

    let baseline_version = baseline
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    assert!(
        baseline.get("results").is_some(),
        "Baseline should contain 'results' field"
    );

    let results = baseline
        .get("results")
        .expect("Baseline should have results")
        .as_object()
        .expect("Results should be an object");

    assert!(
        !results.is_empty(),
        "Baseline should have benchmark results"
    );

    for (group_name, group_results) in results {
        let group_results = group_results
            .as_array()
            .expect("Group results should be an array");

        for result in group_results {
            assert!(
                result.get("name").is_some(),
                "Each result should have a 'name' field in group {}",
                group_name
            );
            assert!(
                result.get("mean_ns").is_some(),
                "Each result should have a 'mean_ns' field in group {}",
                group_name
            );
        }
    }

    println!(
        "Benchmark baseline validation passed. Version: {}, Groups: {}, Baseline path: {:?}",
        baseline_version,
        results.len(),
        baseline_path
    );
}

#[test]
fn test_baseline_file_structure() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let benches_dir = manifest_dir.parent().unwrap().join("opencode-benches");
    let baseline_path = benches_dir
        .join("benches")
        .join("baseline")
        .join("baseline.json");

    if !baseline_path.exists() {
        eprintln!("No baseline found at {:?}. Skipping test.", baseline_path);
        return;
    }

    let content = fs::read_to_string(&baseline_path).expect("Failed to read baseline");

    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Baseline should be valid JSON");

    assert!(
        parsed.get("version").is_some(),
        "Baseline needs version field"
    );
    assert!(
        parsed.get("results").is_some(),
        "Baseline needs results field"
    );
    assert!(
        parsed.get("created_at").is_some(),
        "Baseline needs created_at field"
    );
}

#[test]
fn test_criterion_benchmarks_exist() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let benches_dir = manifest_dir.parent().unwrap().join("opencode-benches");
    let benches_src_dir = benches_dir.join("benches");

    let expected_benchmarks = vec![
        "session_operations",
        "tool_execution",
        "storage_operations",
        "config_and_token",
        "plugin_operations",
        "jsonc_parsing",
    ];

    for bench_name in expected_benchmarks {
        let bench_rs_path = benches_src_dir.join(format!("{}.rs", bench_name));
        assert!(
            bench_rs_path.exists(),
            "Benchmark file should exist: {:?}",
            bench_rs_path
        );
    }
}

#[test]
fn test_benchmark_runner_binary_exists() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let benches_dir = manifest_dir.parent().unwrap().join("opencode-benches");
    let bin_path = benches_dir.join("src").join("bin").join("bench_runner.rs");

    assert!(
        bin_path.exists(),
        "Benchmark runner binary should exist at {:?}",
        bin_path
    );
}
