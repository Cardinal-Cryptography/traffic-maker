#!/usr/bin/env bash
set -euo pipefail

STATS_BASE_URL=${STATS_BASE_URL:-"http://127.0.0.1:8080"}

RUST_LOG=warn backend
