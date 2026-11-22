#!/usr/bin/env bash
# Config DTO の desktop / web 契約を静的に検証する（fail-closed）。
#
# desktop/src/commands/config.rs と web/src/lib.rs から struct のフィールド名集合
# （pub / private の別を問わず）を抽出し、PEER 対応する struct 同士で完全一致を
# 検査する。未知フィールド・欠落フィールド・struct 名の消失・重複定義は失敗。
#
# CAS 補助フィールドのみ片側例外とする（明示的な例外は下記 PEERS の one_sided）:
#   - revision : web 側 SettingsView の CAS 補助。desktop ConfigView にも現れるが
#                ConfigPatch 側には無いため、ConfigPatch ペアでは web 側のみ許可。
#   - expected : desktop 側 ConfigPatch の CAS 入力。web 側には現れない。
#
# desktop には ReaderConfigView が無いため、reader の PEER は core::ReaderConfig
# （ConfigView.reader の実型）と web::SettingsReaderView の組を使う。
#
# 第 1 引数で repo root を差し替えられる（負向 test 用）。
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

DESKTOP = "desktop/src/commands/config.rs"
WEB = "web/src/lib.rs"
CORE = "core/src/config.rs"

STRUCT_RE = re.compile(
    r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:unsafe\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)
FIELD_RE = re.compile(
    r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*.+[,;]\s*$"
)


def read(rel: str) -> str | None:
    try:
        return (root / rel).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        errors.append(f"{rel} を読み込めません: {exc}")
        return None


def parse_structs(text: str, rel: str, expected: set[str]) -> dict[str, set[str]]:
    """期待名の struct だけを解析し、名前 -> フィールド名集合を返す。"""
    lines = text.splitlines()
    found: dict[str, set[str]] = {}
    i = 0
    while i < len(lines):
        matched = STRUCT_RE.match(lines[i])
        if matched is None or matched.group(1) not in expected:
            i += 1
            continue
        name = matched.group(1)
        depth = 0
        opened_line: int | None = None
        j = i
        while j < len(lines):
            # 行コメント（doc comment 含む）は波括弧の数えから除外する。
            code = lines[j].split("//", 1)[0]
            if "{" in code and opened_line is None:
                opened_line = j
            depth += code.count("{") - code.count("}")
            if opened_line is not None and depth <= 0:
                break
            j += 1
        if opened_line is None or j >= len(lines):
            errors.append(f"{rel}: struct {name} の本体を検出できません（tuple struct 等）")
            i += 1
            continue
        fields: set[str] = set()
        in_attribute = False
        for raw in lines[opened_line + 1 : j]:
            stripped = raw.strip()
            if in_attribute:
                if "]" in stripped:
                    in_attribute = False
                continue
            if not stripped or stripped.startswith("//"):
                continue
            if stripped.startswith("#"):
                if "[" in stripped and "]" not in stripped:
                    in_attribute = True
                continue
            field = FIELD_RE.match(raw.split("//", 1)[0])
            if field is None:
                errors.append(f"{rel}: struct {name} の本体を解釈できません: {stripped}")
                continue
            fields.add(field.group(1))
        if name in found:
            errors.append(f"{rel}: struct {name} が重複定義されています")
        else:
            found[name] = fields
        i = j + 1
    return found


# (desktop struct, web struct, desktop 側の出典, 片側例外)
# 例外の rule: allowed = 許可する側, required = 必ず存在する側。
# 許可されない側に現れた case も未知フィールドとして失敗させる。
WEB_RULE = {"allowed": {"web"}, "required": {"web"}}
DESKTOP_RULE = {"allowed": {"desktop"}, "required": {"desktop"}}
BOTH_RULE = {"allowed": {"desktop", "web"}, "required": {"web"}}

PEERS = [
    ("ConfigView", "SettingsView", DESKTOP, {"revision": BOTH_RULE}),
    (
        "ConfigPatch",
        "SettingsView",
        DESKTOP,
        {"revision": WEB_RULE, "expected": DESKTOP_RULE},
    ),
    ("GeneralConfigView", "SettingsGeneralView", DESKTOP, {}),
    ("AiConfigView", "SettingsAiView", DESKTOP, {}),
    ("OpenAiConfigView", "SettingsOpenAiView", DESKTOP, {}),
    ("OllamaConfigView", "SettingsOllamaView", DESKTOP, {}),
    ("LintConfigView", "SettingsLintView", DESKTOP, {}),
    ("StorageConfigView", "StorageConfigView", DESKTOP, {}),
    ("EditorConfigView", "SettingsEditorView", DESKTOP, {}),
    # desktop には ReaderConfigView が無いため core::ReaderConfig を PEER とする。
    ("ReaderConfig", "SettingsReaderView", CORE, {}),
    ("LocalRootView", "LocalRootView", DESKTOP, {}),
    ("GoogleDriveConfigView", "GoogleDriveConfigView", DESKTOP, {}),
]

expected_by_rel: dict[str, set[str]] = {}
for desktop_name, web_name, desktop_rel, _ in PEERS:
    expected_by_rel.setdefault(WEB, set()).add(web_name)
    expected_by_rel.setdefault(desktop_rel, set()).add(desktop_name)

sources: dict[str, dict[str, set[str]]] = {}
for rel, expected in expected_by_rel.items():
    text = read(rel)
    sources[rel] = {} if text is None else parse_structs(text, rel, expected)

for desktop_name, web_name, desktop_rel, one_sided in PEERS:
    desktop_fields = sources[desktop_rel].get(desktop_name)
    web_fields = sources[WEB].get(web_name)
    if desktop_fields is None:
        errors.append(f"{desktop_rel}: struct {desktop_name} が見つかりません（改名／削除）")
    if web_fields is None:
        errors.append(f"{WEB}: struct {web_name} が見つかりません（改名／削除）")
    if desktop_fields is None or web_fields is None:
        continue

    only_desktop = set(desktop_fields)
    only_web = set(web_fields)
    for field, rule in one_sided.items():
        for side, present in (
            ("desktop", field in only_desktop),
            ("web", field in only_web),
        ):
            if present and side not in rule["allowed"]:
                errors.append(
                    f"{desktop_rel}:{desktop_name} ↔ {WEB}:{web_name}: "
                    f"{field} は {side} 側に許されません（片側例外の範囲外）"
                )
            if not present and side in rule["required"]:
                errors.append(
                    f"{desktop_rel}:{desktop_name} ↔ {WEB}:{web_name}: "
                    f"{field} が {side} 側にありません（CAS 契約上必須）"
                )
        only_desktop.discard(field)
        only_web.discard(field)

    extra_desktop = sorted(only_desktop - only_web)
    extra_web = sorted(only_web - only_desktop)
    if extra_desktop or extra_web:
        errors.append(
            f"{desktop_rel}:{desktop_name} ↔ {WEB}:{web_name} のフィールド集合が不一致です"
            f"（desktop のみ: {extra_desktop or 'なし'} / web のみ: {extra_web or 'なし'}）"
        )

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print(
    f"OK: Config DTO 契約 {len(PEERS)} ペアのフィールド集合が完全一致"
    "（片側例外: revision / expected のみ明示的に許可）"
)
PY
