# opencode-rs Iteration Monitor

## Purpose

This document describes how the opencode-rs timed iteration monitor should work against the **current** PRD layout.

It is the human-readable companion to the OpenClaw cron payload. When the cron behavior changes, update both this file and the cron job payload.

---

## Current PRD Layout

The old `docs/PRD/modules/` flat layout is no longer the source of truth.

The repo now uses a restructured PRD tree:

```text
docs/PRD/
├── 00_OVERVIEW/
├── 10_CORE/
├── 20_INTEGRATION/
├── 30_INFRASTRUCTURE/
├── 40_USER_FACING/
├── 50_IMPLEMENTATION/
├── 90_REFERENCE/
└── 99_ARCHIVE/
```

Key references:

- `docs/PRD/00_OVERVIEW/02_document_index.md`
- `docs/README.md`
- `docs/PLANNING.md`
- active iteration artifacts under `iterations/iteration-*`

---

## Monitor Rules

### 1. Long task model

`iterate-prd.sh` is a long-running task.

Do not treat it as a short foreground command. Start it in background using a nohup-style launch and monitor it separately.

Canonical start forms:

```bash
nohup ./iterate-prd.sh --resume <N> > sessions/iteration-<N>_nohup_$(date +%Y%m%d_%H%M%S).log 2>&1 & echo $!
```

```bash
nohup ./iterate-prd.sh -p <PRD_PATH> > sessions/iteration-<name>_nohup_$(date +%Y%m%d_%H%M%S).log 2>&1 & echo $!
```

### 2. Completion rule

Do not use `verification-report.md` as the completion gate.

Completion is judged by the latest `tasks_vN.json` only:
- all tasks `done` => iteration complete
- any `todo` / `in_progress` / `manual_check` / other non-done states => iteration incomplete

### 3. Stuck rule

An iteration is considered stuck when:
- an `iterate-prd.sh` process is still running, and
- code/task/log files show no meaningful update for more than **20 minutes**

Check at least:
- `iterations/iteration-*/tasks_v*.json`
- latest iteration log under `sessions/`
- relevant source trees such as `opencode-rust/`, `docs/`, `plugins/`, `crates/`

If stuck:
1. kill the running iterate chain
2. resume the same iteration with nohup
3. report the restart to Aaron

### 4. Push rule

Do **not** skip push just because one attempt failed.

Required behavior:
- if there are pushable commits, keep trying to push
- if HTTPS/SSL/network fails, treat it as retryable, not skippable
- if push is rejected because remote moved, fetch/rebase/push
- only report a true blocker when human action is actually needed

### 5. Reporting rule

Every timed check should report in Chinese and must explicitly include:
- project name: `项目：opencode-rs`
- current iteration number
- done / not-done summary
- whether a process is running
- whether it is considered stuck
- whether a restart happened
- whether push happened
- what the next step is

---

## Current Observed Progress

Based on recent iteration history in the repo:

- iteration-44: file-related iteration, completed after repeated restarts and logic fixes
- iteration-45: project-related iteration, completed
- iteration-46: flag-related iteration, in progress during recent timed checks

This means the monitor should no longer assume the old flat `modules/ITERATION_ORDER.md` file exists.
Instead, it must derive the next PRD from the **current PRD hierarchy and real iteration history**.

---

## Next-PRD Selection Guidance

Until a new machine-readable order file is introduced for the restructured PRD tree, the monitor should:

1. inspect the current PRD index and planning docs
2. inspect recent completed iterations and map them to their PRD source
3. choose the next PRD in the intended sequence, not by stale hardcoded paths
4. prefer the current active sequence already visible in iteration history

In other words: use the current repo reality, not obsolete paths.

---

## Maintenance Note

If the PRD tree is restructured again, update this file first, then update the OpenClaw cron payload to match.
