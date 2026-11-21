#!/usr/bin/env bash
# desktop/tauri.conf.json の build.frontendDist が想定の web/dist を指し、
# 生成物（index.html / *_bg.wasm / *.js）が揃っていることを静的に検査する（fail-closed）。
# 第 1 引数で検査対象の repo root を差し替えられる（負向 test 用）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT="${1:-$DEFAULT_ROOT}"

if [[ ! -d "$ROOT" ]]; then
	echo "ERROR: repo root がディレクトリではありません: $ROOT" >&2
	exit 1
fi

python3 - "$ROOT" <<'PY'
import json
import os
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
errors: list[str] = []

CONF_REL = "desktop/tauri.conf.json"
EXPECTED_REL = "web/dist"
conf_path = root / CONF_REL

# --- tauri.conf.json の読み込み（欠落・不正 JSON は fail-closed） ---------------
try:
    with conf_path.open(encoding="utf-8") as f:
        conf = json.load(f)
except (OSError, UnicodeDecodeError, ValueError) as exc:
    errors.append(f"{CONF_REL} の読み込み/パースに失敗しました: {exc}")
    conf = None

frontend_dist_entries: list[str] = []
if conf is not None:
    if not isinstance(conf, dict):
        errors.append(f"{CONF_REL} のトップレベルは JSON オブジェクトである必要があります")
    else:
        build = conf.get("build")
        if not isinstance(build, dict):
            errors.append(f"{CONF_REL} に build オブジェクトがありません")
        else:
            frontend_dist = build.get("frontendDist")
            if frontend_dist is None:
                errors.append(f"{CONF_REL} に build.frontendDist がありません")
            elif isinstance(frontend_dist, str):
                frontend_dist_entries = [frontend_dist]
            elif isinstance(frontend_dist, list) and frontend_dist:
                # Tauri 2 は frontendDist に string[] も許すが、生成物は単一 dist のみ。
                if all(isinstance(entry, str) and entry for entry in frontend_dist):
                    frontend_dist_entries = frontend_dist
                else:
                    errors.append(
                        f"{CONF_REL} の build.frontendDist 配列に空/非文字列要素があります: {frontend_dist!r}"
                    )
            else:
                errors.append(
                    f"{CONF_REL} の build.frontendDist は文字列（または非空の文字列配列）"
                    f"である必要があります: {frontend_dist!r}"
                )

# --- 相対パスは desktop/tauri.conf.json を基準に解決し、想定先と厳密に照合 -------
expected = (root / EXPECTED_REL).resolve()
for entry in frontend_dist_entries:
    resolved = (conf_path.parent / entry).resolve()
    if resolved != expected:
        errors.append(
            f"{CONF_REL} の build.frontendDist={entry!r} は想定の {EXPECTED_REL} を指していません "
            f"(resolved={resolved}, expected={expected})"
        )

# --- 生成物の存在検査（欠落時は fail-closed） ----------------------------------
dist = expected
if not dist.is_dir():
    errors.append(f"{EXPECTED_REL} がディレクトリではありません: {dist}")
else:
    index_file = dist / "index.html"
    if not index_file.is_file():
        errors.append(f"{EXPECTED_REL}/index.html がありません")

    wasm_files = [p for p in dist.glob("*_bg.wasm") if p.is_file()]
    if not wasm_files:
        errors.append(f"{EXPECTED_REL}/*_bg.wasm がありません")

    js_files = [p for p in dist.glob("*.js") if p.is_file()]
    if not js_files:
        errors.append(f"{EXPECTED_REL}/*.js がありません")

    # --- 開発ホストの絶対パス混入を検査（verify.sh の wasm path check と同等） ----
    # バイナリをチャンク読みし、チャンク境界をまたぐ一致も見逃さない。
    home = os.environ.get("HOME", "").rstrip("/")
    if not home:
        errors.append("HOME が空のため開発ホストのパス検査を実行できません")
    else:
        needle = (home + "/").encode("utf-8", "surrogateescape")
        for path in wasm_files + js_files:
            try:
                with path.open("rb") as f:
                    tail = b""
                    while True:
                        chunk = f.read(1 << 20)
                        if not chunk:
                            break
                        if needle in tail + chunk:
                            errors.append(
                                f"{path.relative_to(root)} に開発ホストの絶対パスが含まれています"
                            )
                            break
                        tail = chunk[-(len(needle) - 1) :]
            except OSError as exc:
                errors.append(f"{path.relative_to(root)} のパス検査に失敗しました: {exc}")

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print(
    f"OK: frontendDist は {EXPECTED_REL} を指し、"
    "index.html / wasm / JS グルーが存在します"
)
PY
