use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonc_parser::parse_to_serde_value;
use serde_json::Value;

fn jsonc_parsing_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsonc_parsing");
    group.measurement_time(std::time::Duration::from_secs(5));

    let minimal_jsonc = r#"{
    // Comment
    "name": "test",
    "enabled": true
}"#;

    let full_config_jsonc = r#"{
    // Server configuration
    "server": {
        "port": 3000,
        "hostname": "127.0.0.1"
    },
    /* Model configuration */
    "model": "gpt-4o",
    "temperature": 0.7,
    "max_tokens": 4096,
    // Permissions
    "permission": {
        "bash": "allow",
        "read": "allow",
        "write": "ask",
        "edit": "deny",
        "grep": "allow",
        "glob": "allow",
        "lsp": "allow"
    },
    // Agent configs
    "agent": {
        "plan": {
            "model": "claude-3-5-sonnet",
            "temperature": 0.5
        },
        "build": {
            "model": "gpt-4o",
            "temperature": 0.3
        }
    }
}"#;

    let large_config_jsonc = format!(
        r#"{{
    "model": "gpt-4o",
    "agents": {{
{}
    }}
}}"#,
        (0..50)
            .map(|i| format!(
                r#"        "agent_{0}": {{
            "model": "gpt-4o",
            "temperature": 0.7,
            "permission": {{
                "bash": "allow",
                "read": "allow",
                "write": "ask"
            }}
        }}{}"#,
                if i < 49 { "," } else { "" }
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );

    group.bench_function("parse_minimal_jsonc_current_json5", |b| {
        b.iter(|| {
            let result = opencode_core::config::parse_jsonc(minimal_jsonc);
            black_box(result)
        });
    });

    group.bench_function("parse_minimal_jsonc_jsonc_parser", |b| {
        b.iter(|| {
            let result: Result<Value, _> = parse_to_serde_value(minimal_jsonc, &Default::default());
            black_box(result)
        });
    });

    group.bench_function("parse_full_config_jsonc_current_json5", |b| {
        b.iter(|| {
            let result = opencode_core::config::parse_jsonc(full_config_jsonc);
            black_box(result)
        });
    });

    group.bench_function("parse_full_config_jsonc_jsonc_parser", |b| {
        b.iter(|| {
            let result: Result<Value, _> =
                parse_to_serde_value(full_config_jsonc, &Default::default());
            black_box(result)
        });
    });

    group.bench_function("parse_large_config_jsonc_current_json5", |b| {
        b.iter(|| {
            let result = opencode_core::config::parse_jsonc(&large_config_jsonc);
            black_box(result)
        });
    });

    group.bench_function("parse_large_config_jsonc_jsonc_parser", |b| {
        b.iter(|| {
            let result: Result<Value, _> =
                parse_to_serde_value(&large_config_jsonc, &Default::default());
            black_box(result)
        });
    });

    group.bench_function("parse_minimal_json_stripped_serde", |b| {
        let stripped = r#"{
    "name": "test",
    "enabled": true
}"#;
        b.iter(|| {
            let result = serde_json::from_str::<Value>(stripped);
            black_box(result)
        });
    });

    group.bench_function("parse_full_config_json_stripped_serde", |b| {
        let stripped = r#"{
    "server": {
        "port": 3000,
        "hostname": "127.0.0.1"
    },
    "model": "gpt-4o",
    "temperature": 0.7,
    "max_tokens": 4096,
    "permission": {
        "bash": "allow",
        "read": "allow",
        "write": "ask",
        "edit": "deny",
        "grep": "allow",
        "glob": "allow",
        "lsp": "allow"
    },
    "agent": {
        "plan": {
            "model": "claude-3-5-sonnet",
            "temperature": 0.5
        },
        "build": {
            "model": "gpt-4o",
            "temperature": 0.3
        }
    }
}"#;
        b.iter(|| {
            let result = serde_json::from_str::<Value>(stripped);
            black_box(result)
        });
    });
}

criterion_group!(benches, jsonc_parsing_benches);
criterion_main!(benches);
