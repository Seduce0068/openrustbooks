#!/usr/bin/env bash
# CI（rust-gates.yml の extra_gates）と verify.sh から共通で呼ぶ wasm テストゲート。
# 共通ワークフローは wasm-pack を導入しないため、ここで必要な場合のみ導入する。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

WASM_PACK_VERSION="0.13.1"

if ! command -v wasm-pack >/dev/null 2>&1; then
	echo "wasm-pack が見つからないため $WASM_PACK_VERSION を導入します"
	cargo install wasm-pack --locked --version "$WASM_PACK_VERSION"
fi

cd core
wasm-pack test --node --locked
