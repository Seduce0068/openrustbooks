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
  --check-audit-db-freshness [<dir>]
                advisory DB の鮮度ゲートのみを実行する（テスト／保守用）。
                <dir> 省略時は CARGO_HOME/advisory-db を検査する。
  --check-wasm-bindgen [<lockfile>]
                wasm-bindgen ゲートのみを実行する（テスト／保守用）。
                Cargo.lock の版と CLI --version の一致を fail-closed で検査する。
                <lockfile> 省略時はリポジトリルートの Cargo.lock を使う。
EOF
}

# advisory DB の鮮度を fail-closed で検査する。
# - DB 場所: 第1引数 > AUDIT_DB_DIR > $CARGO_HOME/advisory-db
#   （CARGO_HOME 未設定なら $HOME/.cargo/advisory-db）
# - 許容日数: AUDIT_MAX_AGE_DAYS（未設定時 7、正の整数のみ、最大 90）
# - 欠落 / .git 欠落 / 時刻取得不能 / 未来 / 超過はすべて失敗（rc=1）。
check_audit_db_freshness() {
	local db_dir="${1:-${AUDIT_DB_DIR:-}}"
	# ${VAR-7} とし、未設定のみ既定化する。空文字は不正値として fail-closed にする。
	local max_days="${AUDIT_MAX_AGE_DAYS-7}"
	local cargo_home db_git now commit_ts age max_age_seconds

	if [[ -z "$db_dir" ]]; then
		if [[ -n "${CARGO_HOME:-}" ]]; then
			cargo_home="${CARGO_HOME%/}"
		else
			cargo_home="${HOME%/}/.cargo"
		fi
		db_dir="$cargo_home/advisory-db"
	fi
	# 解決済みパスを呼び出し側へ渡し、鮮度検査と cargo audit が同一 DB を指すようにする。
	AUDIT_DB_PATH="$db_dir"

	# 不正な上限は「判定不能」として fail-closed（空文字も既定へ置換しない）。
	if [[ ! "$max_days" =~ ^[1-9][0-9]*$ ]] || ((max_days > 90)); then
		echo "ERROR: AUDIT_MAX_AGE_DAYS が不正です（1〜90 の整数が必要）: ${max_days:-<empty>}" >&2
		return 1
	fi

	if [[ ! -d "$db_dir" ]]; then
		echo "ERROR: advisory DB が見つかりません: $db_dir" >&2
		return 1
	fi
	db_git="$db_dir/.git"
	if [[ ! -e "$db_git" ]]; then
		echo "ERROR: advisory DB の .git が見つかりません: $db_git" >&2
		return 1
	fi

	now="$(date +%s)" || {
		echo "ERROR: 現在時刻 (date +%s) を取得できません" >&2
		return 1
	}
	# git log の失敗・空出力は || true で吸収し、値検証で fail-closed にする。
	commit_ts="$(git -C "$db_dir" log -1 --format=%ct 2>/dev/null || true)"
	if [[ ! "$commit_ts" =~ ^[0-9]+$ ]]; then
		echo "ERROR: advisory DB の最終コミット時刻を取得できません: $db_dir" >&2
		return 1
	fi

	age=$((now - commit_ts))
	# 未来のタイムスタンプはクロック不整合の可能性があり判定不能として拒否する。
	if ((age < 0)); then
		echo "ERROR: advisory DB の最終コミット時刻が未来です: $db_dir (commit_ts=$commit_ts, now=$now)" >&2
		return 1
	fi
	max_age_seconds=$((max_days * 86400))
	if ((age > max_age_seconds)); then
		echo "ERROR: advisory DB が古すぎます: $db_dir (age=${age}s > limit=${max_age_seconds}s / ${max_days}日)" >&2
		echo "       cargo audit update-db または fetch 有効な CI での更新が必要です" >&2
		return 1
	fi
	echo "advisory DB freshness OK: $db_dir (age=${age}s / limit=${max_age_seconds}s / ${max_days}日)"
	return 0
}

# 鮮度ゲート単独の実行（負向テスト／運用確認用）。通常の全検証より先に処理する。
if [[ "${1:-}" == "--check-audit-db-freshness" ]]; then
	if check_audit_db_freshness "${2:-}"; then
		exit 0
	fi
	exit 1
fi

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
	# 絶対パス / リポジトリ外 / readlink -m による正規化は専用スクリプトに集約し、
	# CI の extra_gates と同じ判定を共有する。失敗時は従来どおり rc=2 で拒否する。
	if DIST_ROOT="$(bash "$SCRIPT_DIR/check-distribution-path.sh" --no-self-test "$DIST_ROOT")"; then
		:
	else
		rc=$?
		exit "$rc"
	fi
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

