use criterion::{black_box, criterion_group, Criterion, BenchmarkId};
use opencode_llm::provider::{ChatMessage, Provider, StreamingCallback};
use opencode_llm::provider_abstraction::{DynProvider, ProviderSpec};
use opencode_llm::ProviderManager;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use opencode_llm::provider::{Model, sealed};

struct MockStreamingProvider {
    first_token_delay_ms: u64,
    chunk_delay_ms: u64,
    chunks: Vec<String>,
    model_name: String,
}

impl MockStreamingProvider {
    fn new(first_token_delay_ms: u64, chunk_delay_ms: u64, chunks: Vec<String>, model_name: &str) -> Self {
        Self {
            first_token_delay_ms,
            chunk_delay_ms,
            chunks,
            model_name: model_name.to_string(),
        }
    }
}

impl sealed::Sealed for MockStreamingProvider {}

#[async_trait]
impl Provider for MockStreamingProvider {
    async fn complete(
        &self,
        _prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        tokio::time::sleep(Duration::from_millis(self.first_token_delay_ms)).await;
        Ok(self.chunks.join(""))
    }

    async fn complete_streaming(
        &self,
        _prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        if self.first_token_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.first_token_delay_ms)).await;
        }

        for chunk in &self.chunks {
            callback(chunk.clone());
            if self.chunk_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.chunk_delay_ms)).await;
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![Model::new(&self.model_name, &self.model_name)]
    }

    fn provider_name(&self) -> &str {
        "mock_streaming"
    }
}

fn create_mock_streaming_provider(
    first_token_delay_ms: u64,
    chunk_delay_ms: u64,
    chunks: Vec<String>,
    model_name: &str,
) -> DynProvider {
    DynProvider::new(MockStreamingProvider::new(
        first_token_delay_ms,
        chunk_delay_ms,
        chunks,
        model_name,
    ))
}

pub fn bench_llm_first_token(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_roundtrip");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    let test_chunks = vec![
        "Hello".to_string(),
        ", ".to_string(),
        "how ".to_string(),
        "can ".to_string(),
        "I ".to_string(),
        "help ".to_string(),
        "you?".to_string(),
    ];

    let delays: Vec<u64> = vec![0, 10, 50, 100];

    for delay in delays {
        let delay_name = match delay {
            0 => "instant",
            10 => "10ms",
            50 => "50ms",
            100 => "100ms",
            _ => "unknown",
        };

        group.bench_with_input(
            BenchmarkId::new("first_token_latency", delay_name),
            &delay,
            |b, delay| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    let provider = create_mock_streaming_provider(
                        *delay,
                        5,
                        test_chunks.clone(),
                        "mock-model",
                    );

                    let start = Instant::now();

                    let callback: Box<dyn FnMut(String) + Send + Sync> =
                        Box::new(move |_chunk: String| {});

                    rt.block_on(provider.complete_streaming("test prompt", callback))
                        .unwrap();

                    let elapsed = start.elapsed();
                    black_box(elapsed);
                });
            },
        );
    }
}

pub fn bench_llm_full_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_roundtrip");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    let test_chunks = vec![
        "The ".to_string(),
        "quick ".to_string(),
        "brown ".to_string(),
        "fox ".to_string(),
        "jumps ".to_string(),
        "over ".to_string(),
        "the ".to_string(),
        "lazy ".to_string(),
        "dog.".to_string(),
    ];

    let delays: Vec<u64> = vec![1, 10, 50];

    for delay in delays {
        let delay_name = match delay {
            1 => "fast",
            10 => "10ms",
            50 => "50ms",
            _ => "unknown",
        };

        group.bench_with_input(
            BenchmarkId::new("full_response_time", delay_name),
            &delay,
            |b, delay| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    let provider = create_mock_streaming_provider(
                        20,
                        *delay,
                        test_chunks.clone(),
                        "mock-model",
                    );

                    let full_content = Arc::new(Mutex::new(String::new()));
                    let full_content_clone = full_content.clone();

                    let callback: Box<dyn FnMut(String) + Send + Sync> =
                        Box::new(move |chunk: String| {
                            if let Ok(mut content) = full_content_clone.lock() {
                                *content += &chunk;
                            }
                        });

                    rt.block_on(provider.complete_streaming("test prompt", callback))
                        .unwrap();

                    let content = full_content.lock().unwrap().clone();
                    black_box(content);
                });
            },
        );
    }
}

pub fn bench_provider_openai(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_roundtrip");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(50);

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("provider_openai_chat", |b| {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let provider = manager.create_provider(&spec).unwrap();
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
            },
        ];

        b.iter(|| {
            let result = rt.block_on(provider.chat(&messages));
            let _ = black_box(result);
        });
    });

    group.bench_function("provider_openai_complete", |b| {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let provider = manager.create_provider(&spec).unwrap();

        b.iter(|| {
            let result = rt.block_on(provider.complete("Say hello in one word", None));
            let _ = black_box(result);
        });
    });
}

pub fn bench_provider_anthropic(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_roundtrip");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(50);

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("provider_anthropic_chat", |b| {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::Anthropic {
            api_key: "test-key".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
        };

        let provider = manager.create_provider(&spec).unwrap();
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
            },
        ];

        b.iter(|| {
            let result = rt.block_on(provider.chat(&messages));
            let _ = black_box(result);
        });
    });

    group.bench_function("provider_anthropic_complete", |b| {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::Anthropic {
            api_key: "test-key".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
        };

        let provider = manager.create_provider(&spec).unwrap();

        b.iter(|| {
            let result = rt.block_on(provider.complete("Say hello in one word", None));
            let _ = black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_llm_first_token,
    bench_llm_full_response,
    bench_provider_openai,
    bench_provider_anthropic
);