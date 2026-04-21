#!/bin/bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$REPO_DIR/sessions"
mkdir -p "$LOG_DIR"

TS="$(date '+%Y%m%d_%H%M%S')"
RUN_LOG="$LOG_DIR/iterate-monitor_${TS}.log"

exec > >(tee -a "$RUN_LOG") 2>&1

echo "[$(date '+%Y-%m-%d %H:%M:%S')] iterate-monitor start"
echo "repo: $REPO_DIR"

cd "$REPO_DIR"

if pgrep -f "$REPO_DIR/iterate-prd.sh" >/dev/null 2>&1; then
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] iterate-prd.sh already running, skip"
  exit 0
fi

LATEST_ITERATION="$(find "$REPO_DIR/iterations" -maxdepth 1 -type d -name 'iteration-*' | sed 's#.*/iteration-##' | sort -n | tail -1)"
if [[ -z "${LATEST_ITERATION:-}" ]]; then
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] no iterations found, start fresh"
  exec "$REPO_DIR/iterate-prd.sh" --verbose --log "$LOG_DIR/iteration-auto_${TS}.log"
fi

LATEST_DIR="$REPO_DIR/iterations/iteration-$LATEST_ITERATION"
TASKS_JSON="$LATEST_DIR/tasks_v${LATEST_ITERATION}.json"
VERIFICATION_REPORT="$LATEST_DIR/verification-report.md"

if [[ ! -f "$TASKS_JSON" ]]; then
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] latest iteration missing tasks json, resume iteration-$LATEST_ITERATION"
  exec "$REPO_DIR/iterate-prd.sh" --resume "$LATEST_ITERATION" --verbose --log "$LOG_DIR/iteration-${LATEST_ITERATION}_resume_${TS}.log"
fi

if [[ ! -f "$VERIFICATION_REPORT" ]]; then
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] verification report missing, resume iteration-$LATEST_ITERATION"
  exec "$REPO_DIR/iterate-prd.sh" --resume "$LATEST_ITERATION" --verbose --log "$LOG_DIR/iteration-${LATEST_ITERATION}_resume_${TS}.log"
fi

if grep -Eq '^- \[ \]' "$TASKS_JSON" 2>/dev/null; then
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] unchecked markdown-style tasks found, resume iteration-$LATEST_ITERATION"
  exec "$REPO_DIR/iterate-prd.sh" --resume "$LATEST_ITERATION" --verbose --log "$LOG_DIR/iteration-${LATEST_ITERATION}_resume_${TS}.log"
fi

echo "[$(date '+%Y-%m-%d %H:%M:%S')] latest iteration-$LATEST_ITERATION looks complete, no action"
