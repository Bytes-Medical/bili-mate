#!/usr/bin/env bash
# Local/CI load run for the PRD-028 objective. Builds release binaries,
# starts the API with test-environment rate limits (spec 09: controlled
# bypass of per-IP throttling), runs bili-load and reports.
#
# Usage: scripts/load-test.sh [duration-seconds] [rps]
set -euo pipefail

DURATION="${1:-60}"
RPS="${2:-100}"
PORT=18190

cd "$(dirname "$0")/.."
cargo build --release -p bili-mate-api -p bili-mate-loadtest

BILI_MATE_BIND="127.0.0.1:${PORT}" \
BILI_MATE_RATE_LIMIT_PER_MINUTE=1000000 \
BILI_MATE_RATE_LIMIT_BURST=1000000 \
RUST_LOG=warn \
./target/release/bili-mate-api &
API_PID=$!
trap 'kill "$API_PID" 2>/dev/null || true' EXIT

for _ in $(seq 1 50); do
  if curl -so /dev/null "http://127.0.0.1:${PORT}/health/ready"; then break; fi
  sleep 0.2
done

./target/release/bili-load --url "http://127.0.0.1:${PORT}" --rps "$RPS" --duration "$DURATION"
