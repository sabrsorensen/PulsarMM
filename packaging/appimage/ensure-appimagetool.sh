#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

TOOL_PATH="${APPIMAGETOOL:-}"
TOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/12/appimagetool-x86_64.AppImage"
# Keep this in hex because hash_file() returns sha256sum/shasum hex output.
# Equivalent Nix base32: 04ws94q71bwskmhizhwmaf41ma4wabvfgjgkagr8wf3vakgv866r
TOOL_SHA256="d918b4df547b388ef253f3c9e7f6529ca81a885395c31f619d9aaf7030499a13"

hash_file() {
    local file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        echo "Missing sha256 tool: install sha256sum or shasum." >&2
        return 1
    fi
}

verify_tool() {
    local file="$1"
    [ -f "$file" ] || return 1
    [ "$(hash_file "$file")" = "$TOOL_SHA256" ]
}

download_tool() {
    mkdir -p "$REPO_ROOT/.cache"
    TOOL_PATH="$REPO_ROOT/.cache/appimagetool-x86_64.AppImage"

    if command -v curl >/dev/null 2>&1; then
        curl -L --fail -o "$TOOL_PATH" "$TOOL_URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$TOOL_PATH" "$TOOL_URL"
    else
        echo "Missing downloader: install curl or wget, or set APPIMAGETOOL." >&2
        return 1
    fi
}

if [ -n "$TOOL_PATH" ]; then
    if ! verify_tool "$TOOL_PATH"; then
        echo "APPIMAGETOOL does not match expected pinned checksum." >&2
        exit 1
    fi
else
    TOOL_PATH="$REPO_ROOT/.cache/appimagetool-x86_64.AppImage"
    if ! verify_tool "$TOOL_PATH"; then
        rm -f "$TOOL_PATH"
        download_tool
        if ! verify_tool "$TOOL_PATH"; then
            echo "Downloaded appimagetool failed checksum verification." >&2
            exit 1
        fi
    fi
fi

# Nix store paths are read-only; copy to cache if execute bit cannot be set.
if ! [ -x "$TOOL_PATH" ]; then
    if [ -w "$TOOL_PATH" ]; then
        chmod +x "$TOOL_PATH"
    else
        mkdir -p "$REPO_ROOT/.cache"
        cache_tool="$REPO_ROOT/.cache/appimagetool-x86_64.AppImage"
        cp -f "$TOOL_PATH" "$cache_tool"
        chmod +x "$cache_tool"
        TOOL_PATH="$cache_tool"
    fi
fi

printf '%s\n' "$TOOL_PATH"
