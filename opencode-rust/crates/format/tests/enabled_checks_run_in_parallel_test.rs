use tempfile::tempdir;

use opencode_format::{all_formatters, FormatterContext};

#[tokio::test]
async fn enabled_checks_run_in_parallel() {
    let temp_dir = tempdir().unwrap();
    let ctx = FormatterContext {
        directory: temp_dir.path().to_path_buf(),
        worktree: temp_dir.path().to_path_buf(),
    };

    let formatters = all_formatters();

    let mut contexts = Vec::new();
    for _ in &formatters {
        contexts.push(FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        });
    }

    let start = std::time::Instant::now();

    let results: Vec<_> = match formatters.len() {
        26 => {
            let (r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17, r18, r19, r20, r21, r22, r23, r24, r25) = tokio::join!(
                formatters[0].enabled(&contexts[0]),
                formatters[1].enabled(&contexts[1]),
                formatters[2].enabled(&contexts[2]),
                formatters[3].enabled(&contexts[3]),
                formatters[4].enabled(&contexts[4]),
                formatters[5].enabled(&contexts[5]),
                formatters[6].enabled(&contexts[6]),
                formatters[7].enabled(&contexts[7]),
                formatters[8].enabled(&contexts[8]),
                formatters[9].enabled(&contexts[9]),
                formatters[10].enabled(&contexts[10]),
                formatters[11].enabled(&contexts[11]),
                formatters[12].enabled(&contexts[12]),
                formatters[13].enabled(&contexts[13]),
                formatters[14].enabled(&contexts[14]),
                formatters[15].enabled(&contexts[15]),
                formatters[16].enabled(&contexts[16]),
                formatters[17].enabled(&contexts[17]),
                formatters[18].enabled(&contexts[18]),
                formatters[19].enabled(&contexts[19]),
                formatters[20].enabled(&contexts[20]),
                formatters[21].enabled(&contexts[21]),
                formatters[22].enabled(&contexts[22]),
                formatters[23].enabled(&contexts[23]),
                formatters[24].enabled(&contexts[24]),
                formatters[25].enabled(&contexts[25]),
            );
            vec![r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, r13, r14, r15, r16, r17, r18, r19, r20, r21, r22, r23, r24, r25]
        }
        _ => panic!("Expected 26 formatters, got {}", formatters.len()),
    };

    let elapsed = start.elapsed().as_millis() as usize;

    assert!(
        elapsed < 2000,
        "tokio::join! should run checks in parallel (took {}ms, expected < 2000ms)",
        elapsed
    );

    let available_count = results.iter().filter(|r| r.is_some()).count();
    assert!(
        available_count > 0,
        "At least some formatters should be available"
    );
}