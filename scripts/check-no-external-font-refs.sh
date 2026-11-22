#!/usr/bin/env bash
# web/index.html に第三者フォント参照が混入していないことを静的に検査する。
# Google Fonts の外部送信（preconnect / stylesheet）が再混入するのをゲートで防ぐ。
# 第 1 引数で検査対象ファイルを差し替えられる（負向 test 用）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
file="${1:-$ROOT_DIR/web/index.html}"

if [[ ! -f "$file" ]]; then
	echo "ERROR: $file が見つかりません" >&2
	exit 1
fi

# 一致なし (rc=1) は成功、読取不可などのエラー (rc>=2) は fail-closed とする。
grep_rc=0
grep -n -F -e 'fonts.googleapis' -e 'fonts.gstatic' -e 'preconnect' "$file" || grep_rc=$?
if ((grep_rc == 0)); then
	echo "ERROR: $file に第三者フォント参照（fonts.googleapis / fonts.gstatic / preconnect）が含まれています" >&2
	exit 1
fi
if ((grep_rc >= 2)); then
	echo "ERROR: $file の第三者フォント参照検査に失敗しました (grep rc=$grep_rc)" >&2
	exit 1
fi

echo "no-external-font-refs OK: $file"
