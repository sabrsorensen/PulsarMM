{
  description = "Pulsar Mod Manager - Nix Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        pname = "pulsar-mm";
        version = "dev";

        # Only include files needed for the build — excludes .github, scripts,
        # screenshots, build artifacts, etc.
        src = pkgs.lib.fileset.toSource {
          root = ./.;
          fileset = pkgs.lib.fileset.unions [
            ./src            # Frontend source
            ./src-tauri       # Rust source, Cargo.toml, tauri.conf.json, icons
            ./index.html      # Frontend entry point
            ./package.json
            ./package-lock.json
            ./vite.config.js  # Vite build config
          ];
        };

        npmDeps = pkgs.fetchNpmDeps {
          name = "${pname}-${version}-npm-deps";
          inherit src;
          hash = "sha256-Dk3ufqpyIm1WmwScrNRDaW43tEsWysj4NGgmtfUahSQ=";
        };

        appImageTool = pkgs.fetchurl {
          url = "https://github.com/AppImage/AppImageKit/releases/download/12/appimagetool-x86_64.AppImage";
          sha256 = "04ws94q71bwskmhizhwmaf41ma4wabvfgjgkagr8wf3vakgv866r";
        };

        # Shared build config — all packages reuse this derivation
        pulsar = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
          inherit pname version src npmDeps;
          cargoLock = {
            lockFile = ./src-tauri/Cargo.lock;
            # Avoid fetch-cargo-vendor-util TLS issues behind corporate MITM proxies.
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = [
            pkgs.cargo-tauri.hook
            pkgs.nodejs
            pkgs.npmHooks.npmConfigHook
            pkgs.pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.wrapGAppsHook4 ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            pkgs.fribidi
            pkgs.glib-networking
            pkgs.harfbuzz
            pkgs.openssl
            pkgs.webkitgtk_4_1
          ];
          cargoRoot = "src-tauri";
          buildAndTestSubdir = "src-tauri";
          tauriBuildFlags = [ "--config" ''{"bundle":{"createUpdaterArtifacts":false},"plugins":{"updater":{"active":false}}}'' ];
          tauriBundleType = "deb"; # Required — install hook extracts binary from deb bundle
        });

        # Reuse the same vendored cargo dependency path as the successful default package.
        rustDeps = pulsar.cargoDeps;
        pulsarBackend = pkgs.rustPlatform.buildRustPackage {
          pname = "${pname}-backend";
          inherit version;
          src = ./src-tauri;
          cargoDeps = rustDeps;
          cargoLock = {
            lockFile = ./src-tauri/Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            pkgs.fribidi
            pkgs.glib-networking
            pkgs.harfbuzz
            pkgs.openssl
            pkgs.webkitgtk_4_1
          ];
          doCheck = false;
        };

        pkgConfigDeps = [
          pkgs.atk
          pkgs.cairo
          pkgs.fontconfig
          pkgs.gdk-pixbuf
          pkgs.glib
          pkgs.gtk3
          pkgs.libepoxy
          pkgs.libsoup_3
          pkgs.libxkbcommon
          pkgs.openssl
          pkgs.pango
          pkgs.wayland
          pkgs.webkitgtk_4_1
          pkgs.libx11
          pkgs.libxcomposite
          pkgs.libxcursor
          pkgs.libxdamage
          pkgs.libxext
          pkgs.libxfixes
          pkgs.libxi
          pkgs.libxinerama
          pkgs.libxrandr
        ];
        # Build scripts — shared between build and run apps
        packagingRuntimeCore = [
          pkgs.cacert
          pkgs.cargo
          pkgs.coreutils
          pkgs.stdenv.cc
          pkgs.file
          pkgs.findutils
          pkgs.git
          pkgs.gnused
          pkgs.gawk
          pkgs.nodejs
          pkgs.pkg-config
          pkgs.patchelf
          pkgs.wget
          pkgs.zsync
          pkgs.rustc
          pkgs.rustfmt
          pkgs.rust-analyzer
          pkgs.sccache
          pkgs.cargo-llvm-cov
          pkgs.llvmPackages_19.llvm
        ] ++ pkgConfigDeps ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
          pkgs.fribidi
          pkgs.glib-networking
          pkgs.harfbuzz
        ];

        appimageRuntimeInputs = packagingRuntimeCore ++ [
          pkgs.appstream
        ];

        flatpakRuntimeInputs = packagingRuntimeCore ++ [
          pkgs.appstream
          pkgs.flatpak
          pkgs.flatpak-builder
        ];
        pkgConfigPath =
          (pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" pkgConfigDeps)
          + ":"
          + (pkgs.lib.makeSearchPath "lib/pkgconfig" pkgConfigDeps);

        runTestsScript = pkgs.writeShellApplication {
          name = "run-tests";
          runtimeInputs = [
            pkgs.git
            pkgs.nodejs
          ];
          text = ''
            set -e
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"
            if [ ! -x node_modules/.bin/vite ]; then
              echo "Installing npm dependencies for tests..."
              npm ci --prefer-offline --no-audit --no-fund
            fi
            npm run test:coverage
          '';
        };

        runQualityGateScript = pkgs.writeShellApplication {
          name = "run-quality-gate";
          runtimeInputs = [
            pkgs.git
            pkgs.nix
          ];
          text = ''
            set -e
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"
            exec nix develop --command bash scripts/run-quality-gate.sh
          '';
        };

        buildFlatpakScript = pkgs.writeShellApplication {
          name = "build-flatpak-bundle";
          runtimeInputs = flatpakRuntimeInputs;
          text = ''
            set -e
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"
            export RUST_DEPS_PATH="${rustDeps}"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export PKG_CONFIG_PATH="${pkgConfigPath}"
            : "''${PULSAR_SKIP_TEST_GATE:=1}"
            export PULSAR_SKIP_TEST_GATE
            export PULSAR_BIN="${pulsar}/bin/.Pulsar-wrapped"
            export PULSAR_PATCH_FOR_FLATPAK=1
            exec bash packaging/flatpak/package-flatpak.sh
          '';
        };

        buildAppimageScript = pkgs.writeShellApplication {
          name = "build-appimage";
          runtimeInputs = appimageRuntimeInputs;
          text = ''
            set -e
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"
            export RUST_DEPS_PATH="${rustDeps}"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export PKG_CONFIG_PATH="${pkgConfigPath}"
            : "''${PULSAR_SKIP_TEST_GATE:=1}"
            export PULSAR_SKIP_TEST_GATE
            export PULSAR_BIN="${pulsar}/bin/.Pulsar-wrapped"
            export APPIMAGETOOL="${appImageTool}"
            exec bash packaging/appimage/build-appimage.sh
          '';
        };

        runRustCoverageScript = pkgs.writeShellApplication {
          name = "run-rust-coverage";
          runtimeInputs = [ pkgs.nix ];
          text = ''
            set -e
            if [ ! -f package.json ]; then
              echo "Run from the repository root (where package.json exists)." >&2
              exit 1
            fi
            exec nix develop --command bash -lc "npm run test:rust:coverage"
          '';
        };

        runDevScript = pkgs.writeShellApplication {
          name = "run-dev";
          runtimeInputs = [ pkgs.nix ];
          text = ''
            set -e
            if [ ! -f package.json ]; then
              echo "Run from the repository root (where package.json exists)." >&2
              exit 1
            fi
            exec nix develop --command bash -lc "
              mkdir -p .cache/cargo-home
              rm -f .cache/cargo-home/config.toml
              cp -f '${pulsar.cargoDeps}/.cargo/config.toml' .cache/cargo-home/config.toml
              chmod 0644 .cache/cargo-home/config.toml
              sed -i 's|directory = \"cargo-vendor-dir\"|directory = \"${pulsar.cargoDeps}\"|' .cache/cargo-home/config.toml
              export CARGO_HOME=\"\$PWD/.cache/cargo-home\"
              export CARGO_NET_OFFLINE=true
              if [ ! -x node_modules/.bin/vite ]; then
                echo 'Installing npm dependencies for dev...'
                npm ci --prefer-offline
              fi
              exec cargo tauri dev
            "
          '';
        };

        runFlatpakScript = pkgs.writeShellApplication {
          name = "run-flatpak";
          runtimeInputs = flatpakRuntimeInputs;
          text = ''
            set -e

            # Build the flatpak first
            echo "=== Building Flatpak ==="
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"
            export RUST_DEPS_PATH="${rustDeps}"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export PKG_CONFIG_PATH="${pkgConfigPath}"
            : "''${PULSAR_SKIP_TEST_GATE:=1}"
            export PULSAR_SKIP_TEST_GATE
            export PULSAR_BIN="${pulsar}/bin/.Pulsar-wrapped"
            export PULSAR_PATCH_FOR_FLATPAK=1
            exec bash packaging/flatpak/run-flatpak.sh
          '';
        };

        runAppimageScript = pkgs.writeShellApplication {
          name = "run-appimage";
          runtimeInputs = appimageRuntimeInputs;
          text = ''
            set -e

            # Build the AppImage first
            echo "=== Building AppImage ==="
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"
            export RUST_DEPS_PATH="${rustDeps}"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export PKG_CONFIG_PATH="${pkgConfigPath}"
            : "''${PULSAR_SKIP_TEST_GATE:=1}"
            export PULSAR_SKIP_TEST_GATE
            export PULSAR_BIN="${pulsar}/bin/.Pulsar-wrapped"
            export APPIMAGETOOL="${appImageTool}"
            exec bash packaging/appimage/run-appimage.sh
          '';
        };

        allTargetsScript = pkgs.writeShellApplication {
          name = "all-targets";
          runtimeInputs = [ pkgs.git pkgs.nix ];
          text = ''
            set -e
            REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
            cd "$REPO_ROOT"

            echo "=== Running tests ==="
            nix run .#quality-gate

            echo "=== Building Nix packages ==="
            nix build .#default .#backend .#rust-deps

            echo "=== Building Flatpak ==="
            nix run .#flatpak

            echo "=== Building AppImage ==="
            nix run .#appimage

            echo "All targets passed."
          '';
        };

        mkApp = description: program: {
          type = "app";
          inherit program;
          meta = { inherit description; };
        };

        devShell = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.cargo-tauri
            pkgs.cargo-llvm-cov
            pkgs.rustc
            pkgs.rustfmt
            pkgs.rust-analyzer
            pkgs.sccache
            pkgs.nodejs
            pkgs.pkg-config
            pkgs.llvmPackages_19.llvm
          ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            pkgs.fribidi
            pkgs.glib-networking
            pkgs.harfbuzz
            pkgs.openssl
            pkgs.webkitgtk_4_1
          ];

          shellHook = ''
            export RUST_DEPS_PATH="${rustDeps}"
            REPO_CARGO_HOME="$PWD/.cache/cargo-home"
            if [ -d "$REPO_CARGO_HOME" ] && [ -w "$REPO_CARGO_HOME" ] && { [ ! -e "$REPO_CARGO_HOME/config.toml" ] || [ -w "$REPO_CARGO_HOME/config.toml" ]; }; then
              export CARGO_HOME="$REPO_CARGO_HOME"
            else
              export CARGO_HOME="$(mktemp -d "''${TMPDIR:-/tmp}/pulsarmm-cargo-home.XXXXXX")"
            fi
            mkdir -p "$CARGO_HOME"
            rm -f "$CARGO_HOME/config.toml" 2>/dev/null || true
            if [ -n "$RUST_DEPS_PATH" ] && [ -f "$RUST_DEPS_PATH/.cargo/config.toml" ]; then
              cp -f "$RUST_DEPS_PATH/.cargo/config.toml" "$CARGO_HOME/config.toml"
              chmod 0644 "$CARGO_HOME/config.toml"
              sed -i "s|directory = \"cargo-vendor-dir\"|directory = \"$RUST_DEPS_PATH\"|" "$CARGO_HOME/config.toml"
            fi
            if [ -f "$CARGO_HOME/config.toml" ]; then
              export CARGO_NET_OFFLINE=true
            fi
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
            echo "Rust build cache enabled:"
            echo "  CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
            echo "  SCCACHE_DIR=$SCCACHE_DIR"
            echo "  TMPDIR=$TMPDIR"
            echo "npm cache enabled:"
            echo "  npm_config_cache=$npm_config_cache"
          '';
        };
      in
      {
        # nix build — compile check only, produces the binary
        packages.default = pulsar;
        packages.rust-deps = rustDeps;
        packages.backend = pulsarBackend;

        checks = {
          package_default = pulsar;
          package_backend = pulsarBackend;
          package_rust_deps = rustDeps;
          packaging_validate_metainfo = pkgs.runCommand "validate-metainfo" { buildInputs = [ pkgs.appstream ]; } ''
            cd ${./.}
            bash packaging/validate-metainfo.sh
            touch "$out"
          '';
          dev_shell = devShell.inputDerivation;
        };

        # nix run — launch the app directly
        apps.default = mkApp "Run the Pulsar desktop application." "${pulsar}/bin/Pulsar";

        # nix run .#tests — run coverage-gated test suite
        apps.tests = mkApp "Run the Node.js coverage-gated test suite." "${runTestsScript}/bin/run-tests";

        # nix run .#quality-gate — run the full JS + Rust quality gate
        apps.quality-gate = mkApp "Run the full JS and Rust quality gate inside the dev shell." "${runQualityGateScript}/bin/run-quality-gate";

        # nix run .#rust-coverage — run Rust coverage summary (requires cargo-llvm-cov)
        apps.rust-coverage = mkApp "Run the Rust coverage summary inside the dev shell." "${runRustCoverageScript}/bin/run-rust-coverage";

        # nix run .#dev — run through nix develop to reuse local sccache/target cache
        apps.dev = mkApp "Run cargo tauri dev through nix develop." "${runDevScript}/bin/run-dev";

        # nix run .#flatpak — build flatpak bundle (requires host flatpak runtimes)
        apps.flatpak = mkApp "Build the Flatpak bundle using the host Flatpak toolchain." "${buildFlatpakScript}/bin/build-flatpak-bundle";

        # nix run .#run-flatpak — build, install, and run the flatpak
        apps.run-flatpak = mkApp "Build, install, and launch the Flatpak package." "${runFlatpakScript}/bin/run-flatpak";

        # nix run .#appimage — build AppImage with pinned appimagetool
        apps.appimage = mkApp "Build the AppImage artifact." "${buildAppimageScript}/bin/build-appimage";

        # nix run .#run-appimage — build and run the AppImage
        apps.run-appimage = mkApp "Build and launch the AppImage artifact." "${runAppimageScript}/bin/run-appimage";

        # nix run .#all-targets — run tests, build packages, and build distributables
        apps.all-targets = mkApp "Run the combined tests and distributable build flow." "${allTargetsScript}/bin/all-targets";

        # nix develop — cached Rust/Tauri dev environment
        devShells.default = devShell;
      }
    );
}
