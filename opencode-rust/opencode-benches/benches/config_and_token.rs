use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opencode_core::config::Config;
use opencode_core::TokenCounter;
use serde_json::json;
use std::collections::HashMap;

fn config_parsing_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");
    group.measurement_time(std::time::Duration::from_secs(3));

    group.bench_function("config_parse_minimal", |b| {
        let json_str = r#"{"model": "gpt-4o"}"#;
        b.iter(|| {
            let config: Result<Config, _> = serde_json::from_str(json_str);
            black_box(config)
        });
    });

    group.bench_function("config_parse_with_permission", |b| {
        let json_str = r#"{
            "model": "gpt-4o",
            "permission": {
                "bash": "allow",
                "read": "allow",
                "edit": "deny"
            }
        }"#;
        b.iter(|| {
            let config: Result<Config, _> = serde_json::from_str(json_str);
            black_box(config)
        });
    });

    group.bench_function("config_parse_full", |b| {
        let json_str = r#"{
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
            let config: Result<Config, _> = serde_json::from_str(json_str);
            black_box(config)
        });
    });

    group.bench_function("config_parse_large_agent_section", |b| {
        let mut agent_config = HashMap::new();
        for i in 0..20 {
            let mut agent_json = HashMap::new();
            agent_json.insert("model", json!("gpt-4o"));
            agent_json.insert("temperature", json!(0.7));
            let mut tools = HashMap::new();
            tools.insert("bash", json!("allow"));
            tools.insert("read", json!("allow"));
            tools.insert("grep", json!("allow"));
            tools.insert("glob", json!("allow"));
            agent_json.insert("tools", json!(tools));
            agent_config.insert(format!("agent_{}", i), agent_json);
        }
        let config_json = json!({
            "model": "gpt-4o",
            "agent": agent_config
        });
        let json_str = serde_json::to_string(&config_json).unwrap();
        b.iter(|| {
            let config: Result<Config, _> = serde_json::from_str(&json_str);
            black_box(config)
        });
    });
}

fn token_counting_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_counting");
    group.measurement_time(std::time::Duration::from_secs(3));

    group.bench_function("token_count_empty_string", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        b.iter(|| {
            let count = counter.count_tokens("");
            black_box(count)
        });
    });

    group.bench_function("token_count_short_text", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        let text = "Hello, world!";
        b.iter(|| {
            let count = counter.count_tokens(text);
            black_box(count)
        });
    });

    group.bench_function("token_count_medium_text", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        let text = "The quick brown fox jumps over the lazy dog. This is a sample sentence for testing token counting performance.";
        b.iter(|| {
            let count = counter.count_tokens(text);
            black_box(count)
        });
    });

    group.bench_function("token_count_long_code_snippet", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        let code = r#"
fn main() {
    let mut vec = Vec::new();
    for i in 0..1000 {
        vec.push(i);
    }
    let sum: u64 = vec.iter().sum();
    println!("Sum: {}", sum);
}
"#;
        b.iter(|| {
            let count = counter.count_tokens(code);
            black_box(count)
        });
    });

    group.bench_function("token_count_messages_10", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        let messages: Vec<opencode_core::Message> = (0..10)
            .map(|i| {
                opencode_core::Message::user(format!("Message number {} with some content", i))
            })
            .collect();
        b.iter(|| {
            let count = counter.count_messages(&messages);
            black_box(count)
        });
    });

    group.bench_function("token_count_messages_100", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        let messages: Vec<opencode_core::Message> = (0..100)
            .map(|i| {
                opencode_core::Message::user(format!("Message number {} with some content", i))
            })
            .collect();
        b.iter(|| {
            let count = counter.count_messages(&messages);
            black_box(count)
        });
    });

    group.bench_function("token_count_messages_500", |b| {
        let counter = TokenCounter::for_model("gpt-4o");
        let messages: Vec<opencode_core::Message> = (0..500)
            .map(|i| {
                opencode_core::Message::user(format!("Message number {} with some content", i))
            })
            .collect();
        b.iter(|| {
            let count = counter.count_messages(&messages);
            black_box(count)
        });
    });
}

criterion_group!(benches, config_parsing_benches, token_counting_benches);
criterion_main!(benches);
