#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
META_FILE="${1:-$PROJECT_ROOT/packaging/pulsar.metainfo.xml}"

if [ ! -f "$META_FILE" ]; then
  echo "Metainfo file not found: $META_FILE" >&2
  exit 1
fi

echo "Validating AppStream metadata: $META_FILE"
appstreamcli validate --no-net --strict "$META_FILE"
