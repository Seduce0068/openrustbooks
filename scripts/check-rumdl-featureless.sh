#!/usr/bin/env bash
# core/Cargo.toml の rumdl 依存 2 行（native / wasm）が featureless であることを厳密検査する。
# Cargo.lock には feature が記録されないため、default-features = false はここが唯一の担保。
# 第 1 引数で検査対象ファイルを差し替えられる（negative test 用）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
file="${1:-$SCRIPT_DIR/../core/Cargo.toml}"

if [[ ! -f "$file" ]]; then
	echo "ERROR: $file が見つかりません"
	exit 1
fi

expected='rumdl = { version = "0.3", default-features = false }'

# rumdl の依存宣言行（行頭の空白後に rumdl = ...）だけを対象にする。
# rumdl に言及するコメント行は宣言ではないので対象外。
# 期待と異なる宣言行は行番号付きで報告し、期待行が 2 行でなければ失敗する。
awk -v file="$file" -v expected="$expected" '
	/^[[:space:]]*rumdl[[:space:]]*=/ {
		decl_count++
		if ($0 == expected) {
			expected_count++
		} else {
			printf "ERROR: %s:%d の rumdl 宣言が期待と異なります: %s\n", file, NR, $0
		}
	}
	END {
		if (expected_count != 2) {
			printf "ERROR: %s の rumdl featureless 宣言は 2 行必要です（実際: %d 行）\n", file, expected_count
		}
		if (expected_count != 2 || decl_count != expected_count) {
			exit 1
		}
	}
' "$file"
