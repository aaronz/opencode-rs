use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opencode_core::{Message, Session};
use std::time::Duration;

fn storage_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("storage_create_small_session", |b| {
        b.iter(|| {
            let mut session = Session::new();
            for i in 0..10 {
                session.add_message(Message::user(format!("Message {}", i)));
                session.add_message(Message::assistant(format!("Response {}", i)));
            }
            black_box(session)
        });
    });

    group.bench_function("storage_create_medium_session", |b| {
        b.iter(|| {
            let mut session = Session::new();
            for i in 0..100 {
                session.add_message(Message::user(format!("Message {}", i)));
                session.add_message(Message::assistant(format!("Response {}", i)));
            }
            black_box(session)
        });
    });

    group.bench_function("storage_create_large_session", |b| {
        b.iter(|| {
            let mut session = Session::new();
            for i in 0..500 {
                session.add_message(Message::user(format!("Message {}", i)));
                session.add_message(Message::assistant(format!("Response {}", i)));
            }
            black_box(session)
        });
    });

    group.bench_function("session_message_iteration", |b| {
        let session = {
            let mut session = Session::new();
            for i in 0..1000 {
                session.add_message(Message::user(format!("Message {}", i)));
            }
            session
        };
        b.iter(|| {
            let count = session
                .messages
                .iter()
                .filter(|m| m.content.len() > 0)
                .count();
            black_box(count)
        });
    });

    group.bench_function("session_message_rev_iteration", |b| {
        let session = {
            let mut session = Session::new();
            for i in 0..1000 {
                session.add_message(Message::user(format!("Message {}", i)));
            }
            session
        };
        b.iter(|| {
            let count = session
                .messages
                .iter()
                .rev()
                .filter(|m| m.content.len() > 0)
                .count();
            black_box(count)
        });
    });
}

criterion_group!(benches, storage_benches);
criterion_main!(benches);
