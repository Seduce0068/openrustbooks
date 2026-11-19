#!/usr/bin/env bash
# openrustbooks の全検証を一括実行する。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

usage() {
	cat <<'EOF'
Usage: ./scripts/verify.sh [--dist <dir>]

全検証ゲートを一括実行する。

Options:
  --dist <dir>  全ゲート成功時のみ、検証済み dist のコピーを
                <dir>/<short-sha>-<UTC>/ へ保持し manifest.txt を生成する。
                一時 dist は常に trap で削除され、--dist 指定時のみ別途コピーが残る。
  -h, --help    このヘルプを表示して終了する。
EOF
}

# 一時 dist は常に trap で削除する。--dist 指定時のみ、成功時に別途コピーを保持する。
RETAIN_DIST=0
DIST_ROOT=""
while (($# > 0)); do
	case "$1" in
		--dist)
			if (($# < 2)) || [[ -z "$2" ]]; then
				echo "ERROR: --dist には保存先ディレクトリが必要です" >&2
				usage >&2
				exit 2
			fi
			DIST_ROOT="$2"
			RETAIN_DIST=1
			shift 2
			;;
		-h|--help)
			usage
			exit 0
			;;
		*)
			echo "ERROR: 不正な引数: $1" >&2
			usage >&2
			exit 2
			;;
	esac
done

if [[ $RETAIN_DIST -eq 1 ]]; then
	# 絶対パスを推奨（相対パスは cwd 依存で危険なため拒否）。
	if [[ "$DIST_ROOT" != /* ]]; then
		echo "ERROR: --dist には絶対パスを指定してください: $DIST_ROOT" >&2
		usage >&2
		exit 2
	fi
	# シンボリックリンクと .. を解決し、リポジトリ配下への生成を拒否する。
	DIST_ROOT="$(readlink -m "$DIST_ROOT")"
	ROOT_CANON="$(readlink -f "$ROOT_DIR")"
	case "$DIST_ROOT" in
		"$ROOT_CANON"|"$ROOT_CANON"/*)
			echo "ERROR: --dist はリポジトリ配下（$ROOT_CANON）を指定できません: $DIST_ROOT" >&2
			usage >&2
			exit 2
			;;
	esac
fi

# E2E ブラウザ識別子（ADR-0265 / GOAL.md）。環境変数で上書き可能。
E2E_BROWSER="${E2E_BROWSER:-google-chrome}"
E2E_STATIC_PORT="${E2E_STATIC_PORT:-8099}"
E2E_CDP_PORT="${E2E_CDP_PORT:-9222}"

# 改行/制御文字が混入すると manifest の行構造を改ざんできるため、
# ブラウザ識別子・ポート・version のすべてで fail-closed に拒否する。
reject_control_chars() {
	if [[ "$1" =~ [[:cntrl:]] ]]; then
		echo "ERROR: $2 に改行/制御文字が含まれています" >&2
		exit 1
	fi
}
reject_control_chars "$E2E_BROWSER" "E2E_BROWSER"
reject_control_chars "$E2E_STATIC_PORT" "E2E_STATIC_PORT"
reject_control_chars "$E2E_CDP_PORT" "E2E_CDP_PORT"

# ポートは十進数文字列のみ許可（python3 側は type=int だが、手前で明示的に検証）。
if [[ ! "$E2E_STATIC_PORT" =~ ^[0-9]+$ ]]; then
	echo "ERROR: E2E_STATIC_PORT が不正です: $E2E_STATIC_PORT" >&2
	exit 1
fi
if [[ ! "$E2E_CDP_PORT" =~ ^[0-9]+$ ]]; then
	echo "ERROR: E2E_CDP_PORT が不正です: $E2E_CDP_PORT" >&2
	exit 1
fi

# ブラウザ version を E2E 起動前に取得し、失敗時は fail-closed で即時終了する。
# 長い E2E を走らせる前にブラウザの存在を確定させ、manifest に
# "unknown" を成功記録しない（manifest 改ざん防止）。
e2e_version_rc=0
E2E_BROWSER_VERSION="$("$E2E_BROWSER" --version 2>/dev/null)" || e2e_version_rc=$?
if ((e2e_version_rc != 0)); then
	echo "ERROR: ブラウザの version 取得に失敗しました: $E2E_BROWSER (rc=$e2e_version_rc)" >&2
	exit 1
fi
reject_control_chars "$E2E_BROWSER_VERSION" "E2E_BROWSER_VERSION"
if [[ -z "${E2E_BROWSER_VERSION//[[:space:]]/}" ]]; then
	echo "ERROR: ブラウザの version が空/空白のみです: $E2E_BROWSER" >&2
	exit 1
fi
if [[ "${E2E_BROWSER_VERSION,,}" == "unknown" ]]; then
	echo "ERROR: ブラウザの version が unknown です: $E2E_BROWSER" >&2
	exit 1
fi

# manifest には command -v で解決した実行ファイルの絶対パスも記録する。
E2E_BROWSER_PATH="$(command -v "$E2E_BROWSER" 2>/dev/null || true)"
if [[ -z "$E2E_BROWSER_PATH" ]]; then
	echo "ERROR: ブラウザ実行ファイルのパスを解決できません: $E2E_BROWSER" >&2
	exit 1
fi
reject_control_chars "$E2E_BROWSER_PATH" "E2E_BROWSER_PATH"

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

# 検証済み dist を保持先へコピーし、manifest.txt を生成する。
# 呼び出し側で全ゲート成功（fail=0）を確認してから呼ぶ。
# 既存の同名ディレクトリは決して上書きせず、連番の別名を探索する。
# ステージングへ書き出してから atomic mv するため、中途半端な成果物は残らない。
retain_verified_dist() {
	local short_sha utc_stamp base dest suffix
	short_sha="$(git rev-parse --short HEAD)"
	utc_stamp="$(date -u +'%Y%m%dT%H%M%SZ')"

	mkdir -p "$DIST_ROOT" || {
		echo "ERROR: 保持先ディレクトリを作成できません: $DIST_ROOT" >&2
		exit 1
	}

	base="${short_sha}-${utc_stamp}"
	dest="$DIST_ROOT/$base"
	suffix=1
	while [[ -e "$dest" ]]; do
		dest="$DIST_ROOT/${base}-${suffix}"
		suffix=$((suffix + 1))
		if ((suffix > 99)); then
			echo "ERROR: 保持先の候補がすべて使用中です: $DIST_ROOT/$base" >&2
			exit 1
		fi
	done

	# STAGE_DIR は EXIT trap で必ず掃除する（成功時に空へ戻して解除）。
	STAGE_DIR="${dest}.tmp.$$"
	mkdir -p "$STAGE_DIR" || {
		echo "ERROR: ステージングディレクトリを作成できません: $STAGE_DIR" >&2
		exit 1
	}
	# 一時 DIST_DIR の中身（隠しファイル含む）を属性付きでコピーする。
	if ! cp -a "$DIST_DIR"/. "$STAGE_DIR"/; then
		echo "ERROR: dist のコピーに失敗しました: $STAGE_DIR" >&2
		exit 1
	fi

	# E2E の期待件数と実際の結果を、再実行せず既存の情報から取得する。
	# 抽出不能なら "unknown" を記録したうえで失敗扱いとし、成功風の数値は書かない。
	local metadata_ok=1
	local e2e_expected e2e_result trunk_ver
	e2e_expected="$(grep -o -m1 -E 'Checker\(expected=[0-9]+\)' tools/verify-web-reader.py | grep -o -E '[0-9]+' || true)"
	if [[ -z "$e2e_expected" ]]; then
		e2e_expected="unknown"
		metadata_ok=0
	fi
	e2e_result="$(grep -m1 -E '^passed: [0-9]+, failed: [0-9]+$' "$E2E_LOG" || true)"
	if [[ -z "$e2e_result" ]]; then
		e2e_result="unknown"
		metadata_ok=0
	fi
	trunk_ver="$(trunk --version 2>/dev/null || true)"
	if [[ -z "$trunk_ver" ]]; then
		trunk_ver="unknown"
		metadata_ok=0
	fi

	# dist 内ファイルの SHA256（相対パス、決定的な並び順）。
	local sha256_block
	sha256_block="$(cd "$DIST_DIR" && find . -type f -print0 | sort -z | xargs -0 sha256sum)"

	# 一時ファイルに書き出して mv でステージング内へ配置する。
	local tmp_manifest="$STAGE_DIR/.manifest.txt.tmp"
	{
		echo "openrustbooks verified distribution manifest"
		echo "============================================"
		echo "commit sha   : $(git rev-parse HEAD)"
		echo "verified at  : $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
		echo "rustc        : $(rustc --version)"
		echo "trunk        : $trunk_ver"
		echo "e2e expected : $e2e_expected"
		echo "e2e result   : $e2e_result"
		echo "browser executable : $E2E_BROWSER"
		echo "browser path       : $E2E_BROWSER_PATH"
		echo "browser version    : $E2E_BROWSER_VERSION"
		echo "static port       : $E2E_STATIC_PORT"
		echo "cdp port          : $E2E_CDP_PORT"
		echo "source dist  : temporary DIST_DIR (removed by trap on exit)"
		echo ""
		echo "SHA256 (relative paths under dist):"
		echo "$sha256_block"
	} > "$tmp_manifest"
	mv "$tmp_manifest" "$STAGE_DIR/manifest.txt"

	if [[ $metadata_ok -eq 0 ]]; then
		echo "ERROR: manifest の必須メタデータを取得できませんでした（unknown を記録）: e2e_expected=$e2e_expected e2e_result=$e2e_result trunk=$trunk_ver" >&2
		exit 1
	fi

	# ステージングを最終ディレクトリへ原子的に配置（既存ディレクトリは上書きしない）。
	if ! mv -n -T "$STAGE_DIR" "$dest"; then
		echo "ERROR: ステージングの配置に失敗しました: $dest" >&2
		exit 1
	fi
	if [[ -e "$STAGE_DIR" ]]; then
		echo "ERROR: 配置先が競合したため保持を中止しました: $dest" >&2
		exit 1
	fi
	STAGE_DIR=""

	echo "===== 検証済み dist を保持しました: $dest ====="
}

# web/index.html に第三者フォント参照が混入していないことを静的に検査する。
# Google Fonts の外部送信（preconnect / stylesheet）が再混入するのをゲートで防ぐ。
check_no_external_font_refs() {
	local file="web/index.html"
	if [[ ! -f "$file" ]]; then
		echo "ERROR: $file が見つかりません"
		return 1
	fi

	# 一致なし (rc=1) は成功、読取不可などのエラー (rc>=2) は fail-closed とする。
	local grep_rc=0
	grep -n -F -e 'fonts.googleapis' -e 'fonts.gstatic' -e 'preconnect' "$file" || grep_rc=$?
	if ((grep_rc == 0)); then
		echo "ERROR: $file に第三者フォント参照（fonts.googleapis / fonts.gstatic / preconnect）が含まれています"
		return 1
	fi
	if ((grep_rc >= 2)); then
		echo "ERROR: $file の第三者フォント参照検査に失敗しました (grep rc=$grep_rc)"
		return 1
	fi
	return 0
}

# core/Cargo.toml の rumdl 2 行が featureless であることを検査する。
# 実体は専用スクリプトに置き、CI の extra_gates からも同じ検査を呼べるようにする。
check_rumdl_featureless() {
	bash "$SCRIPT_DIR/check-rumdl-featureless.sh"
}

run fmt cargo fmt --all -- --check
run no-external-font-refs check_no_external_font_refs
run rumdl-featureless check_rumdl_featureless
run version-consistency bash scripts/check-version-consistency.sh
run e2e-count-consistency bash scripts/check-e2e-count-consistency.sh
run csp-consistency bash scripts/check-csp-consistency.sh
run updater-honesty bash scripts/check-updater-honesty.sh
run capability-scope bash scripts/check-capability-scope.sh
run check cargo check --all-targets --locked
run test cargo test --all-targets --locked
run clippy cargo clippy --all-targets --locked -- -D warnings
run wasm-core cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked
run wasm-web cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked
# wasm-bindgen-test のテストは Node 上で実際に実行する（--no-run では不十分）。
# wasm-pack の導入とテスト実行は共通スクリプトに集約し、CI と単一情報源にする。
run wasm-test bash scripts/ci-wasm-test.sh

# 脆弱性ゲート: advisory DB をフェッチせずローカルキャッシュだけで監査する。
# -n/--no-fetch がオフライン／キャッシュ優先のオプション（git fetch を実行しない）。
# --deny warnings は意図的に使わない（unmaintained 等 10 件は既知 baseline の警告）。
# 脆弱性が 1 件でもあれば cargo audit は rc!=0 を返し、run ヘルパーがゲートを失敗させる。
# advisory DB が無い場合も cargo audit が rc!=0 を返してゲート失敗（fail-closed）。
# --no-fetch は stale 判定を無効化するため、鮮度は定期的な `cargo audit update-db` の
# 運用責任とし、--stale は指定しない（ADR-0125 / ADR-0086）。
# cargo-audit 未インストール時もスキップせず、ERROR を出してゲートを失敗させる
# （未インストールを理由に fail-open にしない）。
if command -v cargo-audit >/dev/null 2>&1; then
	run audit cargo audit --no-fetch
else
	# run ヘルパーと同じ START/END 形式で失敗を記録し、fail を立てる。
	echo "===== START: audit ====="
	echo "ERROR: cargo-audit is required; install with: cargo install cargo-audit --locked"
	echo "===== END: audit rc=127 ====="
	fail=1
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
# E2E 出力を保持して manifest の結果欄に使う（常に一時ファイルとして扱う）。
E2E_LOG="$(mktemp)"
# STAGE_DIR は retain_verified_dist が設定する。中途失敗時は EXIT trap で掃除する。
STAGE_DIR=""
trap 'rm -rf "$DIST_DIR" "$E2E_LOG"; [[ -z "$STAGE_DIR" ]] || rm -rf "$STAGE_DIR"' EXIT

echo "===== START: trunk build ====="
# 開発ホストの絶対パスを配布 WASM に残さない（rustc の正しいフラグは --remap-path-prefix）。
if (cd web && RUSTFLAGS="--remap-path-prefix=$home=~" trunk build --offline --locked --dist "$DIST_DIR"); then
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
# 出力は従来どおりストリーミング表示しつつ、manifest 用に E2E_LOG へも保存する。
# pipefail により python3 の失敗はパイプラインの終了コードへ伝播する。
if python3 tools/verify-web-reader.py --dist "$DIST_DIR" --chrome "$E2E_BROWSER" --port "$E2E_STATIC_PORT" --debug-port "$E2E_CDP_PORT" 2>&1 | tee "$E2E_LOG"; then
	echo "===== END: e2e rc=0 ====="
else
	rc=$?
	echo "===== END: e2e rc=$rc ====="
	exit "$rc"
fi

echo "===== 全検証成功 ====="

# --dist 指定時のみ、全ゲート成功（fail=0）を確認してから成果物を保持する。
# ゲート失敗時は上流で exit するため、ここに到達した時点では全ゲート成功済み。
if [[ $RETAIN_DIST -eq 1 && $fail -eq 0 ]]; then
	retain_verified_dist
fi
