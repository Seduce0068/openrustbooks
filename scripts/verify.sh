#!/usr/bin/env bash
# openrustbooks の全検証を一括実行する。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

# 開発ホストのホームを正規化（末尾の / を除去）し、remap と grep で同じ値を使う。
home=${HOME%/}
if [[ -z "$home" ]]; then
	echo "HOME が空のため開発ホストのパス検査を実行できません"
	exit 1
fi

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
# wasm-bindgen-test のテストは Node 上で実際に実行する（--no-run では不十分）。
# wasm-pack が wasm-bindgen / test-runner を自動的に導入し、--locked でロックファイルを固定する。
run wasm-test bash -c 'cd core && wasm-pack test --node --locked'

# 脆弱性ゲート: advisory DB をフェッチせずローカルキャッシュだけで監査する。
# -n/--no-fetch がオフライン／キャッシュ優先のオプション（git fetch を実行しない）。
# --deny warnings は意図的に使わない（unmaintained 等 10 件は既知 baseline の警告）。
# 脆弱性が 1 件でもあれば cargo audit は rc!=0 を返し、run ヘルパーがゲートを失敗させる。
# cargo-audit 未インストール時は run せず、警告を出して他ゲートは続行する。
# ネットワーク障害等で advisory DB が無い／古い場合は「スキップせずゲート失敗」とする
# （安全側。DB が無い状態でスキップすると脆弱性を見逃すため。DB 更新は明示的な
#  `cargo audit update-db` に委ね、--stale も指定しない）。
if command -v cargo-audit >/dev/null 2>&1; then
	run audit cargo audit --no-fetch
else
	echo "WARNING: cargo-audit not installed; skipping audit gate"
fi

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
if (cd web && RUSTFLAGS="--remap-path-prefix=$home=~" trunk build --offline --dist "$DIST_DIR"); then
	echo "===== END: trunk build rc=0 ====="
else
	rc=$?
	echo "===== END: trunk build rc=$rc ====="
	exit "$rc"
fi

echo "===== START: wasm path check ====="
# glob が非マッチだと grep が rc=2 を返すが if 条件では偽扱いになり
# fail-open になるため、nullglob で配列に展開して件数を明示的に検査する。
# nullglob の元状態を保存し、検査後に必ず復元する。
# shopt -p は無効時に rc=1 を返すため || true で set -e の退出を防ぐ。
nullglob_state=$(shopt -p nullglob || true)
shopt -s nullglob
wasm_files=("$DIST_DIR"/*_bg.wasm)
js_files=("$DIST_DIR"/*.js)
eval "$nullglob_state"

if [[ ${#wasm_files[@]} -eq 0 ]]; then
	echo "配布 WASM が見つかりません"
	exit 1
fi
# wasm-bindgen の生成 JS グルーも配布物なので、欠落時は fail-closed とする。
if [[ ${#js_files[@]} -eq 0 ]]; then
	echo "生成 JS グルーが見つかりません"
	exit 1
fi

# /home/ 固定ではなく正規化済み開発ホーム配下を固定文字列で検索する。
files=("${wasm_files[@]}" "${js_files[@]}")
if grep -a -q -F -- "$home/" "${files[@]}"; then
	echo "配布 WASM（および生成 JS グルー）に開発ホストの絶対パスが含まれています"
	exit 1
else
	grep_rc=$?
	# grep は一致なしで 1、ファイル読取不可などのエラーで 2 を返す。
	# rc>=2 は検査不能なので通過させない（fail-closed）。
	if ((grep_rc >= 2)); then
		echo "配布 WASM / JS グルーのパス検査に失敗しました (grep rc=$grep_rc)"
		exit 1
	fi
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
