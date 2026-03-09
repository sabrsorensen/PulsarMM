#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

if [ ! -x node_modules/.bin/vite ]; then
    echo "Installing npm dependencies..."
    npm ci --prefer-offline --no-audit --no-fund
fi

npm run test:coverage
npm run test:rust:coverage
