#!/usr/bin/env bash
# desktop/tauri.conf.json にトップレベル version キーが無いことを検査する。
# Tauri 2 は version 省略時に Cargo のバージョンへフォールバックするため、
# Cargo workspace を単一情報源とし、JSON 側の固定値（二重管理）を禁じる。
# 第 1 引数で検査対象ファイルを差し替えられる（negative test 用）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
file="${1:-$ROOT_DIR/desktop/tauri.conf.json}"

# ファイル欠落は fail-closed。
if [[ ! -f "$file" ]]; then
	echo "ERROR: $file が見つかりません" >&2
	exit 1
fi

# Cargo workspace のバージョンを単一情報源として取得する（失敗時は fail-closed）。
# --locked により Cargo.lock の更新を許さない。
metadata=""
if ! metadata="$(cd "$ROOT_DIR" && cargo metadata --no-deps --locked --format-version 1)"; then
	echo "ERROR: cargo metadata の実行に失敗しました" >&2
	exit 1
fi

# JSON パースと version キー検査は Python stdlib で行う（jq 非依存）。
# - パース失敗は fail-closed。
# - トップレベルに version キーが存在したら ERROR（Cargo 単一情報源の違反）。
# - バージョン値の一致は検査しない（省略を強制するため）。
METADATA_JSON="$metadata" python3 - "$file" <<'PY'
import json
import os
import sys

conf_path = sys.argv[1]
metadata_raw = os.environ["METADATA_JSON"]

try:
    with open(conf_path, encoding="utf-8") as f:
        conf = json.load(f)
except (OSError, UnicodeDecodeError, ValueError) as exc:
    print(f"ERROR: {conf_path} の JSON パースに失敗しました: {exc}", file=sys.stderr)
    sys.exit(1)

if not isinstance(conf, dict):
    print(f"ERROR: {conf_path} のトップレベルは JSON オブジェクトである必要があります", file=sys.stderr)
    sys.exit(1)

if "version" in conf:
    print(
        f"ERROR: {conf_path} にトップレベル version キーが存在します"
        f"（Cargo 単一情報源の違反）: {conf['version']!r}",
        file=sys.stderr,
    )
    sys.exit(1)

try:
    metadata = json.loads(metadata_raw)
except ValueError as exc:
    print(f"ERROR: cargo metadata の JSON パースに失敗しました: {exc}", file=sys.stderr)
    sys.exit(1)

packages = metadata.get("packages")
if not isinstance(packages, list):
    print("ERROR: cargo metadata に packages がありません", file=sys.stderr)
    sys.exit(1)

desktop_versions = [
    p.get("version")
    for p in packages
    if isinstance(p, dict) and p.get("name") == "openrustbooks-desktop"
]
if len(desktop_versions) != 1 or not isinstance(desktop_versions[0], str):
    print(
        "ERROR: Cargo workspace から openrustbooks-desktop のバージョンを取得できませんでした",
        file=sys.stderr,
    )
    sys.exit(1)

print(
    f"OK: {conf_path} は version を省略"
    f"（Cargo workspace version = {desktop_versions[0]} にフォールバック）"
)
PY
