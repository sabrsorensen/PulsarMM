#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

APPIMAGE_OUTPUT="${PULSAR_APPIMAGE_OUTPUT:-PulsarMM-Linux.AppImage}"

bash packaging/appimage/build-appimage.sh

chmod +x "$APPIMAGE_OUTPUT"

# Prefer native FUSE execution when available, otherwise fall back to extract-and-run.
if [ -e /dev/fuse ] && { ldconfig -p 2>/dev/null | grep -q "libfuse.so.2" || [ -e /usr/lib/libfuse.so.2 ] || [ -e /lib/libfuse.so.2 ]; }; then
    exec "$REPO_ROOT/$APPIMAGE_OUTPUT"
fi

echo "FUSE runtime not available; running AppImage via extract-and-run fallback."
exec env APPIMAGE_EXTRACT_AND_RUN=1 "$REPO_ROOT/$APPIMAGE_OUTPUT"
