#!/usr/bin/env bash
# Tauri config に無効な plugin scope キーが再混入していないことを検査する（fail-closed）。
# Tauri 2.12.5 の fs / shell plugin config は deny_unknown_fields で未知キーを拒否し、
# dialog には config 型が無いため、scope / allowedPaths は無効キー（R6 R2 / GOAL.md）。
# 今回の契約は plugins ブロックの完全削除と未使用 plugin 登録の削除であり、
# 静的ゲートで再混入を防ぐ。
# window/mod.rs への global shortcut 再混入も検出する。
# 無効キーは JSON キーとして検出し、正当な文字列との部分一致による誤検知を避ける。
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
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
errors: list[str] = []

CONF_REL = "desktop/tauri.conf.json"
MAIN_REL = "desktop/src/main.rs"
PLUGIN_NAMES = ("fs", "dialog", "shell")
INVALID_KEYS = ("scope", "allowedPaths")

# 必須 init は log / dialog のみ。log は Builder パターンで初期化するため
# `init()` ではなく `Builder::new()` を検査する。
# wdio は debug + feature ゲートの別経路（E2E / feature build）で担保し、
# 本ゲートの必須一覧には含めない。
PLUGIN_INITS = (
    "tauri_plugin_log::Builder::new()",
    "tauri_plugin_dialog::init()",
)

# 未使用 plugin の再混入を fail-closed で検出する（負向 test の更新）。
REMOVED_PLUGIN_PREFIXES = (
    "tauri_plugin_store::",
    "tauri_plugin_global_shortcut::",
    "tauri_plugin_shell::",
    "tauri_plugin_fs::",
    "tauri_plugin_opener::",
    "tauri_plugin_notification::",
)

# --- desktop/tauri.conf.json: 欠落・不正 JSON は fail-closed ---------------------
conf_path = root / CONF_REL
conf_text: str | None = None
try:
    conf_text = conf_path.read_text(encoding="utf-8")
except (OSError, UnicodeDecodeError) as exc:
    errors.append(f"{CONF_REL} を読み込めません: {exc}")

conf: dict | None = None
if conf_text is not None:
    try:
        with conf_path.open(encoding="utf-8") as f:
            loaded = json.load(f)
    except (OSError, UnicodeDecodeError, ValueError) as exc:
        errors.append(f"{CONF_REL} の JSON 読み込み/パースに失敗しました: {exc}")
    else:
        if not isinstance(loaded, dict):
            errors.append(f"{CONF_REL} のトップレベルは JSON オブジェクトである必要があります")
        else:
            conf = loaded

# --- 完全削除を要求: トップレベルに plugins キーが無いこと -----------------------
if conf is not None:
    if "plugins" in conf:
        errors.append(f"{CONF_REL} にトップレベル plugins キーが存在します（完全削除が必要）")
        plugins = conf["plugins"]
        if isinstance(plugins, dict):
            found = [name for name in PLUGIN_NAMES if name in plugins]
            if found:
                errors.append(
                    f"{CONF_REL} の plugins 配下に plugin 設定があります: {', '.join(found)}"
                )

# --- 既知無効キーの再混入を検出 -------------------------------------------------
# 部分文字列ではなく JSON のキーとして検出する。説明文などの正当な文字列に
# "scope" が含まれていても誤検知しない（部分文字列マッチの誤検知指摘対応）。
INVALID_KEYS_LOWER = {key.lower() for key in INVALID_KEYS}


def walk_keys(node: object, path: str) -> None:
    if isinstance(node, dict):
        for key, value in node.items():
            if isinstance(key, str) and key.lower() in INVALID_KEYS_LOWER:
                errors.append(f"{CONF_REL} に既知無効キー {key!r} が残っています ({path})")
            walk_keys(value, f"{path}.{key}")
    elif isinstance(node, list):
        for index, value in enumerate(node):
            walk_keys(value, f"{path}[{index}]")


if conf is not None:
    walk_keys(conf, "$")