# cargo audit の警告を advisory ID + 有効期限付きで管理する（ADR-0349）。
# 未登録 ID / 期限切れ / JSON 解析失敗は fail-closed。
check_audit_allowlist() {
	local output_file="$1"
	python3 - "$output_file" <<'PY'
import json
import sys
from datetime import date
from pathlib import Path

# 既知の unmaintained / unsound 警告。新規追加時は ADR とこの一覧を更新する。
ALLOWED = {
    "RUSTSEC-2024-0370": "2027-03-31",  # proc-macro-error
    "RUSTSEC-2024-0429": "2027-03-31",  # glib
    "RUSTSEC-2024-0436": "2027-03-31",  # paste
    "RUSTSEC-2024-0449": "2027-03-31",  # unsafe-libyaml
    "RUSTSEC-2025-0141": "2027-03-31",  # bincode
    "RUSTSEC-2026-0543": "2027-03-31",  # chrono
}

path = Path(sys.argv[1])
try:
    report = json.loads(path.read_text(encoding="utf-8"))
except (OSError, UnicodeDecodeError, ValueError) as exc:
    print(f"ERROR: cargo audit の JSON を解釈できません: {exc}", file=sys.stderr)
    sys.exit(1)

errors: list[str] = []
for group in ("vulnerabilities", "warnings"):
    section = report.get(group)
    if not isinstance(section, dict):
        errors.append(f"cargo audit JSON の {group} が辞書ではありません")
        continue
    for item in section.get("list", []):
        if group == "vulnerabilities":
            errors.append(
                f"脆弱性が検出されました: {item.get('id', 'unknown')} "
                f"({item.get('package', {}).get('name', 'unknown')})"
            )
            continue
        advisory = item.get("advisory", {})
        advisory_id = advisory.get("id", "")
        if advisory_id not in ALLOWED:
            errors.append(f"未登録の advisory 警告: {advisory_id or 'unknown'}")
            continue
        if date.today().isoformat() > ALLOWED[advisory_id]:
            errors.append(f"advisory 例外の有効期限切れ: {advisory_id} ({ALLOWED[advisory_id]})")

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

allowed = ", ".join(sorted(ALLOWED))
print(f"OK: audit 警告は許可リスト内のみ ({len(ALLOWED)} IDs, 期限 2027-03-31)")
PY
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

	# manifest 形式（必須キー / e2e expected=<コード出典> / failed:0 / SHA256 行）を
	# CI と共通のスクリプトで検査する。不合格なら保持を行わず hard fail する。
	if ! bash "$SCRIPT_DIR/check-distribution-path.sh" --no-self-test "$DIST_ROOT" "$STAGE_DIR/manifest.txt"; then
		echo "ERROR: manifest 形式検査に失敗しました: $STAGE_DIR/manifest.txt" >&2
		exit 1
	fi

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

# web/index.html の第三者フォント参照を検査する。
# 実体は専用スクリプトに置き、CI の extra_gates からも同じ検査を呼べるようにする。
check_no_external_font_refs() {
	bash "$SCRIPT_DIR/check-no-external-font-refs.sh"
}

# シークレット走査（Betterleaks）。ローカルは pre-commit を主軸とし、
# 本ゲートと CI の extra_gates が最終ゲートとして fail-closed で検査する。
check_secrets() {
	bash "$SCRIPT_DIR/check-secrets.sh"
}

# core/Cargo.toml の rumdl 2 行が featureless であることを検査する。
# 実体は専用スクリプトに置き、CI の extra_gates からも同じ検査を呼べるようにする。
check_rumdl_featureless() {
	bash "$SCRIPT_DIR/check-rumdl-featureless.sh"
}

# serde_yaml / tikv-jemalloc-sys の lock 残存を既知例外（ID / 期限 / 経路）なしに
# 通さない。rumdl featureless を前提とするため、直前に並べて実行する。
check_dependency_hygiene() {
	bash "$SCRIPT_DIR/check-dependency-hygiene.sh"
}

run fmt cargo fmt --all -- --check
run no-external-font-refs check_no_external_font_refs
run secrets check_secrets
run rumdl-featureless check_rumdl_featureless
run dependency-hygiene check_dependency_hygiene
run version-consistency bash scripts/check-version-consistency.sh
run e2e-count-consistency bash scripts/check-e2e-count-consistency.sh
run csp-consistency bash scripts/check-csp-consistency.sh
run tauri-plugin-config bash scripts/check-tauri-plugin-config.sh
run updater-honesty bash scripts/check-updater-honesty.sh
run capability-scope bash scripts/check-capability-scope.sh
run config-dto-contract bash scripts/check-config-dto-contract.sh
run check cargo check --all-targets --locked
run test cargo test --all-targets --locked
run clippy cargo clippy --all-targets --locked -- -D warnings -D clippy::undocumented_unsafe_blocks
run wasm-core cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked
run wasm-web cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked
# wasm-bindgen-test のテストは Node 上で実際に実行する（--no-run では不十分）。
# wasm-pack の導入とテスト実行は共通スクリプトに集約し、CI と単一情報源にする。
run wasm-test bash scripts/ci-wasm-test.sh

# advisory DB の鮮度ゲート（cargo audit より先に実行する）。
# --no-fetch は stale 判定を無効化するため、鮮度は自前で検査する（ADR-0125 / ADR-0086）。
# 鮮度不合格時は cargo audit を実行せず、ゲート失敗を確定させる（fail-closed）。
# audit_freshness_ok は前段ゲートの fail 状態に影響されない専用の成否フラグ。
AUDIT_DB_PATH=""
audit_freshness_ok=0
echo "===== START: audit-db-freshness ====="
if check_audit_db_freshness; then
	echo "===== END: audit-db-freshness rc=0 ====="
	audit_freshness_ok=1
else
	audit_freshness_rc=$?
	echo "===== END: audit-db-freshness rc=$audit_freshness_rc ====="
	audit_freshness_ok=0
	fail=1
fi
if ((audit_freshness_ok == 1)); then
	# 脆弱性ゲート: advisory DB をフェッチせずローカルキャッシュだけで監査する。
	# -n/--no-fetch がオフライン／キャッシュ優先のオプション（git fetch を実行しない）。
	# --deny warnings は意図的に使わない（unmaintained 等 10 件は既知 baseline の警告）。
	# 脆弱性が 1 件でもあれば cargo audit は rc!=0 を返し、run ヘルパーがゲートを失敗させる。
	# advisory DB が無い場合も cargo audit が rc!=0 を返してゲート失敗（fail-closed）。
	# --stale は指定しない（鮮度は上のゲートで担保するため）。
	# CI 向け: 新鮮な advisory DB を得る目的では fetch 有効の `cargo audit`（または
	# `cargo audit update-db`）の利用を推奨する。ただし verify.sh はオフライン再現性を
	# 優先して --no-fetch を維持し、鮮度ゲートでキャッシュの古さを検出する。
	# 共通 workflow（Seduce0068/workflows）は変更しない。
	# cargo-audit 未インストール時もスキップせず、ERROR を出してゲートを失敗させる
	# （未インストールを理由に fail-open にしない）。
	if command -v cargo-audit >/dev/null 2>&1; then
		# 鮮度検査で解決した DB を明示渡しし、検査対象と監査対象を一致させる。
		# --deny warnings は使わず、ID + 期限付き allowlist で新規警告を拒否する。
		audit_json="$(mktemp)"
		echo "===== START: audit ====="
		if cargo audit --no-fetch --db "$AUDIT_DB_PATH" --json >"$audit_json" 2>/tmp/audit-stderr.$$; then
			if check_audit_allowlist "$audit_json"; then
				echo "===== END: audit rc=0 ====="
			else
				echo "===== END: audit rc=1 ====="
				fail=1
			fi
		else
			audit_rc=$?
			cat /tmp/audit-stderr.$$ >&2
			echo "===== END: audit rc=$audit_rc ====="
			fail=1
		fi
		rm -f "$audit_json" /tmp/audit-stderr.$$
	else
		# run ヘルパーと同じ START/END 形式で失敗を記録し、fail を立てる。
		echo "===== START: audit ====="
		echo "ERROR: cargo-audit is required; install with: cargo install cargo-audit --locked"
		echo "===== END: audit rc=127 ====="
		fail=1
	fi
else
	echo "ERROR: advisory DB の鮮度が不合格のため cargo audit をスキップします (db=$AUDIT_DB_PATH)" >&2
fi

if [[ $fail -ne 0 ]]; then
	echo "===== ゲート失敗: E2E は実行しません ====="
	exit 1
fi

# Cargo.lock 版の wasm-bindgen CLI を PATH へ追加し、解決された CLI の
# --version が Cargo.lock と一致することを fail-closed で確認してから trunk build する。
setup_wasm_bindgen_path || exit 1
check_wasm_bindgen_version || exit 1
# wasm-bindgen-test-runner の PATH 解決は scripts/ci-wasm-test.sh に集約する。

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

# 一時 DIST は E2E 専用で web/dist を更新しないため、ここで現行 frontend を
# web/dist へ生成し直し、desktop バイナリがそれを埋め込めることを確認する。
# 先の fail 判定は通過済みなので、失敗は fail-open にせず即時終了する。
echo "===== START: build-desktop ====="
if bash scripts/build-desktop.sh; then
	echo "===== END: build-desktop rc=0 ====="
else
	rc=$?
	echo "===== END: build-desktop rc=$rc ====="
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

echo "===== START: wdio ====="
if bash scripts/ci-wdio-test.sh; then
	echo "===== END: wdio rc=0 ====="
else
	rc=$?
	echo "===== END: wdio rc=$rc ====="
	exit "$rc"
fi

echo "===== 全検証成功 ====="

# --dist 指定時のみ、全ゲート成功（fail=0）を確認してから成果物を保持する。
# ゲート失敗時は上流で exit するため、ここに到達した時点では全ゲート成功済み。
if [[ $RETAIN_DIST -eq 1 && $fail -eq 0 ]]; then
	retain_verified_dist
fi
