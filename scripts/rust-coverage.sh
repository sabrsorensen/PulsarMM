#!/usr/bin/env bash
set -euo pipefail

MANIFEST_PATH="${1:-src-tauri/Cargo.toml}"
SCOPE="${RUST_COVERAGE_SCOPE:-all-targets}"
ENFORCE_THRESHOLDS="${RUST_COVERAGE_ENFORCE:-1}"
MIN_REGION_COVERAGE="${RUST_COVERAGE_MIN_REGIONS:-98.56}"
MIN_FUNCTION_COVERAGE="${RUST_COVERAGE_MIN_FUNCTIONS:-100.00}"
MIN_LINE_COVERAGE="${RUST_COVERAGE_MIN_LINES:-99.87}"
# Ignore only files that intentionally contain thin Tauri/bootstrap wrappers.
# Testable domain and seam logic should remain outside these files.
IGNORE_REGEX="${RUST_COVERAGE_IGNORE_REGEX:-(^.*/src/adapters/tauri/.*\\.rs$|^.*/src/main\\.rs$|^.*/src/lib\\.rs$|^.*/src/mods/archive_rar_backend\\.rs$|^.*/src/mods/archive_rar_runtime\\.rs$|^.*/src/linux/runtime\\.rs$|^.*/src/mods/install_command_runtime\\.rs$)}"

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"
source scripts/cargo-env.sh

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; cannot run Rust coverage." >&2
  exit 1
fi

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  cat >&2 <<'EOF'
cargo-llvm-cov is not installed.

Install options:
- nix develop / nix-shell (project shells now include coverage tools)
- cargo install cargo-llvm-cov
EOF
  exit 1
fi

# Use rustup-installed llvm-tools if present. Otherwise rely on PATH.
if [[ -z "${LLVM_COV:-}" ]] && command -v llvm-cov >/dev/null 2>&1; then
  export LLVM_COV
  LLVM_COV="$(command -v llvm-cov)"
fi
if [[ -z "${LLVM_PROFDATA:-}" ]] && command -v llvm-profdata >/dev/null 2>&1; then
  export LLVM_PROFDATA
  LLVM_PROFDATA="$(command -v llvm-profdata)"
fi

export RUSTC_WRAPPER=""

IGNORE_ARGS=()
if [[ -n "$IGNORE_REGEX" ]]; then
  IGNORE_ARGS=(--ignore-filename-regex "$IGNORE_REGEX")
fi

run_summary() {
  local summary_file="$1"
  if [[ "$SCOPE" == "all-targets" ]]; then
    echo "Scope: combined unit + integration + doctest targets (--all-targets)"
    cargo llvm-cov \
      --manifest-path "$MANIFEST_PATH" \
      --all-targets \
      "${IGNORE_ARGS[@]}" \
      --summary-only | tee "$summary_file"
  elif [[ "$SCOPE" == "tests-only" ]]; then
    echo "Scope: test binaries only (unit + integration tests via --tests)"
    cargo llvm-cov \
      --manifest-path "$MANIFEST_PATH" \
      --tests \
      "${IGNORE_ARGS[@]}" \
      --summary-only | tee "$summary_file"
  else
    echo "Unknown RUST_COVERAGE_SCOPE='$SCOPE' (expected: all-targets|tests-only)" >&2
    exit 2
  fi
}

enforce_thresholds() {
  local summary_file="$1"
  local total_line
  local actual_regions
  local actual_functions
  local actual_lines

  total_line="$(awk '$1 == "TOTAL" { print $0 }' "$summary_file" | tail -n1)"
  if [[ -z "$total_line" ]]; then
    echo "Failed to find TOTAL coverage line in llvm-cov summary." >&2
    exit 3
  fi

  read -r actual_regions actual_functions actual_lines <<<"$(awk '$1 == "TOTAL" { gsub(/%/, "", $4); gsub(/%/, "", $7); gsub(/%/, "", $10); print $4, $7, $10 }' "$summary_file" | tail -n1)"

  if ! awk -v actual="$actual_regions" -v min="$MIN_REGION_COVERAGE" 'BEGIN { exit !(actual + 0 >= min + 0) }'; then
    echo "Rust region coverage gate failed: $actual_regions% < $MIN_REGION_COVERAGE%" >&2
    exit 4
  fi

  if ! awk -v actual="$actual_functions" -v min="$MIN_FUNCTION_COVERAGE" 'BEGIN { exit !(actual + 0 >= min + 0) }'; then
    echo "Rust function coverage gate failed: $actual_functions% < $MIN_FUNCTION_COVERAGE%" >&2
    exit 5
  fi

  if ! awk -v actual="$actual_lines" -v min="$MIN_LINE_COVERAGE" 'BEGIN { exit !(actual + 0 >= min + 0) }'; then
    echo "Rust line coverage gate failed: $actual_lines% < $MIN_LINE_COVERAGE%" >&2
    exit 6
  fi

  echo "Rust coverage gate passed: regions=$actual_regions% functions=$actual_functions% lines=$actual_lines%"
}

echo "Running Rust coverage (summary)..."
summary_file="$(mktemp)"
trap 'rm -f "$summary_file"' EXIT
run_summary "$summary_file"

if [[ "$ENFORCE_THRESHOLDS" != "0" ]]; then
  enforce_thresholds "$summary_file"
fi
