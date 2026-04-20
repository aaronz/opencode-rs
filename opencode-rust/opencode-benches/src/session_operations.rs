use criterion::{black_box, criterion_group, Criterion};
use opencode_core::{Message, Session};
use std::time::Duration;

pub fn session_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_operations");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("session_new", |b| {
        b.iter(|| {
            let session = Session::new();
            black_box(session)
        });
    });

    group.bench_function("session_with_messages", |b| {
        b.iter(|| {
            let mut session = Session::new();
            for i in 0..100 {
                session.add_message(Message::user(format!("Message {}", i)));
                session.add_message(Message::assistant(format!("Response {}", i)));
            }
            black_box(session)
        });
    });

    group.bench_function("session_save_load", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("session.json");

        b.iter(|| {
            let mut session = Session::new();
            for i in 0..50 {
                session.add_message(Message::user(format!("Message {}", i)));
                session.add_message(Message::assistant(format!("Response {}", i)));
            }
            session.save(&path).unwrap();
            let loaded = Session::load(&path).unwrap();
            black_box(loaded)
        });
    });

    group.bench_function("session_1000_messages", |b| {
        b.iter(|| {
            let mut session = Session::new();
            for i in 0..1000 {
                session.add_message(Message::user(format!("Message {}", i)));
            }
            black_box(session.messages.len())
        });
    });
}

criterion_group!(benches, session_create);