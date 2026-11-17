#!/usr/bin/env bash
# openrustbooks の全検証を一括実行する。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

fail=0
run() {
	local name="$1"
	shift
	echo "===== START: $name ====="
	if "$@"; then
		echo "===== END: $name rc=0 ====="
	else
		local rc=$?
		echo "===== END: $name rc=$rc ====="
		fail=1
	fi
}

run fmt cargo fmt --all -- --check
run check cargo check --all-targets --locked
run test cargo test --all-targets --locked
run clippy cargo clippy --all-targets --locked -- -D warnings
run wasm-core cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked
run wasm-web cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked

if [[ $fail -ne 0 ]]; then
	echo "===== ゲート失敗: E2E は実行しません ====="
	exit 1
fi

# wasm-bindgen-cli がホーム配下にある場合は PATH に追加する。
if [[ -d "$HOME/.cache/trunk/wasm-bindgen-0.2.129" ]]; then
	export PATH="$HOME/.cache/trunk/wasm-bindgen-0.2.129:$PATH"
fi

DIST_DIR="$(mktemp -d)"
trap 'rm -rf "$DIST_DIR"' EXIT

echo "===== START: trunk build ====="
# 開発ホストの絶対パスを配布 WASM に残さない（rustc の正しいフラグは --remap-path-prefix）。
if (cd web && RUSTFLAGS="--remap-path-prefix=$HOME=~" trunk build --offline --dist "$DIST_DIR"); then
	echo "===== END: trunk build rc=0 ====="
else
	rc=$?
	echo "===== END: trunk build rc=$rc ====="
	exit "$rc"
fi

echo "===== START: wasm path check ====="
if grep -a -q '/home/' "$DIST_DIR"/*_bg.wasm; then
	echo "配布 WASM に開発ホストの絶対パスが含まれています"
	exit 1
fi
echo "===== END: wasm path check rc=0 ====="

echo "===== START: e2e ====="
if python3 tools/verify-web-reader.py --dist "$DIST_DIR"; then
	echo "===== END: e2e rc=0 ====="
else
	rc=$?
	echo "===== END: e2e rc=$rc ====="
	exit "$rc"
fi

echo "===== 全検証成功 ====="
