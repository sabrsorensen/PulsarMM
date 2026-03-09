#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"
source scripts/cargo-env.sh

exec cargo test --manifest-path src-tauri/Cargo.toml --all-targets "$@"
