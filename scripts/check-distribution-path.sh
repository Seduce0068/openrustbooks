#!/usr/bin/env bash
# 検証済み dist の保持先が「リポジトリ外の絶対パス」であることを fail-closed で検査する。
# verify.sh の --dist 検証と CI の extra_gates で同じ判定を共有する（単一情報源）。
#
# 第 1 引数省略時は DIST_ROOT（未設定なら既定の保持先）を検査する。
# 第 2 引数に manifest.txt を渡すと、必須キー / e2e expected=256 / failed:0 /
# SHA256 行形式も検査する。
# 成功時は readlink -m で正規化した絶対パスを stdout へ出力する。
# 失敗時はパス不正が rc=2、manifest 不正が rc=1。
#
# 本スクリプトは毎回「拒否されるべきパスが実際に拒否されること」と
# 「manifest 形式検査が正負両方で機能すること」の自己検査を先に実行する。
# これにより CI の無引数呼び出しが恒真なゲートにならないことを担保する。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEFAULT_DIST_ROOT="/home/koppe/.local/share/openrustbooks-verified"

# パス検証本体。成功時に正規化パスを stdout、失敗時に rc=2 を返す。
check_path() {
	local candidate="$1"
	local resolved root_canon

	if [[ -z "$candidate" ]]; then
		echo "ERROR: 配布保持先が空です" >&2
		return 2
	fi
	# 相対パスは cwd 依存で危険なため拒否（従来の --dist 検証と同じ）。
	if [[ "$candidate" != /* ]]; then
		echo "ERROR: 配布保持先には絶対パスを指定してください: $candidate" >&2
		return 2
	fi

	# シンボリックリンクと .. を解決し、リポジトリ配下への生成を拒否する。
	resolved="$(readlink -m "$candidate")" || {
		echo "ERROR: 配布保持先を正規化できません: $candidate" >&2
		return 2
	}
	root_canon="$(readlink -f "$ROOT_DIR")" || {
		echo "ERROR: リポジトリルートを正規化できません: $ROOT_DIR" >&2
		return 2
	}
	case "$resolved" in
	"$root_canon" | "$root_canon"/*)
		echo "ERROR: 配布保持先はリポジトリ配下（$root_canon）を指定できません: $resolved" >&2
		return 2
		;;
	esac
	printf '%s\n' "$resolved"
}

# manifest 形式検査。必須キー、期待値、SHA256 行をすべて満たす場合のみ成功。
check_manifest() {
	local manifest="$1"
	local key sha_lines

	if [[ ! -f "$manifest" ]]; then
		echo "ERROR: manifest が見つかりません: $manifest" >&2
		return 1
	fi

	local required_keys=(
		"commit sha" "verified at" "rustc" "trunk" "e2e expected" "e2e result"
		"browser executable" "browser path" "browser version"
		"static port" "cdp port" "source dist"
	)
	for key in "${required_keys[@]}"; do
		if ! grep -q -E "^${key}[[:space:]]*:" "$manifest"; then
			echo "ERROR: manifest に必須キーがありません: $key ($manifest)" >&2
			return 1
		fi
	done

	if ! grep -q -F 'SHA256 (relative paths under dist):' "$manifest"; then
		echo "ERROR: manifest に SHA256 セクションがありません: $manifest" >&2
		return 1
	fi
	if ! grep -q -E '^e2e expected[[:space:]]*:[[:space:]]*256[[:space:]]*$' "$manifest"; then
		echo "ERROR: manifest の e2e expected が 256 ではありません: $manifest" >&2
		return 1
	fi
	# passed: 0 は「1件も実行していない」成功風の記録なので拒否する。
	# 期待件数（現行契約 256）は ADR 側で管理し、変更時は同じ出典へ追従して更新する。
	if ! grep -q -E '^e2e result[[:space:]]*:[[:space:]]*passed:[[:space:]]*[1-9][0-9]*,[[:space:]]*failed:[[:space:]]*0[[:space:]]*$' "$manifest"; then
		echo "ERROR: manifest の e2e result が passed>=1 / failed: 0 ではありません: $manifest" >&2
		return 1
	fi

	sha_lines="$(grep -c -E '^[0-9a-f]{64}  \./' "$manifest" || true)"
	if [[ ! "$sha_lines" =~ ^[0-9]+$ ]] || ((sha_lines < 1)); then
		echo "ERROR: manifest に有効な SHA256 行がありません: $manifest" >&2
		return 1
	fi
	echo "manifest format OK: $manifest"
}

# 拒否ケースが rc=2 で拒否されることを確認する。
expect_path_reject() {
	local candidate="$1" rc=0
	check_path "$candidate" >/dev/null 2>&1 || rc=$?
	if ((rc != 2)); then
		echo "ERROR: 自己検査失敗: 拒否されるべきパスが rc=$rc でした: $candidate" >&2
		return 1
	fi
}

# 自己検査用の正常 manifest（verify.sh の生成形式に合わせた最小 fixture）。
write_manifest_fixture() {
	cat >"$1" <<'EOF'
openrustbooks verified distribution manifest
============================================
commit sha   : 0123456789abcdef0123456789abcdef01234567
verified at  : 2026-11-22T01:02:03Z
rustc        : rustc 1.95.0
trunk        : trunk 0.22.1
e2e expected : 256
e2e result   : passed: 256, failed: 0
browser executable : google-chrome
browser path       : /usr/bin/google-chrome
browser version    : Google Chrome 1.2.3
static port       : 8099
cdp port          : 9222
source dist  : temporary DIST_DIR (removed by trap on exit)

SHA256 (relative paths under dist):
0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  ./index.html
EOF
}

# 経路と manifest 形式の正負を確認する自己検査（毎回実行）。
self_test() {
	local tmp fixture

	expect_path_reject "$ROOT_DIR" || return 1
	expect_path_reject "relative-dist" || return 1
	expect_path_reject "$ROOT_DIR/../$(basename "$ROOT_DIR")/dist" || return 1

	tmp="$(mktemp -d)" || return 1
	# repo 内を指すシンボリックリンクは readlink -m の解決後に拒否されるべき。
	if ! ln -s "$ROOT_DIR" "$tmp/inside"; then
		rm -rf "$tmp"
		return 1
	fi
	if ! expect_path_reject "$tmp/inside"; then
		rm -rf "$tmp"
		return 1
	fi
	# repo 外の一時ディレクトリ自体は受け入れられる（正の制御）。
	if ! check_path "$tmp" >/dev/null; then
		echo "ERROR: 自己検査失敗: repo 外の一時ディレクトリが拒否されました" >&2
		rm -rf "$tmp"
		return 1
	fi

	fixture="$tmp/manifest.txt"
	write_manifest_fixture "$fixture"
	if ! check_manifest "$fixture" >/dev/null; then
		echo "ERROR: 自己検査失敗: 正常 fixture が拒否されました" >&2
		rm -rf "$tmp"
		return 1
	fi
	# 期待値 256 の変更、passed: 0、failed: 1、SHA256 行の欠落はすべて拒否されるべき。
	sed 's/^e2e expected[[:space:]]*:.*/e2e expected : 253/' "$fixture" >"$tmp/bad-expected.txt"
	if check_manifest "$tmp/bad-expected.txt" >/dev/null 2>&1; then
		echo "ERROR: 自己検査失敗: e2e expected=253 が受け入れられました" >&2
		rm -rf "$tmp"
		return 1
	fi
	sed 's/passed: 256/passed: 0/' "$fixture" >"$tmp/bad-passed.txt"
	if check_manifest "$tmp/bad-passed.txt" >/dev/null 2>&1; then
		echo "ERROR: 自己検査失敗: passed: 0 が受け入れられました" >&2
		rm -rf "$tmp"
		return 1
	fi
	sed 's/failed: 0/failed: 1/' "$fixture" >"$tmp/bad-failed.txt"
	if check_manifest "$tmp/bad-failed.txt" >/dev/null 2>&1; then
		echo "ERROR: 自己検査失敗: failed: 1 が受け入れられました" >&2
		rm -rf "$tmp"
		return 1
	fi
	grep -v -E '^[0-9a-f]{64}  \./' "$fixture" >"$tmp/bad-sha.txt" || true
	if check_manifest "$tmp/bad-sha.txt" >/dev/null 2>&1; then
		echo "ERROR: 自己検査失敗: SHA256 行欠落が受け入れられました" >&2
		rm -rf "$tmp"
		return 1
	fi

	rm -rf "$tmp"
	echo "distribution-path self-test OK"
}

# 内部呼び出し（自己検査内）以外は常に自己検査を実行する。
if [[ "${1:-}" != "--no-self-test" ]]; then
	self_test
else
	shift
fi

dist_root="${1:-${DIST_ROOT:-$DEFAULT_DIST_ROOT}}"
resolved="$(check_path "$dist_root")" || exit $?
if (($# >= 2)); then
	check_manifest "$2" || exit $?
fi
printf '%s\n' "$resolved"
