#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"
source scripts/cargo-env.sh

if [ ! -x node_modules/.bin/tauri ]; then
    echo "Installing npm dependencies for Tauri build..."
    npm ci --prefer-offline --no-audit --no-fund
fi

export PATH="$REPO_ROOT/node_modules/.bin:$PATH"

echo "Building Tauri binary..."
npm run tauri build -- --no-bundle --config '{"bundle":{"createUpdaterArtifacts":false},"plugins":{"updater":{"active":false}}}'
