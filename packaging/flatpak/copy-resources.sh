#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

PULSAR_BIN="${PULSAR_BIN:-src-tauri/target/release/Pulsar}"

mkdir -p flatpak-source

bash packaging/validate-metainfo.sh

cp "$PULSAR_BIN" flatpak-source/Pulsar

if [ "${PULSAR_PATCH_FOR_FLATPAK:-0}" = "1" ]; then
    echo "Patching binary for flatpak runtime..."
    chmod +w flatpak-source/Pulsar
    patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 --remove-rpath flatpak-source/Pulsar
fi

if [ -d src-tauri/target/release/locales ]; then
    cp -r src-tauri/target/release/locales flatpak-source/locales
fi

if [ -f src-tauri/icons/128x128.png ]; then
    cp src-tauri/icons/128x128.png flatpak-source/pulsar.png
fi

cp packaging/pulsar.metainfo.xml flatpak-source/pulsar.metainfo.xml

mkdir -p flatpak-source/screenshots
cp screenshots/Screenshot*.png flatpak-source/screenshots/ 2>/dev/null || true

cp packaging/flatpak/pulsar-wrapper flatpak-source/pulsar-wrapper
chmod +x flatpak-source/pulsar-wrapper

cp packaging/pulsar.desktop.template flatpak-source/pulsar.desktop
