#!/usr/bin/env bash
# meta CSP（web/index.html）と config CSP（desktop/tauri.conf.json）の
# 静的一貫性を検査する（ADR-0130 の未決定を解消 / ADR-0268）。
#
# - script-src / style-src は meta と config で完全一致（空白・token 順は無視）。
# - config には default-src / img-src / font-src / connect-src が必須。
# - meta に http:// / https:// オリジンは禁止。
# - config の外部オリジンは許可リストのみ（未知オリジンは fail-closed）。
#
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

META_REL = "web/index.html"
CONF_REL = "desktop/tauri.conf.json"

# --- web/index.html から meta CSP を正規表現で抽出 -------------------------------
# 属性値内の '>' をタグ終端と誤判定しないよう、引用符を考慮して <meta> を拾う。
TAG_RE = re.compile(r"<meta\b((?:\"[^\"]*\"|'[^']*'|[^>\"'])*)>", re.IGNORECASE)
ATTR_RE = re.compile(
    r"""([A-Za-z_:][\w:.-]*)\s*=\s*("([^"]*)"|'([^']*)'|([^\s"'`=<>]+))"""
)


def attributes(tag_attrs: str) -> dict[str, str]:
    result: dict[str, str] = {}
    for match in ATTR_RE.finditer(tag_attrs):
        key = match.group(1).lower()
        dq, sq, bare = match.group(3), match.group(4), match.group(5)
        value = dq if dq is not None else (sq if sq is not None else bare)
        if value is None:
            errors.append(f"{META_REL} の meta タグ属性 {key} の値を取得できません")
            continue
        if key in result:
            errors.append(f"{META_REL} の meta タグに属性 {key} が重複しています")
        result[key] = value
    return result


def read_text(rel: str) -> str | None:
    path = root / rel
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        errors.append(f"{rel} を読み込めません: {exc}")
        return None


meta_policy: str | None = None
html = read_text(META_REL)
if html is not None:
    csp_tags: list[str] = []
    for tag in TAG_RE.finditer(html):
        attrs = attributes(tag.group(1))
        if attrs.get("http-equiv", "").lower() == "content-security-policy":
            content = attrs.get("content")
            if content is None:
                errors.append(f"{META_REL} の CSP meta タグに content 属性がありません")
            else:
                csp_tags.append(content)
    # 見つからない／複数は fail-closed。
    if len(csp_tags) != 1:
        errors.append(
            f"{META_REL} の CSP meta タグは 1 件である必要があります（実際: {len(csp_tags)} 件）"
        )
    elif not csp_tags[0].strip():
        errors.append(f"{META_REL} の CSP content が空です")
    else:
        meta_policy = csp_tags[0]

# --- desktop/tauri.conf.json から app.security.csp を取得 ------------------------
config_policy: str | None = None
conf: dict | None = None
try:
    with (root / CONF_REL).open(encoding="utf-8") as f:
        conf = json.load(f)
except (OSError, UnicodeDecodeError, ValueError) as exc:
    errors.append(f"{CONF_REL} の JSON 読み込み/パースに失敗しました: {exc}")

if conf is not None:
    if not isinstance(conf, dict):
        errors.append(f"{CONF_REL} のトップレベルは JSON オブジェクトである必要があります")
    else:
        try:
            csp = conf["app"]["security"]["csp"]
        except (KeyError, TypeError) as exc:
            errors.append(f"{CONF_REL} から app.security.csp を取得できません: {exc}")
        else:
            if not isinstance(csp, str) or not csp.strip():
                errors.append(f"{CONF_REL} の app.security.csp が空/文字列ではありません")
            else:
                config_policy = csp

# --- directive 名と token 集合を正規化（空白・順不同） ---------------------------
# 既知集合外の directive は fail-closed で拒否する。特に script-src-elem /
# style-src-elem は base の script-src / style-src より優先適用され、
# 「script-src / style-src が一致すれば実効も一致する」という前提を破るため。
KNOWN_DIRECTIVES = {
    "default-src",
    "script-src",
    "style-src",
    "img-src",
    "font-src",
    "connect-src",
}


def parse_policy(label: str, policy: str) -> dict[str, set[str]]:
    directives: dict[str, set[str]] = {}
    for raw in policy.split(";"):
        part = raw.strip()
        if not part:
            continue
        tokens = part.split()
        name = tokens[0].lower()
        if name not in KNOWN_DIRECTIVES:
            errors.append(f"{label} に既知集合外の directive があります: {name}")
            continue
        if name in directives:
            errors.append(f"{label} に directive {name} が重複しています")
        directives[name] = set(tokens[1:])
    return directives


meta_dirs = parse_policy(META_REL, meta_policy) if meta_policy else {}
config_dirs = parse_policy(CONF_REL, config_policy) if config_policy else {}

# --- script-src / style-src は meta と config で完全一致 -------------------------
for directive in ("script-src", "style-src"):
    if directive not in meta_dirs:
        errors.append(f"{META_REL} に {directive} がありません")
    if directive not in config_dirs:
        errors.append(f"{CONF_REL} に {directive} がありません")
    if directive in meta_dirs and directive in config_dirs:
        if meta_dirs[directive] != config_dirs[directive]:
            errors.append(
                f"{directive} が一致しません: "
                f"meta={sorted(meta_dirs[directive])} config={sorted(config_dirs[directive])}"
            )

# --- config に必須 directive が存在すること -------------------------------------
for directive in ("default-src", "img-src", "font-src", "connect-src"):
    if directive not in config_dirs:
        errors.append(f"{CONF_REL} に必須 directive {directive} がありません")

# --- 'unsafe-eval' は ADR-0130 で棄却済み（両 CSP で禁止） -----------------------
# meta と config を同時に緩和すると一致検査だけでは検出できないため明示的に拒否する。
for label, dirs in ((META_REL, meta_dirs), (CONF_REL, config_dirs)):
    script_tokens = dirs.get("script-src", set())
    if any(token.lower() == "'unsafe-eval'" for token in script_tokens):
        errors.append(f"{label} の script-src に 'unsafe-eval' が含まれています（ADR-0130 違反）")

# --- meta に http:// / https:// が無いこと --------------------------------------
if meta_policy and re.search(r"https?://", meta_policy, re.IGNORECASE):
    errors.append(f"{META_REL} の CSP に http:// / https:// オリジンが含まれています")

# --- config の外部オリジンは許可リストのみ（未知オリジンは fail-closed） ----------
ALLOWED_ORIGINS = {
    "ipc:",
    "http://ipc.localhost",
    "https://api.openai.com",
    "wss://api.openai.com",
    "http://localhost:11434",
}
ALLOWED_SCHEMES = {"data:", "blob:"}


def is_keyword(token: str) -> bool:
    # 'self' / 'unsafe-inline' / 'wasm-unsafe-eval' / 'none' / nonce-/hash- 等。
    return len(token) >= 2 and token.startswith("'") and token.endswith("'")


unknown: set[str] = set()
for tokens in config_dirs.values():
    for token in tokens:
        lowered = token.lower()
        if is_keyword(token) or lowered in ALLOWED_SCHEMES or lowered in ALLOWED_ORIGINS:
            continue
        unknown.add(token)
if unknown:
    errors.append(
        f"{CONF_REL} の CSP に許可リスト外のオリジン/トークンがあります: {sorted(unknown)}"
    )

if errors:
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    sys.exit(1)

print(
    "OK: meta/config の script-src / style-src は一致、"
    "config の必須 directive とオリジン許可リストも適合"
)
PY
