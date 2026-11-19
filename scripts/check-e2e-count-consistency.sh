#!/usr/bin/env bash
# E2E 期待件数（Checker(expected=N)）と current 表記（todo CI-H01 /
# docs/documentation-operations.md）の一致を静的に検査する（fail-closed）。
# 出典はコードのみ（ADR-0090）。履歴 254 / 253 と完了済み TODO の数値は検査しない。
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
import ast
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


def bullet_block(lines: list[str], index: int) -> str:
    """指定行から次のトピック（見出し／別 bullets）までの 1 項目を返す。"""
    block = [lines[index]]
    for line in lines[index + 1:]:
        if line.startswith("- ") or line.startswith("#"):
            break
        block.append(line)
    return "\n".join(block)


# --- 出典: tools/verify-web-reader.py の Checker(expected=N) ----------------------
source = read_text("tools/verify-web-reader.py")
tree: ast.Module | None = None
expected: int | None = None
if source is not None:
    try:
        tree = ast.parse(source, filename="tools/verify-web-reader.py")
    except SyntaxError as exc:
        errors.append(f"tools/verify-web-reader.py の AST パースに失敗しました: {exc}")

if tree is not None:
    checker_calls = [
        node
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "Checker"
    ]
    if len(checker_calls) != 1:
        errors.append(
            f"Checker(...) の呼び出しは 1 件である必要があります（実際: {len(checker_calls)} 件）"
        )
    else:
        expected_kw = [kw for kw in checker_calls[0].keywords if kw.arg == "expected"]
        if len(expected_kw) != 1:
            errors.append(
                f"Checker(expected=...) は 1 件である必要があります（実際: {len(expected_kw)} 件）"
            )
        else:
            value = expected_kw[0].value
            if (
                isinstance(value, ast.Constant)
                and isinstance(value.value, int)
                and not isinstance(value.value, bool)
                and value.value > 0
            ):
                expected = value.value
            else:
                errors.append("Checker(expected=...) は正の整数リテラルである必要があります")

# --- checker.check / checker.equal の第 1 引数（文字列リテラル）を数える ------------
# equal() は内部で self.check() を呼ぶが、AST 上の直接呼び出しだけを数える。
# レシーバを checker に限定し、Checker クラス内の self.check は対象外とする。
names: list[str] = []
if tree is not None:
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        func = node.func
        if not (isinstance(func, ast.Attribute) and func.attr in ("check", "equal")):
            continue
        recv = func.value
        if not (isinstance(recv, ast.Name) and recv.id == "checker"):
            continue
        if not node.args:
            errors.append(f"checker.{func.attr} の第 1 引数がありません (line {node.lineno})")
            continue
        first = node.args[0]
        if isinstance(first, ast.Constant) and isinstance(first.value, str):
            names.append(first.value)
        else:
            errors.append(
                f"checker.{func.attr} の第 1 引数は文字列リテラルである必要があります"
                f" (line {node.lineno})"
            )

    # 同名 check の重複検知（ADR-0182）
    duplicates = sorted({name for name in names if names.count(name) > 1})
    if duplicates:
        errors.append(f"check 名が重複しています: {duplicates}")

    # 期待総数との一致（ADR-0090）
    if expected is not None and len(names) != expected:
        errors.append(
            f"checker.check + checker.equal = {len(names)} 件、"
            f"Checker(expected={expected}) と不一致です"
        )

# --- todo.md の未着手 CI-H01 行（完了済み TODO は検査しない） ----------------------
todo = read_text("todo.md")
if todo is not None:
    lines = todo.splitlines()
    start = next((i for i, line in enumerate(lines) if line.startswith("## 未着手")), None)
    if start is None:
        errors.append("todo.md に『## 未着手』見出しが見つかりません")
    else:
        end = next(
            (i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")),
            len(lines),
        )
        section = lines[start:end]
        ci_blocks = [
            bullet_block(section, i)
            for i, line in enumerate(section)
            if line.startswith("- ") and "【CI-H01】" in line
        ]
        if len(ci_blocks) != 1:
            errors.append(
                f"todo.md 未着手の CI-H01 行は 1 件である必要があります（実際: {len(ci_blocks)} 件）"
            )
        elif expected is not None:
            token = f"E2E {expected} 件"
            if token not in ci_blocks[0]:
                errors.append(f"todo.md 未着手 CI-H01 に『{token}』が含まれていません")

# --- docs/documentation-operations.md の current 値節 -----------------------------
# 履歴の 254 / 253 は別 bullets にあるため、current 値の 1 項目だけを検査する。
doc = read_text("docs/documentation-operations.md")
if doc is not None:
    lines = doc.splitlines()
    idxs = [i for i, line in enumerate(lines) if line.startswith("- current 値は")]
    if len(idxs) != 1:
        errors.append(
            f"documentation-operations.md の current 値行は 1 件である必要があります"
            f"（実際: {len(idxs)} 件）"
        )
    else:
        text = bullet_block(lines, idxs[0])
        match = re.search(r"current 値は\s*(\d+)\s*件", text)
        if match is None:
            errors.append(
                "documentation-operations.md の current 値節が『current 値は N 件』形式ではありません"
            )
        elif expected is not None and int(match.group(1)) != expected:
            errors.append(
                f"documentation-operations.md の current 値は {match.group(1)} 件で、"
                f"Checker(expected={expected}) と不一致です"
            )
        if expected is not None and f"Checker(expected={expected})" not in text:
            errors.append(
                f"documentation-operations.md の current 値節に"
                f"『Checker(expected={expected})』が含まれていません"
            )

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print(
    f"OK: E2E 期待件数 {expected} 件 / check+equal {len(names)} 件、"
    "todo CI-H01 と documentation-operations の current 表記は一致"
)
PY
