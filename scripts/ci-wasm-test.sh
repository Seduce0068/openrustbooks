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
	echo "wasm-pack が見つからないため $WASM_PACK_VERSION を導入します（ネットワークが必要）"
	if ! cargo install wasm-pack --locked --version "$WASM_PACK_VERSION"; then
		echo "ERROR: wasm-pack の導入に失敗しました。レジストリ到達性 / CARGO_NET_OFFLINE を確認してください" >&2
		exit 1
	fi
	# cargo install の導入先が PATH に無い CI でも検出できるようにする（CARGO_HOME 対応）。
	export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
	# 導入後も期待バージョンであることを確認し、不一致なら fail-closed とする。
	current="$(wasm-pack --version 2>/dev/null || true)"
	if ! command -v wasm-pack >/dev/null 2>&1 || ! check_wasm_pack_version; then
		echo "ERROR: 導入後の wasm-pack が $WASM_PACK_VERSION ではありません: ${current:-version 取得失敗}" >&2
		exit 1
	fi
fi

# Cargo.lock が固定する wasm-bindgen のバージョンを抽出する。
# verify.sh / build-desktop.sh と同一のロジック（抽出失敗は fail-closed）。
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

# runner の --version 出力が "wasm-bindgen-test-runner <expected>" と一致するか。
# バージョン直後が空白または行末であることを確認し、"0.2.1290" を誤許可しない。
runner_matches_locked_version() {
	local runner="$1" expected="$2" output
	[[ -x "$runner" ]] || return 1
	output="$("$runner" --version 2>/dev/null)" || return 1
	[[ "$output" == "wasm-bindgen-test-runner $expected" ||
		"$output" == "wasm-bindgen-test-runner $expected "* ]]
}

# wasm-pack は wasm-bindgen CLI と同じディレクトリに test runner を期待する。
# trunk の CLI ディレクトリには runner が無いため、Cargo.lock と同一版の
# runner がある wasm-pack キャッシュを先に PATH へ追加する（無い場合は既定のまま）。
locked_wbg_version="$(wasm_bindgen_locked_version "$ROOT_DIR/Cargo.lock")" || exit 1
for runner_dir in "$HOME"/.cache/.wasm-pack/wasm-bindgen-*; do
	if runner_matches_locked_version "$runner_dir/wasm-bindgen-test-runner" "$locked_wbg_version"; then
		export PATH="$runner_dir:$PATH"
		break
	fi
done

cd core
wasm-pack test --node --locked
