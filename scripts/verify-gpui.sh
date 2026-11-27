#!/usr/bin/env bash
# GPUI 独立 workspace のローカルゲート。ルート workspace の verify.sh とは
# 分離して実行する（rust-version 1.100 / 別 lockfile / ネイティブ图形スタック）。
set -euo pipefail

die() {
	printf 'ERROR: %s\n' "$*" >&2
	exit 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GPUI_DIR="$ROOT_DIR/gpui"

[[ -f "$GPUI_DIR/Cargo.toml" ]] || die "gpui/Cargo.toml がありません: $GPUI_DIR"
[[ -f "$GPUI_DIR/Cargo.lock" ]] || die "gpui/Cargo.lock がありません（cargo check で生成）"

rust_version="$(rustc --version)"
major="${rust_version#rustc }"
major="${major%%.*}"
minor="${rust_version#*.}"
minor="${minor%%.*}"
if ((major < 1 || (major == 1 && minor < 100))); then
	die "GPUI には rustc >= 1.100 が必要です: $rust_version"
fi

cd "$GPUI_DIR"
echo "===== gpui: fmt ====="
cargo fmt --all -- --check
echo "===== gpui: check ====="
cargo check --all-targets --locked
echo "===== gpui: test ====="
cargo test --locked
echo "===== gpui: clippy ====="
cargo clippy --all-targets --locked -- -D warnings -D clippy::undocumented_unsafe_blocks
echo "===== gpui: dependency-hygiene ====="
bash "$ROOT_DIR/scripts/check-dependency-hygiene.sh" "$GPUI_DIR/Cargo.lock"
echo "===== gpui: all gates passed ====="
