#!/usr/bin/env bash
# gitleaks でリポジトリのシークレットを走査する（fail-closed）。
# ローカルは pre-commit（.pre-commit-config.yaml）を主軸とし、本スクリプトは
# verify.sh と CI の最終ゲートとして同じ検査を実行する。
# 第 1 引数で走査対象リポジトリを差し替えられる（負向 test 用）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
source_dir="${1:-$ROOT_DIR}"

if ! command -v gitleaks >/dev/null 2>&1; then
	echo "ERROR: gitleaks is required; install gitleaks (https://github.com/gitleaks/gitleaks#installing)" >&2
	exit 127
fi

echo "===== gitleaks detect --no-banner --redact (source=$source_dir) ====="
rc=0
gitleaks detect --no-banner --redact --source "$source_dir" || rc=$?
if ((rc != 0)); then
	echo "ERROR: gitleaks がシークレットを検出しました、または走査に失敗しました (rc=$rc)" >&2
	exit "$rc"
fi
echo "gitleaks: no secrets detected"
