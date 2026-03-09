#!/usr/bin/env bash

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "scripts/cargo-env.sh must be sourced, not executed." >&2
    exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/.cache/cargo-target}"
export SCCACHE_DIR="${SCCACHE_DIR:-$REPO_ROOT/.cache/sccache}"
export TMPDIR="${TMPDIR:-$REPO_ROOT/.cache/tmp}"
export TMP="${TMP:-$TMPDIR}"
export TEMP="${TEMP:-$TMPDIR}"
export RUST_DEPS_PATH="${RUST_DEPS_PATH:-}"

# Prefer a vendored Cargo config when available (works around corporate crates.io blocks).
if [[ -n "$RUST_DEPS_PATH" && -f "$RUST_DEPS_PATH/.cargo/config.toml" ]]; then
    REPO_CARGO_HOME="$REPO_ROOT/.cache/cargo-home"
    if [[ -d "$REPO_CARGO_HOME" && -w "$REPO_CARGO_HOME" && ( ! -e "$REPO_CARGO_HOME/config.toml" || -w "$REPO_CARGO_HOME/config.toml" ) ]]; then
        export CARGO_HOME="$REPO_CARGO_HOME"
    elif [[ -z "${CARGO_HOME:-}" ]]; then
        export CARGO_HOME="$(mktemp -d "${TMPDIR:-/tmp}/pulsarmm-cargo-home.XXXXXX")"
    fi

    mkdir -p "$CARGO_HOME"
    rm -f "$CARGO_HOME/config.toml" 2>/dev/null || true
    cp -f "$RUST_DEPS_PATH/.cargo/config.toml" "$CARGO_HOME/config.toml"
    chmod 0644 "$CARGO_HOME/config.toml"
    sed -i "s|directory = \"cargo-vendor-dir\"|directory = \"$RUST_DEPS_PATH\"|" "$CARGO_HOME/config.toml"
    export CARGO_NET_OFFLINE=true
fi

mkdir -p "$CARGO_TARGET_DIR" "$SCCACHE_DIR" "$TMPDIR"

sanitize_cargo_build_dir() {
    local build_dir="$1"
    if [ -d "$build_dir" ]; then
        find "$build_dir" -type f ! -perm -u+w -exec chmod u+w {} + 2>/dev/null || true
        find "$build_dir" -type d ! -perm -u+w -exec chmod u+w {} + 2>/dev/null || true
    fi
}

sanitize_cargo_build_trees() {
    local target_root="$1"
    local build_dir

    if [ ! -d "$target_root" ]; then
        return 0
    fi

    while IFS= read -r build_dir; do
        sanitize_cargo_build_dir "$build_dir"
    done < <(find "$target_root" -type d -path '*/debug/build' 2>/dev/null || true)
}

sanitize_cargo_build_trees "$CARGO_TARGET_DIR"
