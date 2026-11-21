#!/usr/bin/env bash
# CI（rust-gates.yml の extra_gates）と verify.sh から共通で呼ぶ wasm テストゲート。
# 共通ワークフローは wasm-pack を導入しないため、ここで必要な場合のみ導入する。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

WASM_PACK_VERSION="0.13.1"

# wasm-pack --version の出力は "wasm-pack <version>"。
# 末尾にコミット情報などが付く環境でも誤拒否しないよう、バージョン値の直後が
# 空白または行末であることを確認する（"0.13.10" のような誤許可はしない）。
# 出力が取得できない場合も成功扱いにせず、呼び出し側で fail-closed にする。
check_wasm_pack_version() {
	local output
	output="$(wasm-pack --version 2>/dev/null)" || return 1
	[[ "$output" == "wasm-pack $WASM_PACK_VERSION" || "$output" == "wasm-pack $WASM_PACK_VERSION "* ]]
}

if command -v wasm-pack >/dev/null 2>&1; then
	current="$(wasm-pack --version 2>/dev/null || true)"
	if ! check_wasm_pack_version; then
		echo "ERROR: wasm-pack のバージョンは $WASM_PACK_VERSION 固定です: ${current:-version 取得失敗}" >&2
		exit 1
	fi
else
	echo "wasm-pack が見つからないため $WASM_PACK_VERSION を導入します"
	cargo install wasm-pack --locked --version "$WASM_PACK_VERSION"
	# cargo install の導入先が PATH に無い CI でも検出できるようにする（CARGO_HOME 対応）。
	export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
	# 導入後も期待バージョンであることを確認し、不一致なら fail-closed とする。
	current="$(wasm-pack --version 2>/dev/null || true)"
	if ! command -v wasm-pack >/dev/null 2>&1 || ! check_wasm_pack_version; then
		echo "ERROR: 導入後の wasm-pack が $WASM_PACK_VERSION ではありません: ${current:-version 取得失敗}" >&2
		exit 1
	fi
fi

cd core
wasm-pack test --node --locked
