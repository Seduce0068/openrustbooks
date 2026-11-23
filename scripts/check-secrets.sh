#!/usr/bin/env bash
# Betterleaks でリポジトリのシークレットを走査する（fail-closed）。
# ローカルは pre-commit（.pre-commit-config.yaml）を主軸とし、本スクリプトは
# verify.sh と CI の最終ゲートとして同じ検査を実行する。
# 第 1 引数で走査対象リポジトリを差し替えられる（負向 test 用）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
source_dir="${1:-$ROOT_DIR}"

if ! command -v betterleaks >/dev/null 2>&1; then
	echo "ERROR: betterleaks is required; install it with 'bash scripts/install-betterleaks.sh' (see https://github.com/betterleaks/betterleaks)" >&2
	exit 127
fi

if [ ! -d "$source_dir" ]; then
	echo "ERROR: 走査対象ディレクトリが存在しません: $source_dir" >&2
	exit 2
fi

config_file="$ROOT_DIR/.betterleaks.toml"
if [ ! -f "$config_file" ]; then
	echo "ERROR: Betterleaks 設定がありません: $config_file" >&2
	exit 2
fi
if ! betterleaks config check -c "$config_file" >/dev/null 2>&1; then
	echo "ERROR: Betterleaks 設定が不正です: $config_file" >&2
	exit 2
fi

echo "===== betterleaks git --config .betterleaks.toml --redact --verbose (source=$source_dir) ====="
# .betterleaksignore の fingerprint 例外も明示渡しする（cwd 非依存）。
# 設定側の Expr filter が path + rule + test 行に限定した false positive 除外を担当する。
rc=0
betterleaks git "$source_dir" --config "$config_file" --redact --verbose --gitleaks-ignore-path "$source_dir" || rc=$?
if ((rc != 0)); then
	echo "ERROR: Betterleaks がシークレットを検出しました、または走査に失敗しました (rc=$rc)" >&2
	exit "$rc"
fi
echo "betterleaks: no secrets detected"
