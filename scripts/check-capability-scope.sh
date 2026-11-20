#!/usr/bin/env bash
# 設定画面の Tauri IPC を get_config / update_config_patch に限定する静的ゲート（fail-closed）。
# web 側ソースの静的解析であり、runtime の権限制限（capability / ACL）ではない。
# 独自 command は ACL 非対象という根拠は ADR-0253:73-74（ADR-0264 は IPC 経路の設計受容）。
# 出典はコードのみ（GOAL.md「R6 A05 前提 2」/ ADR-0253:73-74 / ADR-0274）。
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
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
errors: list[str] = []

# 設定画面から許可する command 名（GOAL.md の必須事項 1）。
ALLOWED_COMMANDS = {"get_config", "update_config_patch"}


def read_text(rel: str) -> str | None:
    path = root / rel
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        errors.append(f"{rel} を読み込めません: {exc}")
        return None


# --- web/src/lib.rs: 設定画面の IPC 呼び出し -----------------------------------
web_rs = read_text("web/src/lib.rs")
if web_rs is not None:
    # invoke("...") / invoke_update("...") の第 1 引数（文字列リテラル）だけを抽出する。
    # 関数定義 `async fn invoke(command: &str, ...)` は文字列が続かないため一致しない。
    # `invoke_fn.call2(...)` や `JsValue::from_str("invoke")` も `\("` に続かないため無視する。
    command_pattern = r'\binvoke(?:_update)?\s*\(\s*"([^"]*)"'
    found = set(re.findall(command_pattern, web_rs))

    if not found:
        errors.append(
            "web/src/lib.rs に設定画面用の invoke 呼び出しが見つかりません "
            f"(期待: {', '.join(sorted(ALLOWED_COMMANDS))})"
        )
    missing = ALLOWED_COMMANDS - found
    if missing:
        errors.append(
            "web/src/lib.rs に許可 command の呼び出しがありません: "
            + ", ".join(sorted(missing))
        )
    extra = found - ALLOWED_COMMANDS
    if extra:
        errors.append(
            "web/src/lib.rs に許可されていない command 呼び出しがあります: "
            + ", ".join(sorted(extra))
        )

    # Tauri plugin の invoke スキーム（plugin:fs|... 等）は設定画面から呼ばない。
    if "plugin:" in web_rs:
        errors.append("web/src/lib.rs に plugin: 参照（Tauri plugin invoke）が存在します")

    # 更新チェック API / command は web 側に置かない。
    # 設定フィールドの `check_updates`（複数形）は現状の正当なコードなので、
    # 語境界で複数形を除外し、単数形の API 呼び出しのみを検出する。
    # Tauri v2 の `updater.check().await` 等は `.check(` なので直後に `(` を要求し、
    # `.check_updates(` / `.check_update(` / `.checked()` と誤爆しないようにする。
    updater_patterns = [
        (r"UpdaterExt::check", "UpdaterExt::check"),
        (r"\bcheckUpdates\b", "checkUpdates"),
        (r"\bcheck_update\b", "check_update"),
        (r"\.check_updates?\s*\(", ".check_update(s)("),
        (r"\.check\s*\(", ".check("),
    ]
    for pattern, label in updater_patterns:
        if re.search(pattern, web_rs) is not None:
            errors.append(
                "web/src/lib.rs に更新チェック呼び出しがあります "
                f"({label}, pattern={pattern!r})"
            )

# desktop/src/main.rs: invoke_handler に設定 command が登録されているかを確認する。
# 抽出不能なら検査できないため fail-closed とする（cargo check も必須ゲートだが、
# 本ゲート単体を fail-open にしない）。
main_rs = read_text("desktop/src/main.rs")
if main_rs is not None:
    handler_match = re.search(r"generate_handler!\s*\[(.*?)\]", main_rs, re.S)
    if handler_match is None:
        errors.append("desktop/src/main.rs から generate_handler! の登録一覧を抽出できません")
    else:
        registered = set(
            re.findall(r"commands::config::([A-Za-z0-9_]+)", handler_match.group(1))
        )
        missing_in_handler = ALLOWED_COMMANDS - registered
        if missing_in_handler:
            errors.append(
                "desktop/src/main.rs の invoke_handler に設定画面用 command の登録がありません: "
                + ", ".join(sorted(missing_in_handler))
            )

# desktop/tauri.conf.json の plugins ブロックは完全削除済みで、
# check-tauri-plugin-config.sh が担当する。本ゲートは web 側の IPC 表面のみを検査する。

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print("OK: 設定画面の IPC は get_config / update_config_patch に限定されています")
PY
