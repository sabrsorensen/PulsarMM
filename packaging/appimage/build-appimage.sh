#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

APPIMAGE_OUTPUT="${PULSAR_APPIMAGE_OUTPUT:-PulsarMM-Linux.AppImage}"
APPDIR="PulsarMM.AppDir"

collect_ldd_paths() {
    ldd "$1" 2>/dev/null | awk '
        $2 == "=>" && $3 ~ /^\// { print $3 }
        $1 ~ /^\// { print $1 }
    ' | sort -u
}

skip_bundled_lib() {
    case "$1" in
        ld-linux*|libc.so*|libpthread.so*|libdl.so*|libm.so*|librt.so*)
            return 0
            ;;
    esac
    return 1
}

bundle_libs_from() {
    local binary="$1"
    collect_ldd_paths "$binary" | while read -r lib; do
        local name
        name="$(basename "$lib")"
        if ! skip_bundled_lib "$name"; then
            cp "$lib" "$APPDIR/usr/lib/"
        fi
    done
}

if [ "${PULSAR_SKIP_TEST_GATE:-0}" != "1" ]; then
    echo "Running quality gate..."
    bash scripts/run-quality-gate.sh
else
    echo "Skipping quality gate because PULSAR_SKIP_TEST_GATE=1"
fi

if [ -z "${PULSAR_BIN:-}" ]; then
    bash scripts/build-tauri-binary.sh
    PULSAR_BIN="$REPO_ROOT/src-tauri/target/release/Pulsar"
fi

APPIMAGETOOL_BIN="$(bash packaging/appimage/ensure-appimagetool.sh)"

echo "Preparing AppDir..."
rm -rf "$APPDIR" "$APPIMAGE_OUTPUT"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib"
mkdir -p "$APPDIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/metainfo"

cp "$PULSAR_BIN" "$APPDIR/usr/bin/Pulsar"
chmod +wx "$APPDIR/usr/bin/Pulsar"

if [ -d src-tauri/target/release/locales ]; then
    cp -r src-tauri/target/release/locales "$APPDIR/usr/bin/locales"
fi

if [ -f src-tauri/icons/128x128.png ]; then
    cp src-tauri/icons/128x128.png "$APPDIR/usr/share/icons/hicolor/128x128/apps/com.sabrsorensen.Pulsar.png"
    cp src-tauri/icons/128x128.png "$APPDIR/com.sabrsorensen.Pulsar.png"
fi

sed -e 's|^Exec=.*|Exec=Pulsar|' \
    -e 's|^Icon=.*|Icon=com.sabrsorensen.Pulsar|' \
    packaging/pulsar.desktop.template > "$APPDIR/com.sabrsorensen.Pulsar.desktop"
cp "$APPDIR/com.sabrsorensen.Pulsar.desktop" "$APPDIR/usr/share/applications/"

cp packaging/pulsar.metainfo.xml "$APPDIR/usr/share/metainfo/com.sabrsorensen.Pulsar.appdata.xml"

printf '%s\n' \
  "#!/bin/bash" \
  "HERE=\"\$(dirname \"\$(readlink -f \"\$0\")\")\"" \
  "export LD_LIBRARY_PATH=\"\$HERE/usr/lib:\$LD_LIBRARY_PATH\"" \
  "export GDK_BACKEND=x11" \
  "exec \"\$HERE/usr/bin/Pulsar\" \"\$@\"" \
  > "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"

echo "Bundling shared libraries..."
bundle_libs_from "$APPDIR/usr/bin/Pulsar"

echo "Bundling transitive dependencies..."
for _pass in 1 2 3; do
    found_new=0
    while read -r lib; do
        name="$(basename "$lib")"
        if [ ! -f "$APPDIR/usr/lib/$name" ] && ! skip_bundled_lib "$name"; then
            cp "$lib" "$APPDIR/usr/lib/"
            found_new=1
        fi
    done < <(find "$APPDIR/usr/lib" -name '*.so*' -exec sh -c '
        for file in "$@"; do
            ldd "$file" 2>/dev/null | awk '"'"'
                $2 == "=>" && $3 ~ /^\// { print $3 }
                $1 ~ /^\// { print $1 }
            '"'"'
        done
    ' sh {} + | sort -u)
    if [ "$found_new" = 0 ]; then
        break
    fi
done

echo "Bundled $(find "$APPDIR/usr/lib" -name '*.so*' | wc -l) libraries"

bash packaging/validate-metainfo.sh

echo "Building AppImage..."
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$APPIMAGETOOL_BIN" --no-appstream "$APPDIR" "$APPIMAGE_OUTPUT"
echo "AppImage built: $APPIMAGE_OUTPUT ($(du -h "$APPIMAGE_OUTPUT" | cut -f1))"
