use criterion::{black_box, criterion_group, Criterion};
use opencode_tools::glob::GlobTool;
use opencode_tools::grep_tool::GrepTool;
use opencode_tools::read::ReadTool;
use opencode_tools::write::WriteTool;
use opencode_tools::Tool;
use std::time::Duration;
use tempfile::TempDir;

pub fn bench_read_small_file(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("small.txt");
    std::fs::write(&file_path, "a".repeat(500)).unwrap();

    let tool = ReadTool::new();
    let path_str = file_path.to_str().unwrap().to_string();

    let mut group = c.benchmark_group("tool_execution");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("bench_read_small_file", |b| {
        b.iter(|| {
            let args = serde_json::json!({
                "path": path_str,
                "limit": 1000
            });
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tool.execute(args.clone(), None));
            black_box(result)
        });
    });
}

pub fn bench_read_large_file(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");
    let content = "line content here\n".repeat(20000);
    std::fs::write(&file_path, content).unwrap();

    let tool = ReadTool::new();
    let path_str = file_path.to_str().unwrap().to_string();

    let mut group = c.benchmark_group("tool_execution");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("bench_read_large_file", |b| {
        b.iter(|| {
            let args = serde_json::json!({
                "path": path_str,
                "limit": 2000
            });
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tool.execute(args.clone(), None));
            black_box(result)
        });
    });
}

pub fn bench_write_file(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let tool = WriteTool;
    let content = "a".repeat(1024);

    let mut group = c.benchmark_group("tool_execution");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("bench_write_file", |b| {
        b.iter(|| {
            let file_path = temp_dir.path().join("benchmark_write.txt");
            let args = serde_json::json!({
                "path": file_path.to_str().unwrap(),
                "content": content.clone()
            });
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tool.execute(args.clone(), None));
            black_box(result)
        });
    });
}

pub fn bench_grep(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("thousand_lines.txt");
    let content: String = (0..1000)
        .map(|i| format!("line {} with some searchable content\n", i))
        .collect();
    std::fs::write(&file_path, &content).unwrap();

    let tool = GrepTool;
    let path_str = temp_dir.path().to_str().unwrap().to_string();

    let mut group = c.benchmark_group("tool_execution");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("bench_grep", |b| {
        b.iter(|| {
            let args = serde_json::json!({
                "pattern": "searchable",
                "path": path_str
            });
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tool.execute(args.clone(), None));
            black_box(result)
        });
    });
}

pub fn bench_glob(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        std::fs::write(&file_path, format!("content {}", i)).unwrap();
    }

    let tool = GlobTool;
    let path_str = temp_dir.path().to_str().unwrap().to_string();

    let mut group = c.benchmark_group("tool_execution");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("bench_glob", |b| {
        b.iter(|| {
            let args = serde_json::json!({
                "pattern": "*.txt",
                "path": path_str
            });
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tool.execute(args.clone(), None));
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_read_small_file,
    bench_read_large_file,
    bench_write_file,
    bench_grep,
    bench_glob
);