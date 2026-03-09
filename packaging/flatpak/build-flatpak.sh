#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

OUTPUT="${PULSAR_FLATPAK_OUTPUT:-PulsarMM.flatpak}"

echo "Running flatpak-builder..."
flatpak-builder --repo=flatpak-repo --force-clean --disable-updates --disable-rofiles-fuse flatpak-build packaging/flatpak/com.sabrsorensen.Pulsar.json
echo "Creating Flatpak bundle..."
flatpak build-bundle flatpak-repo "$OUTPUT" com.sabrsorensen.Pulsar
echo "Flatpak build and bundle complete. Output: $OUTPUT"
