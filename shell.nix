{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    nodejs
    rustc
    cargo
    cargo-tauri
    cargo-llvm-cov
    rustfmt
    rust-analyzer
    sccache
    pkg-config
    llvmPackages_19.llvm

    webkitgtk_4_1
    fribidi
    glib-networking
    harfbuzz
    openssl

    flatpak
    flatpak-builder
    patchelf
    git
  ];
  shellHook = ''
    export PATH=$PATH:./node_modules/.bin
    export CARGO_TARGET_DIR="$PWD/.cache/cargo-target"
    export SCCACHE_DIR="$PWD/.cache/sccache"
    export RUSTC_WRAPPER="${pkgs.sccache}/bin/sccache"
    export TMPDIR="$PWD/.cache/tmp"
    export TMP="$TMPDIR"
    export TEMP="$TMPDIR"
    export npm_config_cache="$PWD/.cache/npm"
    export NPM_CONFIG_CACHE="$npm_config_cache"
    mkdir -p "$CARGO_TARGET_DIR" "$SCCACHE_DIR" "$TMPDIR" "$npm_config_cache"
    while IFS= read -r build_dir; do
      if [ -d "$build_dir" ]; then
        find "$build_dir" -type f ! -perm -u+w -exec chmod u+w {} + 2>/dev/null || true
        find "$build_dir" -type d ! -perm -u+w -exec chmod u+w {} + 2>/dev/null || true
      fi
    done < <(find "$CARGO_TARGET_DIR" -type d -path '*/debug/build' 2>/dev/null || true)

    if command -v llvm-cov >/dev/null 2>&1; then
      export LLVM_COV="$(command -v llvm-cov)"
    fi
    if command -v llvm-profdata >/dev/null 2>&1; then
      export LLVM_PROFDATA="$(command -v llvm-profdata)"
    fi

    # Match flake app/dev behavior: use vendored Rust deps from nix build output.
    # Fall back to a temp cargo home when the repo-local cache is not writable.
    REPO_CARGO_HOME="$PWD/.cache/cargo-home"
    mkdir -p "$PWD/.cache" 2>/dev/null || true
    mkdir -p "$REPO_CARGO_HOME" 2>/dev/null || true
    if [ -d "$REPO_CARGO_HOME" ] && [ -w "$REPO_CARGO_HOME" ] && { [ ! -e "$REPO_CARGO_HOME/config.toml" ] || [ -w "$REPO_CARGO_HOME/config.toml" ]; }; then
      export CARGO_HOME="$REPO_CARGO_HOME"
    else
      export CARGO_HOME="$(mktemp -d "''${TMPDIR:-/tmp}/pulsarmm-cargo-home.XXXXXX")"
    fi
    mkdir -p "$CARGO_HOME"
    if command -v nix >/dev/null 2>&1; then
      rm -f "$CARGO_HOME/config.toml" 2>/dev/null || true
      RUST_DEPS_PATH="$(nix build .#rust-deps --no-link --print-out-paths 2>/dev/null | head -n1)"
      if [ -n "$RUST_DEPS_PATH" ] && [ -f "$RUST_DEPS_PATH/.cargo/config.toml" ]; then
        cp -f "$RUST_DEPS_PATH/.cargo/config.toml" "$CARGO_HOME/config.toml"
        chmod 0644 "$CARGO_HOME/config.toml"
        sed -i "s|directory = \"cargo-vendor-dir\"|directory = \"$RUST_DEPS_PATH\"|" "$CARGO_HOME/config.toml"
      fi
    fi

    # Prefer offline mode when vendored config exists to avoid crates.io network policy failures.
    if [ -f "$CARGO_HOME/config.toml" ]; then
      export CARGO_NET_OFFLINE=true
    fi

    echo "\nPulsarMM Nix dev shell loaded!\n"
    echo "- Node: $(node --version)"
    echo "- Rust: $(rustc --version)"
    echo "- Flatpak: $(flatpak --version)"
    echo "- TMPDIR: $TMPDIR"
    if [ -f "$CARGO_HOME/config.toml" ]; then
      echo "- Cargo: offline vendored mode enabled"
    else
      echo "- Cargo: online mode (vendored config unavailable)"
    fi
    if [ "$CARGO_HOME" != "$REPO_CARGO_HOME" ]; then
      echo "- Cargo home fallback: $CARGO_HOME"
    fi
  '';
}
