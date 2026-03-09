#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

OUTPUT="${PULSAR_FLATPAK_OUTPUT:-PulsarMM.flatpak}"
PULSAR_PATCH_FOR_FLATPAK="${PULSAR_PATCH_FOR_FLATPAK:-1}"

if [ "${PULSAR_SKIP_TEST_GATE:-0}" != "1" ]; then
    echo "Running quality gate..."
    bash scripts/run-quality-gate.sh
else
    echo "Skipping quality gate because PULSAR_SKIP_TEST_GATE=1"
fi

echo "Cleaning previous build artifacts..."
rm -rf flatpak-build flatpak-repo .flatpak-builder flatpak-source

if [ -z "${PULSAR_BIN:-}" ]; then
    bash scripts/build-tauri-binary.sh
    PULSAR_BIN="$REPO_ROOT/src-tauri/target/release/Pulsar"
fi

PULSAR_BIN="$PULSAR_BIN" \
PULSAR_PATCH_FOR_FLATPAK="$PULSAR_PATCH_FOR_FLATPAK" \
bash packaging/flatpak/copy-resources.sh

FLATPAK_INSTALL_SCOPE="${FLATPAK_INSTALL_SCOPE:-user}" bash packaging/flatpak/ensure-runtimes.sh
PULSAR_FLATPAK_OUTPUT="$OUTPUT" bash packaging/flatpak/build-flatpak.sh
