#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

if [ -z "${PULSAR_FLATPAK_OUTPUT:-}" ]; then
    GIT_HASH="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
    BUILD_STAMP="$(date -u +%Y%m%d-%H%M%S)"
    export PULSAR_FLATPAK_OUTPUT="PulsarMM-${BUILD_STAMP}-${GIT_HASH}.flatpak"
fi
FLATPAK_OUTPUT="$PULSAR_FLATPAK_OUTPUT"
USER_HICOLOR_INDEX="$HOME/.local/share/flatpak/exports/share/icons/hicolor/index.theme"

bash packaging/flatpak/package-flatpak.sh

if [ -f "$USER_HICOLOR_INDEX" ] && [ ! -w "$USER_HICOLOR_INDEX" ]; then
    chmod u+w "$USER_HICOLOR_INDEX" || true
fi

flatpak install -y --user "$REPO_ROOT/$FLATPAK_OUTPUT"
exec flatpak run --user com.sabrsorensen.Pulsar
