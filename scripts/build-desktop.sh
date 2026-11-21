#!/usr/bin/env bash
# desktop バイナリを「現行 frontend」からビルドする（ADR-0289 / GOAL.md）。
#
# Tauri の beforeBuildCommand (`trunk build --release`) は `tauri build` 実行時のみ
# 発火し、`cargo build` からは呼ばれない。本スクリプトが web/dist の生成を
# 明示することで、古い／存在しない web/dist を埋め込むことを防ぐ。
# `tauri build` と競合しないよう、ここでは `cargo build` を直接呼ぶ。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

# 開発ホストの絶対パスを配布 WASM に残さない。
# verify.sh の trunk build と同じく、末尾の / を除去して正規化する。
home="${HOME%/}"
if [[ -z "$home" ]]; then
	echo "ERROR: HOME が空のため --remap-path-prefix を設定できません" >&2
	exit 1
fi

# trunk --offline は wasm-bindgen CLI をダウンロードできないため、
# Cargo.lock が固定するバージョンのキャッシュ済み CLI を PATH に追加する。
# 複数バージョンがキャッシュされてもアプリ要求版を優先する（verify.sh と同じ版）。
WASM_BINDGEN_BIN="$HOME/.cache/trunk/wasm-bindgen-0.2.129"
if [[ -d "$WASM_BINDGEN_BIN" ]]; then
	export PATH="$WASM_BINDGEN_BIN:$PATH"
fi

# プロファイル方針: verify.sh の E2E trunk build と同一の dev プロファイルを使う。
# こうすると 256 件の E2E / WDIO で検証した配布物と同じ条件の web/dist を
# desktop に埋め込める。tauri.conf.json の beforeBuildCommand (`--release`) は
# `tauri build` 専用で、本スクリプトは `cargo build` を直接呼ぶため発火しない。
echo "===== START: trunk build (web/dist) ====="
(
	cd web
	RUSTFLAGS="--remap-path-prefix=$home=~" trunk build --offline --locked --dist dist
)
echo "===== END: trunk build rc=0 ====="

# 生成直後の dist を同じゲートで検証する（index.html / wasm / JS の欠落は fail-closed）。
echo "===== START: check-frontend-dist ====="
bash "$SCRIPT_DIR/check-frontend-dist.sh" "$ROOT_DIR"
echo "===== END: check-frontend-dist rc=0 ====="

# tauri-build は tauri.conf.json / capabilities にしか rerun-if-changed を出さず、
# web/dist の更新を cargo が検知しない。build.rs の mtime を進めて build script を
# 再実行させ、generate_context!() が新しい web/dist を埋め込むようにする（内容変更なし）。
touch desktop/build.rs

echo "===== START: cargo build (openrustbooks-desktop) ====="
cargo build -p openrustbooks-desktop --locked
echo "===== END: cargo build rc=0 ====="
