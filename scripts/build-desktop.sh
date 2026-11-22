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

# --- wasm-bindgen CLI ゲート（verify.sh / build-desktop.sh で同一。変更時は両方を更新） ---
# バージョンの単一情報源は Cargo.lock。固定パスのディレクトリ名だけを信じず、
# PATH 上で解決された CLI の --version まで照合して fail-closed にする。

# Cargo.lock が固定する wasm-bindgen のバージョンを抽出する。
# 抽出失敗（読取不能 / 0 件 / 複数件 / semver 不正）はすべて fail-closed で rc=1。
wasm_bindgen_locked_version() {
	local lock_file="${1:-${ROOT_DIR:-.}/Cargo.lock}"
	local -a versions=()

	if [[ ! -r "$lock_file" ]]; then
		echo "ERROR: Cargo.lock が読めません: $lock_file" >&2
		return 1
	fi

	# [[package]] ブロック単位で name / version を追跡し、name がちょうど
	# "wasm-bindgen" のブロックの version のみを出力する。
	# wasm-bindgen-futures 等の接頭辞一致はアンカーで拒否する。
	mapfile -t versions < <(
		awk '
			function emit() {
				if (is_target && version != "") print version
			}
			/^\[\[package\]\][[:space:]]*$/ {
				emit()
				is_target = 0
				version = ""
				next
			}
			/^name[[:space:]]*=[[:space:]]*"/ {
				n = $0
				sub(/^name[[:space:]]*=[[:space:]]*"/, "", n)
				sub(/"[[:space:]]*$/, "", n)
				is_target = (n == "wasm-bindgen")
				next
			}
			is_target && /^version[[:space:]]*=[[:space:]]*"/ {
				v = $0
				sub(/^version[[:space:]]*=[[:space:]]*"/, "", v)
				sub(/"[[:space:]]*$/, "", v)
				version = v
				next
			}
			END { emit() }
		' "$lock_file"
	)

	if [[ ${#versions[@]} -ne 1 ]]; then
		echo "ERROR: Cargo.lock の wasm-bindgen エントリが不正です（${#versions[@]} 件）: $lock_file" >&2
		return 1
	fi
	if [[ ! "${versions[0]}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		echo "ERROR: Cargo.lock の wasm-bindgen バージョンが不正です: ${versions[0]}" >&2
		return 1
	fi
	printf '%s\n' "${versions[0]}"
}

# Cargo.lock の版に対応する trunk キャッシュ CLI を PATH の先頭へ追加する。
# キャッシュが無い場合も追加せず、後続の check_wasm_bindgen_version で fail-closed にする。
setup_wasm_bindgen_path() {
	local lock_file="${1:-${ROOT_DIR:-.}/Cargo.lock}"
	local version cache_dir

	version="$(wasm_bindgen_locked_version "$lock_file")" || return 1
	cache_dir="${HOME%/}/.cache/trunk/wasm-bindgen-$version"
	if [[ -d "$cache_dir" ]]; then
		export PATH="$cache_dir:$PATH"
	else
		echo "警告: wasm-bindgen CLI キャッシュがありません: $cache_dir" >&2
	fi
}

# PATH 上で解決される wasm-bindgen CLI の --version が Cargo.lock 版と一致するか検査する。
# 不一致 / コマンド不在 / 実行失敗 / 抽出失敗はすべて fail-closed（rc=1）。
check_wasm_bindgen_version() {
	local lock_file="${1:-${ROOT_DIR:-.}/Cargo.lock}"
	local expected output

	expected="$(wasm_bindgen_locked_version "$lock_file")" || return 1

	if ! command -v wasm-bindgen >/dev/null 2>&1; then
		echo "ERROR: wasm-bindgen CLI が PATH にありません（Cargo.lock: $expected）" >&2
		return 1
	fi

	output="$(wasm-bindgen --version 2>&1)" || {
		echo "ERROR: wasm-bindgen --version が失敗しました（期待: $expected）: ${output:-<no output>}" >&2
		return 1
	}

	# 出力は "wasm-bindgen <version>"。バージョン直後が空白または行末であることを確認し、
	# "0.2.1290" のような誤許可を防ぐ（ci-wasm-test.sh の wasm-pack ゲートと同じ方針）。
	if [[ "$output" != "wasm-bindgen $expected" && "$output" != "wasm-bindgen $expected "* ]]; then
		echo "ERROR: wasm-bindgen CLI と Cargo.lock のバージョンが不一致 (CLI: ${output:-<no output>}, Cargo.lock: $expected)" >&2
		return 1
	fi

	echo "wasm-bindgen CLI バージョン一致: $expected (Cargo.lock)"
}
# --- wasm-bindgen CLI ゲート END（verify.sh / build-desktop.sh で同一） ---

# wasm-bindgen ゲート単独の実行（負向テスト／運用確認用）。
# 第1引数に Cargo.lock のパスを渡すと、そのファイルを抽出対象にできる。
if [[ "${1:-}" == "--check-wasm-bindgen" ]]; then
	shift
	if (($# > 0)); then
		check_lock="$1"
	else
		check_lock="$ROOT_DIR/Cargo.lock"
	fi
	setup_wasm_bindgen_path "$check_lock" || exit 1
	check_wasm_bindgen_version "$check_lock" || exit 1
	exit 0
fi

# 開発ホストの絶対パスを配布 WASM に残さない。
# verify.sh の trunk build と同じく、末尾の / を除去して正規化する。
home="${HOME%/}"
if [[ -z "$home" ]]; then
	echo "ERROR: HOME が空のため --remap-path-prefix を設定できません" >&2
	exit 1
fi

# trunk --offline は wasm-bindgen CLI をダウンロードできないため、
# Cargo.lock が固定するバージョンのキャッシュ済み CLI を PATH に追加し、
# 解決された CLI の --version が Cargo.lock 版と一致するか fail-closed で検査する。
setup_wasm_bindgen_path || exit 1
check_wasm_bindgen_version || exit 1

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
