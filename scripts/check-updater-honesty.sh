#!/usr/bin/env bash
# updater 死設定の正直表示と保存時 fail-closed を静的に検査する（fail-closed）。
# 出典はコードのみ（ADR-0222 / ADR-0277 / GOAL.md「R6 A05 再確認」）。
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


def read_text(rel: str) -> str | None:
    path = root / rel
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        errors.append(f"{rel} を読み込めません: {exc}")
        return None


# --- desktop/src/main.rs: plugin 初期化はあるが check 呼び出しは無い ---------------
main_rs = read_text("desktop/src/main.rs")
if main_rs is not None:
    init = "tauri_plugin_updater::Builder::new().build()"
    if init not in main_rs:
        errors.append(f"desktop/src/main.rs に {init} がありません")

    # 実際の更新チェック呼び出しは禁止。設定フィールド名 check_updates と誤判定しない
    # よう、呼び出し形式（トレイト修飾 / メソッド呼び出し）のみを検出する。
    for pattern in (r"UpdaterExt::check", r"\.check_updates?\s*\("):
        if re.search(pattern, main_rs) is not None:
            errors.append(
                "desktop/src/main.rs に更新チェック呼び出しがあります "
                f"(pattern={pattern!r})"
            )

# --- desktop/tauri.conf.json: pubkey / endpoints が無い --------------------------
conf_text = read_text("desktop/tauri.conf.json")
if conf_text is not None:
    try:
        conf = json.loads(conf_text)
    except json.JSONDecodeError as exc:
        errors.append(f"desktop/tauri.conf.json の JSON パースに失敗しました: {exc}")
    else:
        plugins = conf.get("plugins") if isinstance(conf, dict) else None
        updater = plugins.get("updater") if isinstance(plugins, dict) else None
        if isinstance(updater, dict):
            for key in ("pubkey", "endpoints"):
                if key in updater:
                    errors.append(
                        f"desktop/tauri.conf.json に plugins.updater.{key} が存在します"
                    )
        elif updater is not None:
            errors.append("desktop/tauri.conf.json の plugins.updater が辞書型ではありません")

# --- web/src/lib.rs: 固定文言があり editable input が無い ------------------------
web_rs = read_text("web/src/lib.rs")
if web_rs is not None:
    if "更新確認: 未配線" not in web_rs:
        errors.append("web/src/lib.rs に固定文言『更新確認: 未配線』がありません")
    if 'id="settings-check-updates"' in web_rs:
        errors.append(
            'web/src/lib.rs に editable な id="settings-check-updates" が存在します'
        )

# --- desktop/src/commands/config.rs: 現在値復元が残っている ----------------------
config_rs = read_text("desktop/src/commands/config.rs")
if config_rs is not None:
    restore = "merged.general.check_updates = current.general.check_updates;"
    if restore not in config_rs:
        errors.append(
            "desktop/src/commands/config.rs に check_updates の現在値復元がありません"
        )
    # 保存時 fail-closed の回帰を守る単体 test も必須。
    test_name = "fn merge_secrets_preserves_check_updates_value()"
    if test_name not in config_rs:
        errors.append(
            f"desktop/src/commands/config.rs に {test_name} の単体 test がありません"
        )

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print("OK: updater は未配線のまま、固定表示と check_updates 現在値復元を確認")
PY
