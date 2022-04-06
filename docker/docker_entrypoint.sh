#!/usr/bin/env bash
set -euo pipefail

STATS_BASE_URL=${STATS_BASE_URL:-"http://127.0.0.1:8080"}

# Unpack frontend if applicable
FRONTEND_PACK="frontend.tar.gz"
if [ -f "$FRONTEND_PACK" ]; then
  tar -xzvf "$FRONTEND_PACK"
fi

pushd monitoring
RUST_LOG=warn trunk serve --release &
popd

RUST_LOG=warn backend
