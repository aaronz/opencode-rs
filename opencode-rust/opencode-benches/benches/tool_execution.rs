use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opencode_tools::ToolRegistry;
use std::time::Duration;

fn tool_registry_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_registry");
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("registry_new", |b| {
        b.iter(|| {
            let registry = ToolRegistry::new();
            black_box(registry)
        });
    });

    group.bench_function("registry_is_disabled", |b| {
        let registry = ToolRegistry::new();
        b.iter(|| {
            let disabled = registry.is_disabled("read");
            black_box(disabled)
        });
    });
}

criterion_group!(benches, tool_registry_benches);
criterion_main!(benches);