# JSON パースが失敗していても、無効キーがキーとして書かれていれば検出する。
KEY_TOKEN_RE = re.compile(r"""["']\s*(scope|allowedPaths)\s*["']\s*:""", re.IGNORECASE)
if conf is None and conf_text is not None:
    if KEY_TOKEN_RE.search(conf_text) is not None:
        errors.append(f"{CONF_REL} に既知無効キー scope / allowedPaths がキーとして残っています")

# --- plugin 初期化が維持されていること（GOAL.md 必須事項 2） ---------------------
main_path = root / MAIN_REL
try:
    main_rs = main_path.read_text(encoding="utf-8")
except (OSError, UnicodeDecodeError) as exc:
    errors.append(f"{MAIN_REL} を読み込めません: {exc}")
else:
    for init_call in PLUGIN_INITS:
        if init_call not in main_rs:
            errors.append(f"{MAIN_REL} に plugin 初期化がありません: {init_call}")
    for prefix in REMOVED_PLUGIN_PREFIXES:
        if prefix in main_rs:
            errors.append(f"{MAIN_REL} に削除済み plugin が再混入しています: {prefix}")

# --- window/mod.rs: global shortcut の再混入を検出（ADR-0303） ---------------------
WINDOW_REL = "desktop/src/window/mod.rs"
SHORTCUT_RE = re.compile(r"\bglobal_shortcut\b", re.IGNORECASE)

window_path = root / WINDOW_REL
try:
    window_rs = window_path.read_text(encoding="utf-8")
except (OSError, UnicodeDecodeError) as exc:
    errors.append(f"{WINDOW_REL} を読み込めません: {exc}")
else:
    # コメント / 文字列 supernumerary の記述では誤検知しないよう、行コメントを除外する。
    for line in window_rs.splitlines():
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*"):
            continue
        if SHORTCUT_RE.search(line) is not None:
            errors.append(f"{WINDOW_REL} に global_shortcut が再混入しています")
            break

# --- capability JSON: wdio permission が test 用ファイルに閉じていること --------
CAP_REL = "desktop/capabilities/wdio/wdio.json"
cap_path = root / CAP_REL
cap_text: str | None = None
try:
    cap_text = cap_path.read_text(encoding="utf-8")
except (OSError, UnicodeDecodeError) as exc:
    errors.append(f"{CAP_REL} を読み込めません: {exc}")
else:
    try:
        cap = json.loads(cap_text)
    except ValueError as exc:
        errors.append(f"{CAP_REL} の JSON パースに失敗しました: {exc}")
    else:
        if not isinstance(cap, dict):
            errors.append(f"{CAP_REL} のトップレベルは JSON オブジェクトである必要があります")
        else:
            if cap.get("identifier") != "wdio":
                errors.append(f"{CAP_REL} の identifier は wdio 固定です: {cap.get('identifier')!r}")
            permissions = cap.get("permissions")
            if not isinstance(permissions, list) or not all(isinstance(x, str) for x in permissions):
                errors.append(f"{CAP_REL} の permissions は文字列配列である必要があります")
            else:
                if "wdio:default" not in permissions or "wdio-webdriver:default" not in permissions:
                    errors.append(f"{CAP_REL} に WDIO 用 permission がありません")
                if "core:default" not in permissions:
                    errors.append(f"{CAP_REL} に core:default がありません")
            # 本番 build は build.rs が default capability のみを読む。
            # wdio capability が他有効 Capability へ混入しないよう、既知の
            # test permission が tauri.conf.json に出ていないことを検査する。
            if conf_text is not None:
                for marker in ("wdio:default", "wdio-webdriver:default"):
                    if marker in conf_text:
                        errors.append(
                            f"{CONF_REL} に test 用 permission {marker} が混入しています"
                        )

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print(
    "OK: tauri.conf.json に plugins / scope / allowedPaths はなく、"
    "log / dialog の初期化は維持され、未使用 plugin の再混入はありません。"
    "wdio capability は test 用ファイルに閉じられています"
)
PY
