#!/usr/bin/env python3
"""Web 版 EPUB リーダーのエンドツーエンド検証（Chrome DevTools Protocol）。

trunk のビルド成果物を静的配信し、ヘッドレス Chrome で
ファイル選択 → 章表示 → 相対リンク移動 → サニタイズを確認する。

使い方:
    cd web && trunk build --dist dist
    cd .. && python3 tools/verify-web-reader.py --dist web/dist

必要なもの: Python 3.10 以降（websockets）と Chrome または Chromium。
サンプル EPUB は一時ディレクトリに生成する。
"""

from __future__ import annotations

import argparse
import asyncio
import base64
import json
import os
import queue
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request
import zipfile
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

DEFAULT_DIST = Path(__file__).resolve().parent.parent / "web" / "dist"
DIST = DEFAULT_DIST
SAMPLE = Path(tempfile.gettempdir()) / "openrustbooks-e2e" / "sample.epub"
PORT = 8099
DEBUG_PORT = 9222

CH1 = """<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Chapter One</title></head>
<body>
<h1 id="top">Chapter One</h1>
<p>Hello from chapter one.</p>
<p><a id="to-ch2" href="ch2.xhtml">Go to chapter two</a></p>
<p><a id="to-ch2-part" href="ch2.xhtml#part">Go to chapter two part</a></p>
<p><a id="external-link" href="https://example.com">external link</a></p>
<p><a id="mailto-link" href="mailto:test@example.com">mail link</a></p>
<p><a id="bad-js" href="javascript:alert(1)">bad link</a></p>
<p><a id="missing" href="missing.xhtml">missing link</a></p>
<p><a id="ext-link" href="https://example.invalid/x">external</a></p>
<img id="pic" src="images/pic.png" alt="pic"/>
<img id="ext-img" src="https://example.invalid/x.png" alt="ext"/>
<script>window.__pwned = true;</script>
<p onclick="window.__pwned2 = true">click handler text</p>
</body>
</html>
"""

FILLER = "\n".join(f"<p>filler paragraph {i}</p>" for i in range(40))

EVIL_SVG = """<?xml version="1.0" encoding="UTF-8"?>
<?xml-stylesheet href="https://attacker.example/style.css" type="text/css"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
     width="20" height="20" onload="window.__pwned = 1">
<script>window.__pwned = 1;</script>
<rect width="20" height="20" fill="red"/>
<foreignObject><div onclick="window.__pwned = 1">x</div></foreignObject>
<image xlink:href="java&#10;script:window.__pwned = 1" width="1" height="1"/>
<set attributeName="onload" to="window.__pwned = 1" begin="0s"/>
<animate attributeName="href" values="javascript:window.__pwned = 1" begin="0s"/>
<use xlink:href="https://attacker.example/track.svg#x" width="1" height="1"/>
<image href="data:image/png;base64,AAAA" width="1" height="1"/>
<image href="images/pic.png" width="1" height="1"/>
</svg>
"""

CH2 = f"""<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Chapter Two</title></head>
<body>
<h1 id="start">Chapter Two</h1>
<p>Second chapter text.</p>
{FILLER}
<img id="tall" src="images/tall.png" alt="tall"/>
<img id="evil-svg" src="images/evil.svg" alt="evil svg"/>
<h2 id="part">Part Two</h2>
<p>Part two text.</p>
<p><a id="to-start" href="#start">Back to top of chapter two</a></p>
<p><a id="back" href="ch1.xhtml">Back to chapter one</a></p>
</body>
</html>
"""

NAV = """<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Contents</title></head>
<body>
<nav epub:type="toc">
<ol>
<li><a href="ch1.xhtml">Chapter One</a></li>
<li><a href="ch2.xhtml#part">Chapter Two</a></li>
</ol>
</nav>
<nav epub:type="landmarks">
<ol>
<li><a epub:type="bodymatter" href="ch2.xhtml">Start Reading</a></li>
</ol>
</nav>
<nav epub:type="page-list">
<ol>
<li><a href="ch1.xhtml#top">1</a></li>
<li><a href="ch2.xhtml#part">2</a></li>
</ol>
</nav>
</body>
</html>
"""

OPF = """<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Sample Book</dc:title>
    <dc:language>ja</dc:language>
    <dc:identifier id="pub-id">urn:uuid:00000000-0000-0000-0000-000000000001</dc:identifier>
    <meta property="dcterms:modified">2026-01-01T00:00:00Z</meta>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="pic" href="images/pic.png" media-type="image/png"/>
    <item id="tall" href="images/tall.png" media-type="image/png"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"""

CONTAINER = """<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"""

PNG = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
)


def tall_png(width: int = 3000, height: int = 4000) -> bytes:
    """指定サイズのノイズ PNG を組み立てる（レイアウトずれ検証用）。

    単色だと展開・デコードが速すぎて画像読み込み待ちの効果を判別できないため、
    圧縮の効きにくいノイズを使う。
    """
    import random
    import struct
    import zlib

    rng = random.Random(20261115)
    row = bytearray()
    for _ in range(width):
        row += bytes((rng.randrange(256), rng.randrange(256), rng.randrange(256)))
    raw = b"".join(b"\x00" + bytes(row) for _ in range(height))

    def chunk(kind: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw, 6))
        + chunk(b"IEND", b"")
    )


def build_sample() -> None:
    SAMPLE.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(SAMPLE, "w") as z:
        z.writestr(
            zipfile.ZipInfo("mimetype"),
            "application/epub+zip",
            compress_type=zipfile.ZIP_STORED,
        )
        z.writestr("META-INF/container.xml", CONTAINER, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/content.opf", OPF, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/nav.xhtml", NAV, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch1.xhtml", CH1, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch2.xhtml", CH2, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/pic.png", PNG, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr(
            "OEBPS/images/tall.png", tall_png(), compress_type=zipfile.ZIP_DEFLATED
        )
        z.writestr("OEBPS/images/evil.svg", EVIL_SVG, compress_type=zipfile.ZIP_DEFLATED)


class SpaHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(DIST), **kwargs)

    def do_GET(self):
        path = self.translate_path(self.path)
        if not os.path.exists(path) and "." not in os.path.basename(self.path):
            self.path = "/index.html"
        return super().do_GET()

    def log_message(self, *args):
        pass


def wait_for_debug_port(timeout: float = 20.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{DEBUG_PORT}/json/version", timeout=1):
                return
        except OSError:
            time.sleep(0.2)
    raise RuntimeError("Chrome のデバッグポートが開きませんでした")


# --- 起動した Chrome のターゲットを決定的に特定するための支援関数 --------------------
# 方針（優先順）:
#   1. 起動 Chrome 自身の stderr に出る `DevTools listening on ws://.../devtools/browser/<id>`
#      を捕捉し、その browser-level WS に接続する。port が既存 Chrome に占有されていても
#      Chrome は [::1] にフォールバックするため、この行は起動プロセス固有の WS を指す。
#   2. その browser WS で Target.getTargets を呼び、属する page target だけを候補にする。
#   3. 起動前に /json で採取した target id は「ヒント」（候補の優先順付け）としてのみ使う。
# 候補が 0 件・複数件・deadline 超過のときは fail-closed で RuntimeError にする。
BROWSER_WS_RE = re.compile(r"DevTools listening on (ws://[^\s]+/devtools/browser/[0-9a-fA-F-]+)")
TARGET_SELECT_TIMEOUT = 8.0


def fetch_json_targets() -> list[dict] | None:
    """/json を 1 回だけ読み、失敗したら None を返す（起動前スナップショット用）。"""
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{DEBUG_PORT}/json", timeout=1) as response:
            targets = json.load(response)
        return targets if isinstance(targets, list) else None
    except (OSError, ValueError, TimeoutError):
        return None


def snapshot_target_ids() -> set[str]:
    """差分ヒント用に、Chrome 起動前に見えていた target id を記録する。"""
    targets = fetch_json_targets()
    if targets is None:
        return set()
    return {t["id"] for t in targets if isinstance(t, dict) and isinstance(t.get("id"), str)}


def drain_chrome_stderr(stderr, browser_ws_queue: queue.Queue[str]) -> None:
    """Chrome の stderr を最後まで読み、browser WS URL をキューへ渡す。

    stderr を PIPE にした以上、読み続けないとバッファが詰まるため専用スレッドで吸い出す。
    """
    try:
        for raw in iter(stderr.readline, b""):
            match = BROWSER_WS_RE.search(raw.decode("utf-8", errors="replace"))
            if match:
                browser_ws_queue.put(match.group(1))
    except (OSError, ValueError):
        pass
    finally:
        stderr.close()


def wait_for_browser_ws(browser_ws_queue: queue.Queue[str], timeout: float = 10.0) -> str:
    """起動 Chrome 自身が stderr に出した browser WS URL を待つ。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        remaining = deadline - time.time()
        if remaining <= 0:
            break
        try:
            return browser_ws_queue.get(timeout=min(0.2, remaining))
        except queue.Empty:
            continue
    raise RuntimeError(
        f"起動した Chrome の DevTools listening 行を stderr から取得できませんでした"
        f"（タイムアウト {timeout}s。デバッグポート {DEBUG_PORT} をバインドできなかった可能性があります）"
    )


async def select_page_target(browser_ws_url: str, known_ids: set[str]) -> dict:
    """起動 Chrome の browser WS から about:blank の page target を一意に選ぶ（fail-closed）。"""
    import websockets

    # Target.getTargets の targetInfos には webSocketDebuggerUrl が無いため、
    # 自身の browser WS と同じオリジンから page WS を組み立てる。
    ws_base = browser_ws_url.rsplit("/devtools/browser/", 1)[0] + "/devtools/page/"
    deadline = time.time() + TARGET_SELECT_TIMEOUT
    async with websockets.connect(browser_ws_url, max_size=None) as ws:
        cdp = Cdp(ws)
        while time.time() < deadline:
            # recv の既定 10s が 8s の deadline を飲み込まないよう、残時間以内に抑える。
            remaining = deadline - time.time()
            if remaining <= 0:
                break
            response = await cdp.send("Target.getTargets", timeout=min(2.0, remaining))
            targets = response.get("result", {}).get("targetInfos", [])
            pages = [t for t in targets if t.get("type") == "page"]
            # 差分はヒント: 起動前に存在しなかった id を優先する。
            fresh = [t for t in pages if t.get("targetId") not in known_ids]
            candidates = fresh or pages
            blanks = [t for t in candidates if t.get("url") == "about:blank"]
            if blanks:
                candidates = blanks
            if len(candidates) > 1:
                raise RuntimeError(
                    "起動 Chrome の page target が複数見つかり、接続先を一意に決められませんでした"
                    f"（browser={browser_ws_url}, candidates="
                    f"{[(t.get('targetId'), t.get('url')) for t in candidates]!r}）"
                )
            if len(candidates) == 1:
                target = candidates[0]
                return {
                    "id": target["targetId"],
                    "url": target.get("url"),
                    "webSocketDebuggerUrl": ws_base + target["targetId"],
                }
            await asyncio.sleep(0.2)
    raise RuntimeError(
        f"起動 Chrome に about:blank の page target が見つかりませんでした"
        f"（タイムアウト {TARGET_SELECT_TIMEOUT}s, browser={browser_ws_url}）"
    )


# 起動フレークの診断に使う CDP イベントだけを覚える。
# Runtime.enable / Log.enable / Network.enable は大量のイベントを流すため、
# send() の応答待ちで捨てていたものをここに退避する。
DIAGNOSTIC_EVENTS = {
    "Runtime.consoleAPICalled",
    "Runtime.exceptionThrown",
    "Log.entryAdded",
    "Network.loadingFailed",
    "Page.domContentEventFired",
    "Page.loadEventFired",
}
MAX_DIAGNOSTIC_EVENTS = 500
MAX_REQUEST_URLS = 500

# CDP の 1 メッセージ受信あたりのタイムアウト（秒）。wait_until の既定 20s より短い値に
# して、CDP が無応答でもデッドラインが効くようにする（ADR-0142 / 0143 は未決定の暫定値）。
CDP_RECV_TIMEOUT = 10.0


class Cdp:
    def __init__(self, ws):
        self.ws = ws
        self.counter = 0
        self.diagnostic_events: list[dict] = []
        self.request_urls: dict[str, str] = {}
        self.dropped_diagnostic_events = 0

    def _record_event(self, message: dict) -> None:
        """応答待ちに混ざるイベントを捨てずに診断用へ退避する。"""
        method = message.get("method", "")
        params = message.get("params", {})
        if method == "Network.requestWillBeSent":
            request_id = params.get("requestId")
            url = params.get("request", {}).get("url", "")
            if request_id:
                if len(self.request_urls) >= MAX_REQUEST_URLS:
                    self.request_urls.pop(next(iter(self.request_urls)))
                self.request_urls[request_id] = url
        if method not in DIAGNOSTIC_EVENTS:
            return
        if len(self.diagnostic_events) < MAX_DIAGNOSTIC_EVENTS:
            self.diagnostic_events.append(message)
        else:
            self.dropped_diagnostic_events += 1

    def take_diagnostics(self):
        """退避したイベントを引き出してクリアする。"""
        events = self.diagnostic_events
        urls = dict(self.request_urls)
        dropped = self.dropped_diagnostic_events
        self.diagnostic_events = []
        self.request_urls = {}
        self.dropped_diagnostic_events = 0
        return events, urls, dropped

    async def send(
        self, method: str, params: dict | None = None, timeout: float = CDP_RECV_TIMEOUT
    ) -> dict:
        self.counter += 1
        request_id = self.counter
        await self.ws.send(json.dumps({"id": request_id, "method": method, "params": params or {}}))
        while True:
            try:
                raw = await asyncio.wait_for(self.ws.recv(), timeout=timeout)
            except asyncio.TimeoutError as exc:
                # CDP が沈黙したら待ちを打ち切る（fail-closed）。診断用にメソッド名、
                # request_id、退避済み／破棄済みのイベント数と内容の抜粋を添える。
                # 抜粋は non-destructive（take_diagnostics はここでは呼ばない）。
                preview = [
                    f"{event.get('method', '')}: {str(event.get('params', {}))[:160]}"
                    for event in self.diagnostic_events[:3]
                ]
                raise RuntimeError(
                    f"CDP 応答タイムアウト: method={method}, request_id={request_id}, "
                    f"timeout={timeout}s, "
                    f"buffered_diagnostic_events={len(self.diagnostic_events)}, "
                    f"dropped_diagnostic_events={self.dropped_diagnostic_events}, "
                    f"diagnostic_event_preview={preview!r}"
                ) from exc
            message = json.loads(raw)
            if message.get("id") == request_id:
                return message
            if "method" in message:
                self._record_event(message)

    async def evaluate(self, expression: str):
        result = await self.send(
            "Runtime.evaluate",
            {"expression": expression, "returnByValue": True, "awaitPromise": True},
        )
        return result.get("result", {}).get("result", {}).get("value")


class Checker:
    def __init__(self, expected: int):
        self.expected = expected
        self.failures: list[str] = []
        self.passed = 0

    def check(self, name: str, condition: bool, detail: str = "") -> None:
        if condition:
            self.passed += 1
            print(f"ok   {name}")
        else:
            self.failures.append(name)
            print(f"FAIL {name} {detail}")

    def equal(self, name: str, actual, expected) -> None:
        self.check(name, actual == expected, f"(actual={actual!r}, expected={expected!r})")


async def wait_until(cdp: Cdp, expression: str, timeout: float = 20.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            if await cdp.evaluate(expression):
                return True
        except RuntimeError:
            # CDP 無応答は検証基盤の失敗。リトライで隠さず fail-closed で伝播させる。
            raise
        except Exception:
            pass
        await asyncio.sleep(0.25)
    return False


async def open_sample(cdp: Cdp) -> bool:
    """サンプル EPUB をファイル入力にセットして change を発火する。"""
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send("DOM.querySelector", {"nodeId": root_id, "selector": "#epub-file"})
    node_id = node["result"]["nodeId"]
    if node_id == 0:
        return False
    await cdp.send("DOM.setFileInputFiles", {"nodeId": node_id, "files": [str(SAMPLE)]})
    await cdp.evaluate(
        "document.querySelector('#epub-file').dispatchEvent(new Event('change', {bubbles: true}))"
    )
    return True


async def reload_reader(cdp: Cdp) -> bool:
    """ページを再読み込みし、Reader のマウントを待つ。

    再読み込みの完了前に古い文書を操作しないよう、文書ごとの印を置いて
    それが消えるまで待つ。
    """
    await cdp.evaluate("window.__orb_reload_marker = true")
    await cdp.send("Page.reload", {"ignoreCache": True})
    await wait_until(cdp, "window.__orb_reload_marker === undefined", timeout=15)
    await wait_until(cdp, "document.readyState === 'complete'", timeout=15)
    return await wait_until(cdp, "document.querySelector('.reader-container') !== null")


async def saved_position(cdp: Cdp):
    """localStorage に保存された読書位置を返す。"""
    return await cdp.evaluate(
        "(() => {"
        " const key = Object.keys(localStorage).find((k) => k.startsWith('openrustbooks:position:'));"
        " if (!key) return null;"
        " return JSON.parse(localStorage.getItem(key));"
        " })()"
    )


async def set_sample_position(cdp: Cdp, payload: str) -> None:
    """サンプル本の読書位置を直接書き換える。

    保存はスクロールのデバウンスでも走るため、保留中の書き込みが済むのを
    待ってから書き、書き込めたことを確認する。
    """
    await asyncio.sleep(0.4)
    expression = (
        "(() => {"
        " const keys = Object.keys(localStorage);"
        " const key = keys.find((k) => k.startsWith('openrustbooks:position:sample.epub'))"
        "   ?? keys.find((k) => k.startsWith('openrustbooks:position:'));"
        " if (!key) return false;"
        f" const value = JSON.stringify({payload});"
        " localStorage.setItem(key, value);"
        " return localStorage.getItem(key) === value;"
        " })()"
    )
    for _ in range(5):
        if await cdp.evaluate(expression):
            return
        await asyncio.sleep(0.2)


async def select_chapter(cdp: Cdp, index: int) -> None:
    """章の選択を切り替えて保存を発生させる。"""
    await cdp.evaluate(
        "(() => {"
        " const select = document.querySelector('#reader-chapter-select');"
        f" select.value = '{index}';"
        " select.dispatchEvent(new Event('change', {bubbles: true}));"
        " })()"
    )


async def wait_saved(cdp: Cdp, predicate, timeout: float = 5.0):
    """保存された読書位置が条件を満たすまで待つ。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        saved = await saved_position(cdp)
        if saved is not None and predicate(saved):
            return saved
        await asyncio.sleep(0.1)
    return None


# 新文書の生成前に注入し、Wasm 起動の節目とページ側の例外を残す。
# Trunk のローダは init() 完了時に TrunkApplicationStarted を dispatch する。
BOOT_PROBE = r"""
(() => {
  window.__orbBoot = { errors: [], events: [] };
  const note = (kind, text) => {
    const boot = window.__orbBoot;
    if (!boot || !Array.isArray(boot.events) || boot.events.length >= 50) return;
    boot.events.push(kind + ': ' + String(text));
  };
  window.addEventListener('error', (e) => {
    if (window.__orbBoot && window.__orbBoot.errors.length < 50) {
      window.__orbBoot.errors.push(String(e.message));
    }
    note('pageerror', e.message);
  });
  window.addEventListener('unhandledrejection', (e) => {
    if (window.__orbBoot && window.__orbBoot.errors.length < 50) {
      window.__orbBoot.errors.push(String(e.reason));
    }
    note('unhandledrejection', e.reason);
  });
  window.addEventListener('TrunkApplicationStarted', () => note('trunk', 'started'));
  document.addEventListener('DOMContentLoaded', () => note('dom', 'DOMContentLoaded'));
})()
"""

BOOT_STATE = """(() => {
  const boot = window.__orbBoot || {};
  return {
    url: location.href,
    readyState: document.readyState,
    bodyChildren: document.body ? document.body.children.length : -1,
    hasReader: document.querySelector('.reader-container') !== null,
    rustStartedAt: window.__orbRustStartedAt ?? null,
    rustMountedAt: window.__orbRustMountedAt ?? null,
    errors: boot.errors || [],
    events: boot.events || [],
    bodySnippet: document.body ? document.body.innerHTML.slice(0, 300) : ''
  };
})()"""


def format_cdp_diagnostics(cdp: Cdp) -> list[str]:
    """Cdp に退避したイベントを可読な行へ変換する。"""
    events, urls, dropped = cdp.take_diagnostics()
    lines: list[str] = []
    for event in events:
        method = event.get("method", "")
        params = event.get("params", {})
        if method == "Runtime.consoleAPICalled":
            args = ", ".join(
                str(arg.get("value", arg.get("description", "")))
                for arg in params.get("args", [])
            )
            lines.append(f"console.{params.get('type', 'log')}: {args}")
        elif method == "Runtime.exceptionThrown":
            detail = params.get("exceptionDetails", {})
            description = detail.get("exception", {}).get("description") or detail.get("text", "")
            lines.append(f"pageerror: {description}")
        elif method == "Log.entryAdded":
            entry = params.get("entry", {})
            lines.append(
                f"log.{entry.get('level', '')}: {entry.get('text', '')} ({entry.get('url', '')})"
            )
        elif method == "Network.loadingFailed":
            request_id = params.get("requestId")
            lines.append(
                f"requestfailed: {urls.get(request_id, '?')} "
                f"{params.get('errorText', '')} type={params.get('type', '')}"
            )
        elif method in ("Page.domContentEventFired", "Page.loadEventFired"):
            lines.append(method)
    if dropped:
        lines.append(f"... {dropped} CDP diagnostic events dropped")
    return lines


async def dump_startup_diagnostics(cdp: Cdp, label: str) -> None:
    """マウント待ち失敗時にだけ原因の手掛かりを出力する（check 数は増やさない）。"""
    print(f"--- startup diagnostics [{label}] ---", flush=True)
    for line in format_cdp_diagnostics(cdp):
        print(f"  {line}", flush=True)
    try:
        state = await cdp.evaluate(BOOT_STATE)
    except Exception as exc:  # 診断取得自体が失敗しても検証は続行する
        state = None
        print(f"  page state error: {exc}", flush=True)
    if isinstance(state, dict):
        for key in (
            "url",
            "readyState",
            "bodyChildren",
            "hasReader",
            "rustStartedAt",
            "rustMountedAt",
            "errors",
            "events",
            "bodySnippet",
        ):
            print(f"  page.{key}: {state.get(key)!r}", flush=True)
    else:
        print(f"  page state unavailable: {state!r}", flush=True)
    print("--- end startup diagnostics ---", flush=True)


async def run_checks(cdp: Cdp) -> int:
    """checker を生成し、CDP 失敗時も診断とサマリを出して fail-closed で伝播させる。"""
    # 期待 check 総数: checker.check 212 + checker.equal 17 = 229。
    checker = Checker(expected=229)
    try:
        return await _run_checks(cdp, checker)
    except RuntimeError as exc:
        print(f"--- CDP エラー: {exc} ---", flush=True)
        print("--- startup diagnostics [CDP タイムアウト] ---", flush=True)
        for line in format_cdp_diagnostics(cdp):
            print(f"  {line}", flush=True)
        print("--- end startup diagnostics ---", flush=True)
        finish(checker)
        raise


async def _run_checks(cdp: Cdp, checker: Checker) -> int:
    await cdp.send("Page.enable")
    await cdp.send("Runtime.enable")
    await cdp.send("Log.enable")
    await cdp.send("Network.enable")
    await cdp.send("DOM.enable")
    # イベントは send() が捨てずに退避するため、有効化は navigate より前に行う。
    await cdp.send("Page.addScriptToEvaluateOnNewDocument", {"source": BOOT_PROBE})
    await cdp.send("Page.navigate", {"url": f"http://127.0.0.1:{PORT}/reader/e2e"})

    reader_selector = "document.querySelector('.reader-container') !== null"
    mounted = await wait_until(cdp, reader_selector)
    if not mounted:
        # 20 秒でマウントしない場合、原因が分かるように接続・読込・描画の痕跡を出す。
        await dump_startup_diagnostics(cdp, "初回マウント待ちタイムアウト")
        # ナビゲーションと Wasm init の競合が疑わしいため、再読み込みは 1 回だけ行う。
        # check は最後に 1 度だけ実行し、期待総数 229 を維持する。
        await cdp.send("Page.reload", {"ignoreCache": True})
        mounted = await wait_until(cdp, reader_selector)
        if not mounted:
            await dump_startup_diagnostics(cdp, "リロード後もマウントせず")
    checker.check("Reader ルートが表示される", mounted)
    if not mounted:
        return finish(checker)

    # 前回の実行で残った保存値を消してから、ファイルをセットする。
    await cdp.evaluate("localStorage.clear()")
    checker.check("EPUB のファイル入力を操作できる", await open_sample(cdp))

    opened = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    checker.check("EPUB を開いて第 1 章を表示する", opened)
    checker.check(
        "保存された履歴が無いときは Back が disabled",
        await cdp.evaluate("document.querySelector('#reader-back')?.disabled === true"),
    )
    if not opened:
        print("status:", await cdp.evaluate("document.querySelector('.reader-status')?.textContent"))
        await dump_startup_diagnostics(cdp, "EPUB 読込待ちタイムアウト")
        return finish(checker)

    # サニタイズ。
    checker.check(
        "script 要素が本文に残らない",
        await cdp.evaluate("document.querySelector('.reader-html script') === null"),
    )
    checker.check(
        "script が実行されない",
        await cdp.evaluate("window.__pwned === undefined"),
    )
    checker.check(
        "イベントハンドラが実行されない",
        await cdp.evaluate("window.__pwned2 === undefined"),
    )
    checker.check(
        "javascript: リンクの href が除去される",
        await cdp.evaluate("document.querySelector('#bad-js')?.getAttribute('href') === null"),
    )
    external_href = await cdp.evaluate(
        "document.querySelector('#external-link')?.getAttribute('href')"
    )
    checker.check(
        "外部リンクの href が保持される",
        external_href == "https://example.com",
        f"(href={external_href!r})",
    )
    external_rel = await cdp.evaluate(
        "document.querySelector('#external-link')?.getAttribute('rel') ?? ''"
    )
    checker.check(
        "外部リンクに noopener noreferrer が付く",
        "noopener" in str(external_rel) and "noreferrer" in str(external_rel),
        f"(rel={external_rel!r})",
    )
    checker.check(
        "外部画像の src が除去される",
        await cdp.evaluate("document.querySelector('#ext-img')?.getAttribute('src') === null"),
    )
    blob_src = await cdp.evaluate("document.querySelector('#pic')?.getAttribute('src') ?? ''")
    checker.check("相対画像が Blob URL になる", blob_src.startswith("blob:"), f"(src={blob_src!r})")
    image_loaded = await cdp.evaluate(
        "document.querySelector('#pic')?.naturalWidth === 1"
    )
    checker.check("Blob URL の画像が読み込まれる", image_loaded)

    # 目次と章タイトル。
    checker.equal(
        "目次の項目数",
        await cdp.evaluate("document.querySelectorAll('#reader-chapter-select option').length"),
        2,
    )
    checker.equal(
        "章タイトルの表示",
        await cdp.evaluate("document.querySelector('.reader-chapter-title')?.textContent"),
        "Chapter One",
    )

    # 相対リンクで次の章へ移動する。
    await cdp.evaluate("document.querySelector('#to-ch2').click()")
    moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("相対リンクで第 2 章へ移動する", moved)
    checker.equal(
        "移動後の章選択",
        await cdp.evaluate("document.querySelector('#reader-chapter-select')?.value"),
        "1",
    )
    checker.equal(
        "移動後の章タイトル",
        await cdp.evaluate("document.querySelector('.reader-chapter-title')?.textContent"),
        "Chapter Two",
    )
    checker.equal(
        "相対リンクのクリックでページ遷移しない",
        await cdp.evaluate("window.location.pathname"),
        "/reader/e2e",
    )

    # フラグメント付きの相対リンク。
    await cdp.evaluate("document.querySelector('#back').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await cdp.evaluate("document.querySelector('#to-ch2-part').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')"
    )
    scrolled = await wait_until(
        cdp, "document.querySelector('.reader-content')?.scrollTop > 0", timeout=5
    )
    scroll_top = await cdp.evaluate("document.querySelector('.reader-content')?.scrollTop ?? 0")
    checker.check("フラグメント付きリンクで移動先へスクロールする", scrolled, f"(scrollTop={scroll_top})")

    # 解決できない相対リンク。
    await cdp.evaluate("document.querySelector('#back').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await cdp.evaluate("document.querySelector('#missing').click()")
    await asyncio.sleep(0.3)
    status_text = await cdp.evaluate("document.querySelector('.reader-status')?.textContent ?? ''")
    checker.check(
        "解決できないリンクは移動せず状態を表示する",
        "cannot be opened" in status_text,
        f"(status={status_text!r})",
    )
    checker.check(
        "解決できないリンクでも章は変わらない",
        await cdp.evaluate(
            "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
        ),
    )

    # 読書位置の保存と復元。
    saved = await saved_position(cdp)
    checker.check("読書位置が localStorage に保存される", saved is not None)
    checker.check(
        "保存する値は章・フラグメント・スクロール位置・保存時刻だけ",
        saved is not None
        and sorted(saved.keys()) == ["chapterIndex", "fragment", "savedAt", "scrollTop"],
        f"(saved={saved!r})",
    )

    await cdp.evaluate("document.querySelector('#to-ch2').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    saved = await wait_saved(cdp, lambda position: position.get("chapterIndex") == 1)
    checker.equal("章の移動で位置が保存される", saved.get("chapterIndex") if saved else None, 1)

    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    restored = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("リロード後に移動した章が復元される", bool(restored))
    checker.equal(
        "復元後の章選択",
        await cdp.evaluate("document.querySelector('#reader-chapter-select')?.value"),
        "1",
    )

    await cdp.evaluate("document.querySelector('#back').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await cdp.evaluate("document.querySelector('#to-ch2-part').click()")
    saved = await wait_saved(cdp, lambda position: position.get("fragment") == "part")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')"
    )
    checker.equal(
        "フラグメントも位置として保存される",
        saved.get("fragment") if saved else None,
        "part",
    )
    # スクロールのデバウンス保存がフラグメントを消さない。
    await asyncio.sleep(0.6)
    stable = await saved_position(cdp)
    checker.equal(
        "スクロール保存後もフラグメントが残る",
        stable.get("fragment") if stable else None,
        "part",
    )

    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    restored_fragment = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')"
    )
    scrolled = await wait_until(
        cdp, "document.querySelector('.reader-content')?.scrollTop > 0", timeout=5
    )
    checker.check("フラグメント付きの位置が復元される", bool(restored_fragment))
    checker.check("復元時にフラグメントへスクロールする", scrolled)

    # 範囲外の保存値は先頭の章にフォールバックする。
    # 保存値は章の切り替えで上書きされるため、上書き後にページを再読み込みする。
    await set_sample_position(cdp, "{chapterIndex: 999, fragment: null}")
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    fallback = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    checker.check("範囲外の保存値では先頭の章になる", bool(fallback))

    # 旧形式（`savedAt` なし）の保存値でも章が復元される。
    await set_sample_position(cdp, "{chapterIndex: 1, fragment: null}")
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    legacy_restored = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("savedAt の無い旧形式でも章が復元される", bool(legacy_restored))

    # 目次の選択そのもので章を移動できる。
    await select_chapter(cdp, 1)
    dropdown_moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("目次の選択で章を移動できる", dropdown_moved)
    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )

    # 章内のスクロール位置の保存と復元。
    await cdp.evaluate("document.querySelector('#to-ch2').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    # 描画後のスクロール適用（先頭へ戻す）が終わるのを待ってからスクロールする。
    await asyncio.sleep(0.3)
    await cdp.evaluate("document.querySelector('.reader-content').scrollTop = 400")
    saved = await wait_saved(cdp, lambda position: position.get("scrollTop", 0) > 0)
    checker.check(
        "スクロール位置が保存される",
        saved is not None and saved.get("scrollTop", 0) > 0,
        f"(saved={saved!r})",
    )

    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    restored_scroll = reopened and await wait_until(
        cdp, "document.querySelector('.reader-content')?.scrollTop > 300", timeout=5
    )
    checker.check("リロード後にスクロール位置が復元される", bool(restored_scroll))
    restored_top = await cdp.evaluate("document.querySelector('.reader-content')?.scrollTop ?? 0")
    expected_top = saved.get("scrollTop", 0) if saved else 0
    checker.check(
        "復元後のスクロール位置が保存値と一致する",
        abs(restored_top - expected_top) <= 2,
        f"(restored={restored_top}, saved={expected_top})",
    )

    # 章を切り替えると先頭に戻る。
    await cdp.evaluate("document.querySelector('#back').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.3)
    saved = await saved_position(cdp)
    checker.equal("章の切り替えで保存値が先頭になる", saved.get("scrollTop") if saved else None, 0)
    await asyncio.sleep(1.0)
    checker.check(
        "章の切り替えでスクロール位置が先頭になる",
        (await cdp.evaluate("document.querySelector('.reader-content')?.scrollTop ?? -1")) == 0,
    )

    # scrollTop の無い旧形式の保存値でも章は復元される。
    await set_sample_position(cdp, "{chapterIndex: 1, fragment: null}")
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    legacy = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("旧形式の保存値でも章が復元される", bool(legacy))

    # 復元直後の描画後スクロールが終わるのを待ってから操作する。
    await asyncio.sleep(0.3)

    # 移動履歴（戻る・進む）。
    # 履歴が無い状態は最初のオープンで確認済み。ここでは機能を確認する。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.3)

    await cdp.evaluate("document.querySelector('.reader-content').scrollTop = 150")
    await asyncio.sleep(0.3)
    await cdp.evaluate("document.querySelector('#back').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.2)
    checker.check(
        "章を移動すると Back が有効になる",
        await cdp.evaluate("document.querySelector('#reader-back')?.disabled === false"),
    )

    await cdp.evaluate("document.querySelector('#reader-back').click()")
    restored_chapter = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("Back で前の章へ戻る", bool(restored_chapter))
    checker.equal(
        "Back で戻った章の選択",
        await cdp.evaluate("document.querySelector('#reader-chapter-select')?.value"),
        "1",
    )
    back_scrolled = await wait_until(
        cdp, "document.querySelector('.reader-content')?.scrollTop > 100", timeout=5
    )
    checker.check("Back でスクロール位置が復元される", back_scrolled)
    checker.check(
        "Back 後は Forward が有効になる",
        await cdp.evaluate("document.querySelector('#reader-forward')?.disabled === false"),
    )

    await cdp.evaluate("document.querySelector('#reader-forward').click()")
    forward_chapter = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    checker.check("Forward で元の章へ進む", bool(forward_chapter))

    # 戻った後に新しい移動をすると Forward は捨てられる。
    await cdp.evaluate("document.querySelector('#to-ch2').click()")
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.2)
    checker.check(
        "新しい移動で Forward が disabled になる",
        await cdp.evaluate("document.querySelector('#reader-forward')?.disabled === true"),
    )

    # 本を開き直すと履歴が復元される。
    reopened = await open_sample(cdp)
    restored = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("開き直すと章が復元される", bool(restored))
    checker.check(
        "開き直すと Back が有効になる",
        reopened and await cdp.evaluate("document.querySelector('#reader-back')?.disabled === false"),
    )
    # 復元した履歴で Back が動く。
    await cdp.evaluate("document.querySelector('#reader-back').click()")
    back_after_reload = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')", timeout=3
    )
    checker.check("復元した履歴で Back が動く", bool(back_after_reload))

    # 画像のレイアウトずれ補正。
    # 画像の後ろにある #part の位置を基準に確かめる。
    await set_sample_position(cdp, "{chapterIndex: 1, fragment: null, scrollTop: 0}")
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')"
    )
    await asyncio.sleep(0.8)
    tall_loaded = await cdp.evaluate(
        "document.querySelector('#tall')?.naturalHeight === 4000"
    )
    checker.check("縦長画像が読み込まれる", bool(tall_loaded))

    # 画像より後ろの見出しへスクロールし、その位置を保存する。
    await cdp.evaluate(
        "(() => {"
        " const content = document.querySelector('.reader-content');"
        " const part = document.querySelector('#part');"
        " content.scrollTop = part.offsetTop;"
        " })()"
    )
    before = await wait_saved(
        cdp, lambda position: position.get("scrollTop", 0) > 0
    )
    checker.check("画像を含む章でもスクロール位置が保存される", before is not None)
    marker_before = await cdp.evaluate(
        "(() => {"
        " const content = document.querySelector('.reader-content');"
        " const part = document.querySelector('#part');"
        " return part.getBoundingClientRect().top - content.getBoundingClientRect().top;"
        " })()"
    )
    saved_top = before.get("scrollTop", 0) if before else 0

    await reload_reader(cdp)
    # 復元時の最初のスクロールが、画像の読み込み完了後に行われるかを記録する。
    await cdp.evaluate(
        "(() => {"
        " window.__scrollOrder = {attempts: []};"
        " document.addEventListener('scroll', (event) => {"
        "  const target = event.target;"
        "  if (!target || target.scrollTop === undefined) return;"
        "  if (window.__scrollOrder.attempts.length > 20) return;"
        "  const image = document.querySelector('#tall');"
        "  window.__scrollOrder.attempts.push({top: target.scrollTop,"
        "   complete: image ? image.complete : null, t: performance.now()});"
        " }, true);"
        " const mark = () => { const image = document.querySelector('#tall');"
        "  if (image && !image.dataset.orderMarked) { image.dataset.orderMarked = '1';"
        "   const loaded = () => { window.__scrollOrder.loaded = performance.now(); };"
        "   if (image.complete) loaded(); else image.addEventListener('load', loaded, {once: true}); } };"
        " window.__scrollOrderTimer = setInterval(mark, 5);"
        " })()"
    )
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')"
    )
    # 画像の読み込みが終わるまで待ってから位置を比べる。
    await wait_until(cdp, "document.querySelector('#tall')?.naturalHeight === 4000", timeout=10)
    await asyncio.sleep(0.8)
    order = await cdp.evaluate("window.__scrollOrder")
    attempts = (order or {}).get("attempts") or []
    # DOM 更新直後にブラウザが行うスクロール調整は位置適用ではない。
    # 保存位置へ適用されたスクロールだけを評価する（検証は弱めない）。
    attempts = [
        entry
        for entry in attempts
        if entry.get("top", 0) > 0 and abs(entry.get("top", 0) - saved_top) <= 50
    ]
    checker.check(
        "位置の適用は画像の読み込み完了を待つ",
        bool(attempts) and all(entry.get("complete") is True for entry in attempts),
        f"(attempts={attempts!r}, saved_top={saved_top})",
    )
    marker_after = await cdp.evaluate(
        "(() => {"
        " const content = document.querySelector('.reader-content');"
        " const part = document.querySelector('#part');"
        " return part.getBoundingClientRect().top - content.getBoundingClientRect().top;"
        " })()"
    )
    restored_top = await cdp.evaluate("document.querySelector('.reader-content')?.scrollTop ?? 0")
    checker.check(
        "画像読み込み後も復元位置が同じ内容を指す",
        abs(marker_after - marker_before) <= 30,
        f"(before={marker_before}, after={marker_after}, saved={saved_top}, restored={restored_top})",
    )
    checker.check(
        "画像を含む章でもスクロール位置が復元される",
        abs(restored_top - saved_top) <= 30,
        f"(saved={saved_top}, restored={restored_top})",
    )

    # 表示設定。
    default_font = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    checker.equal("既定の文字サイズ", default_font, "18px")

    await cdp.evaluate("document.querySelector('#reader-font-larger').click()")
    enlarged = await wait_until(
        cdp,
        "getComputedStyle(document.querySelector('.reader-html')).fontSize === '20px'",
    )
    checker.check("A+ で文字サイズが上がる", enlarged)
    settings = await cdp.evaluate(
        "JSON.parse(localStorage.getItem('openrustbooks:settings') ?? 'null')"
    )
    checker.equal("文字サイズが保存される", settings.get("fontScale") if settings else None, 2)

    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    persisted_font = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    checker.equal("リロード後も文字サイズが維持される", persisted_font, "20px")

    await cdp.evaluate("document.querySelector('#reader-line-spacing').click()")
    line_spacing = await wait_until(
        cdp,
        "parseFloat(getComputedStyle(document.querySelector('.reader-html')).lineHeight) > 40",
    )
    checker.check("行間ボタンで行間が変わる", line_spacing)
    settings = await cdp.evaluate(
        "JSON.parse(localStorage.getItem('openrustbooks:settings') ?? 'null')"
    )
    checker.equal("行間が保存される", settings.get("lineHeight") if settings else None, 2)

    await cdp.evaluate("document.querySelector('#reader-theme').click()")
    dark = await wait_until(
        cdp,
        "document.querySelector('.reader-container')?.classList.contains('reader-dark') === true",
    )
    checker.check("ダーク表示に切り替わる", dark)
    settings = await cdp.evaluate(
        "JSON.parse(localStorage.getItem('openrustbooks:settings') ?? 'null')"
    )
    checker.equal("ダーク表示が保存される", settings.get("dark") if settings else None, True)

    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    persisted_dark = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark') === true"
    )
    checker.check("リロード後もダーク表示が維持される", persisted_dark)

    # 読書データの削除。
    position_keys = await cdp.evaluate(
        "Object.keys(localStorage).filter((key) => key.startsWith('openrustbooks:position:')).length"
    )
    checker.check("削除前に読書位置が保存されている", position_keys > 0)
    removed_keys = await cdp.evaluate(
        "(() => { document.querySelector('#reader-clear-data').click();"
        " return Object.keys(localStorage).filter((key) => key.startsWith('openrustbooks:position:')).length; })()"
    )
    checker.equal("Clear data で読書位置が削除される", removed_keys, 0)
    checker.check(
        "Clear data で表示設定は残る",
        await cdp.evaluate("localStorage.getItem('openrustbooks:settings') !== null"),
    )
    status_text = await cdp.evaluate("document.querySelector('.reader-status')?.textContent ?? ''")
    checker.check("削除後に状態メッセージが出る", "Removed" in status_text)

    # ARIA 属性。
    checker.check(
        "A- ボタンに aria-label と aria-keyshortcuts が付く",
        await cdp.evaluate(
            "document.querySelector('#reader-font-smaller')?.getAttribute('aria-label') === 'Smaller text'"
            " && document.querySelector('#reader-font-smaller')?.getAttribute('aria-keyshortcuts') === '-'"
        ),
    )
    checker.check(
        "A+ ボタンに aria-keyshortcuts が付く",
        await cdp.evaluate(
            "document.querySelector('#reader-font-larger')?.getAttribute('aria-keyshortcuts') === '+'"
        ),
    )
    checker.check(
        "本文領域に role・aria-label・tabindex が付く",
        await cdp.evaluate(
            "document.querySelector('.reader-content')?.getAttribute('role') === 'region'"
            " && document.querySelector('.reader-content')?.getAttribute('aria-label') === 'Chapter content'"
            " && document.querySelector('.reader-content')?.getAttribute('tabindex') === '0'"
        ),
    )
    checker.check(
        "状態メッセージに aria-live が付く",
        await cdp.evaluate(
            "document.querySelector('.reader-status')?.getAttribute('aria-live') === 'polite'"
        ),
    )
    checker.check(
        "章選択に aria-label が付く",
        await cdp.evaluate(
            "document.querySelector('#reader-chapter-select')?.getAttribute('aria-label') === 'Chapter'"
        ),
    )

    # キーボード操作。
    async def key(key: str, *, alt: bool = False, ctrl: bool = False) -> None:
        await cdp.evaluate(
            "(() => {"
            " const event = new KeyboardEvent('keydown', {"
            f"  key: {__import__('json').dumps(key)}, bubbles: true, cancelable: true,"
            f"  altKey: {str(alt).lower()}, ctrlKey: {str(ctrl).lower()}, metaKey: false"
            " });"
            " window.dispatchEvent(event);"
            " })()"
        )
        await asyncio.sleep(0.3)

    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await key("]")
    moved_next = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("] で次の章へ進む", moved_next)
    await key("[")
    moved_prev = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    checker.check("[ で前の章へ戻る", moved_prev)

    await key("+")
    font_after_plus = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    checker.check("+ で文字サイズが上がる", font_after_plus in ("20px", "22.5px", "24px", "27px", "31.5px"))
    await key("-")
    await asyncio.sleep(0.2)

    # ダーク表示を一度オフにしてから d でオンになることを確認する。
    if await cdp.evaluate("document.querySelector('.reader-container')?.classList.contains('reader-dark')"):
        await key("d")
        await asyncio.sleep(0.3)
    before_dark = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    await key("d")
    dark_after_d = await wait_until(
        cdp, "document.querySelector('.reader-container')?.classList.contains('reader-dark') !== " + str(before_dark).lower(), timeout=2
    )
    checker.check("d でダーク表示が切り替わる", dark_after_d)
    await key("d")
    await asyncio.sleep(0.2)

    # 目次の select にフォーカスがあるときは d が効かない。
    await cdp.evaluate("document.querySelector('.reader-chapter-select').focus()")
    before_dark = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    await key("d")
    await asyncio.sleep(0.2)
    after_dark = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    checker.check("select にフォーカス中は d が効かない", before_dark == after_dark)

    # Ctrl 併用時はキー操作が起きない。
    await cdp.evaluate("document.querySelector('.reader-content').focus()")
    before_font = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    await key("+", ctrl=True)
    await asyncio.sleep(0.2)
    after_font = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    checker.check("Ctrl 併用ではキー操作が起きない", before_font == after_font)

    # Alt+ArrowLeft / Alt+ArrowRight で履歴を移動できる。
    # 履歴に back エントリを作るため、まず章 0 に戻してから章 1 へ移動する。
    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.3)
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.3)
    await key("ArrowLeft", alt=True)
    back_moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')", timeout=3
    )
    checker.check("Alt+ArrowLeft で戻る", back_moved)
    await key("ArrowRight", alt=True)
    forward_moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')", timeout=3
    )
    checker.check("Alt+ArrowRight で進む", forward_moved)

    # アプリ全体のテーマ。
    # ダークを一度オフにしてからオンにする。
    if await cdp.evaluate("document.documentElement.classList.contains('dark')"):
        await cdp.evaluate("document.querySelector('#reader-theme').click()")
        await asyncio.sleep(0.3)
    await cdp.evaluate("document.querySelector('#reader-theme').click()")
    await asyncio.sleep(0.3)
    html_dark = await wait_until(
        cdp, "document.documentElement.classList.contains('dark') === true", timeout=3
    )
    checker.check("Dark で <html> に dark クラスが付く", html_dark)
    toolbar_bg = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-toolbar')).backgroundColor"
    )
    checker.check(
        "ダーク時にツールバーの背景がダークになる",
        "15, 23, 42" in toolbar_bg or "rgb(15, 23, 42)" in toolbar_bg or "1e293b" in toolbar_bg.lower() or "30, 41, 59" in toolbar_bg,
        f"(bg={toolbar_bg!r})",
    )

    # リロード後も dark クラスが維持される。
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    persisted_dark = await cdp.evaluate(
        "document.documentElement.classList.contains('dark')"
    )
    checker.check("リロード後も dark クラスが維持される", bool(persisted_dark))

    # Light で dark クラスが外れる。
    await cdp.evaluate("document.querySelector('#reader-theme').click()")
    await asyncio.sleep(0.3)
    html_light = await cdp.evaluate(
        "!document.documentElement.classList.contains('dark')"
    )
    checker.check("Light で dark クラスが外れる", bool(html_light))

    # タッチ操作（スワイプ）。
    async def swipe(dx: int, dy: int = 0, touches: int = 1) -> None:
        await cdp.evaluate(
            "(() => {"
            " const target = document.querySelector('.reader-content');"
            " const rect = target.getBoundingClientRect();"
            " const startX = rect.left + rect.width / 2;"
            " const startY = rect.top + rect.height / 2;"
            " const makeTouch = (x, y) => new Touch({"
            "  identifier: 0, target, clientX: x, clientY: y, pageX: x, pageY: y"
            " });"
            " let touchesList = [makeTouch(startX, startY)];"
            " if (arguments[2] > 1) touchesList.push(makeTouch(startX + 10, startY + 10));"
            " const startEvent = new TouchEvent('touchstart', {"
            "  touches: touchesList, targetTouches: touchesList, changedTouches: touchesList, bubbles: true, cancelable: true"
            " });"
            " target.dispatchEvent(startEvent);"
            " const endX = startX + arguments[0];"
            " const endY = startY + arguments[1];"
            " const endTouches = [makeTouch(endX, endY)];"
            " const endEvent = new TouchEvent('touchend', {"
            "  touches: [], targetTouches: [], changedTouches: endTouches, bubbles: true, cancelable: true"
            " });"
            " target.dispatchEvent(endEvent);"
            " })()",
            # pass dx, dy, touches via separate eval is not possible; inline them
        )
        # Re-dispatch with actual dx/dy/touches embedded.
        await cdp.evaluate(
            "(() => {"
            f" const dx = {dx}; const dy = {dy}; const nTouches = {touches};"
            " const target = document.querySelector('.reader-content');"
            " const rect = target.getBoundingClientRect();"
            " const startX = rect.left + rect.width / 2;"
            " const startY = rect.top + rect.height / 2;"
            " const makeTouch = (x, y, id) => new Touch({"
            "  identifier: id, target, clientX: x, clientY: y, pageX: x, pageY: y"
            " });"
            " const startTouches = [];"
            " for (let i = 0; i < nTouches; i += 1) startTouches.push(makeTouch(startX + i * 5, startY + i * 5, i));"
            " target.dispatchEvent(new TouchEvent('touchstart', {"
            "  touches: startTouches, targetTouches: startTouches, changedTouches: startTouches, bubbles: true, cancelable: true"
            " }));"
            " const endTouches = [makeTouch(startX + dx, startY + dy, 0)];"
            " target.dispatchEvent(new TouchEvent('touchend', {"
            "  touches: [], targetTouches: [], changedTouches: endTouches, bubbles: true, cancelable: true"
            " }));"
            " })()"
        )
        await asyncio.sleep(0.4)

    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await swipe(-60, 0)
    swiped_next = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')", timeout=3
    )
    checker.check("左スワイプで次の章へ進む", swiped_next)

    await swipe(60, 0)
    swiped_prev = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')", timeout=3
    )
    checker.check("右スワイプで前の章へ戻る", swiped_prev)

    # 縦移動が大きい場合は章が変わらない。
    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.3)
    await swipe(-60, 80)
    await asyncio.sleep(0.3)
    checker.check(
        "縦移動が大きいスワイプでは章が変わらない",
        await cdp.evaluate(
            "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
        ),
    )

    # マルチタッチでは章が変わらない。
    await swipe(-60, 0, touches=2)
    await asyncio.sleep(0.3)
    checker.check(
        "マルチタッチでは章が変わらない",
        await cdp.evaluate(
            "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
        ),
    )

    # 読書位置の保存時刻と掃除。
    await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    saved = await wait_saved(cdp, lambda position: position.get("savedAt", 0) > 0)
    now_ms = await cdp.evaluate("Date.now()")
    checker.check(
        "保存値に保存時刻が入る",
        saved is not None and abs(saved.get("savedAt", 0) - now_ms) < 120000,
        f"(saved={saved!r}, now={now_ms})",
    )

    # 45 件の偽エントリを入れてから保存すると、上限 40 件以下になる。
    # 保留中のデバウンス保存が終わるのを待つ。
    await asyncio.sleep(1.0)
    # 既存の position キーを消す（設定は残す）。
    await cdp.evaluate(
        "Object.keys(localStorage)"
        " .filter((k) => k.startsWith('openrustbooks:position:'))"
        " .forEach((k) => localStorage.removeItem(k))"
    )
    # 現在の本の位置を再作成する。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(1.0)

    await cdp.evaluate(
        "(() => {"
        " const now = Date.now();"
        " for (let i = 0; i < 45; i += 1) {"
        "  localStorage.setItem('openrustbooks:position:fake-' + i,"
        "   JSON.stringify({chapterIndex: 0, fragment: null, scrollTop: 0, savedAt: now - i * 60000}));"
        " }"
        " })()"
    )
    injected = await cdp.evaluate(
        "Object.keys(localStorage).filter((k) => k.startsWith('openrustbooks:position:')).length"
    )
    checker.check("掃除前に 40 件を超えている", injected > 40, f"(count={injected})")

    # select_chapter で保存を発生させる。
    await select_chapter(cdp, 0)
    pruned = await wait_until(
        cdp,
        "Object.keys(localStorage).filter((k) => k.startsWith('openrustbooks:position:')).length <= 40",
        timeout=10,
    )
    count_after = await cdp.evaluate(
        "Object.keys(localStorage).filter((k) => k.startsWith('openrustbooks:position:')).length"
    )
    checker.check("保存時に件数が上限以下に掃除される", pruned, f"(count={count_after})")
    current_kept = await cdp.evaluate(
        "Object.keys(localStorage).some((k) => k.startsWith('openrustbooks:position:sample.epub'))"
    )
    checker.check("現在の本の位置は残る", current_kept)

    # 古い順の掃除と混ざらないよう、件数テスト用の偽エントリを消してから
    # 寿命の判定を確認する。
    await cdp.evaluate(
        "Object.keys(localStorage)"
        " .filter((k) => k.startsWith('openrustbooks:position:fake-'))"
        " .forEach((k) => localStorage.removeItem(k))"
    )

    # 寿命の判定を確認する。
    await cdp.evaluate(
        "Object.keys(localStorage)"
        " .filter((k) => k.startsWith('openrustbooks:position:fake-'))"
        " .forEach((k) => localStorage.removeItem(k))"
    )
    await cdp.evaluate(
        "(() => {"
        " const old = Date.now() - 91 * 24 * 60 * 60 * 1000;"
        " const recent = Date.now() - 5 * 24 * 60 * 60 * 1000;"
        " localStorage.setItem('openrustbooks:position:aged-old',"
        "  JSON.stringify({chapterIndex: 0, fragment: null, scrollTop: 0, savedAt: old}));"
        " localStorage.setItem('openrustbooks:position:aged-recent',"
        "  JSON.stringify({chapterIndex: 0, fragment: null, scrollTop: 0, savedAt: recent}));"
        " })()"
    )
    await select_chapter(cdp, 1)
    removed_old = await wait_until(
        cdp, "localStorage.getItem('openrustbooks:position:aged-old') === null", timeout=5
    )
    checker.check("90 日より古い位置は削除される", removed_old)
    checker.check(
        "90 日以内の位置は残る",
        await cdp.evaluate("localStorage.getItem('openrustbooks:position:aged-recent') !== null"),
    )

    # 表示設定は掃除の対象にならない。
    checker.check(
        "掃除しても表示設定は残る",
        await cdp.evaluate("localStorage.getItem('openrustbooks:settings') !== null"),
    )

    # 読書位置のエクスポート・インポート。
    # まず章 1 に移動して位置を保存する。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.4)
    # Export ボタンが存在する。
    checker.check(
        "Export ボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-export') !== null"),
    )
    # Import ボタンが存在する。
    checker.check(
        "Import ボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-import') !== null"),
    )

    # インポート: positions JSON を直接 localStorage に書き込んでから
    # Import 経由で同じ値が取り込まれることを確認する。
    # まず現在の位置をクリアして、インポート用の JSON を作る。
    import json as _json
    payload = {
        "positions": {
            "openrustbooks:position:sample.epub:1:1": {
                "chapterIndex": 0,
                "fragment": None,
                "scrollTop": 123,
                "savedAt": 0,
            }
        },
        "settings": {
            "fontScale": 2,
            "lineHeight": 2,
            "dark": True,
        },
    }
    import_path = SAMPLE.parent / "positions.json"
    import_path.write_text(_json.dumps(payload), encoding="utf-8")
    print("IMPORT FILE CONTENT:", import_path.read_text())

    # Import ファイル入力をセットして change を発火する。
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send("DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"})
    node_id = node["result"]["nodeId"]
    checker.check("Import 用のファイル入力が存在する", node_id != 0)
    if node_id != 0:
        await cdp.send("DOM.setFileInputFiles", {"nodeId": node_id, "files": [str(import_path)]})
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent(new Event('change', {bubbles: true}))"
        )
        imported = await wait_until(
            cdp, "document.querySelector('.reader-status')?.textContent.includes('Imported')", timeout=5
        )
        checker.check("インポート後に状態メッセージが出る", imported)
        await asyncio.sleep(1.0)
        imported_value = await cdp.evaluate(
            "localStorage.getItem('openrustbooks:position:sample.epub:1:1')"
        )
        checker.check(
            "新形式のインポートで表示設定が復元される",
            await cdp.evaluate("localStorage.getItem('openrustbooks:settings') !== null"),
        )
        checker.check(
            "インポートで読書位置が取り込まれる",
            imported_value is not None and "123" in str(imported_value),
            f"(value={imported_value!r})",
        )

    # 不正な JSON では何も取り込まず、状態メッセージが出る。
    bad_path = SAMPLE.parent / "bad.json"
    bad_path.write_text("{not json", encoding="utf-8")
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send("DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"})
    node_id = node["result"]["nodeId"]
    if node_id != 0:
        await cdp.send("DOM.setFileInputFiles", {"nodeId": node_id, "files": [str(bad_path)]})
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent(new Event('change', {bubbles: true}))"
        )
        await asyncio.sleep(0.5)
        bad_status = await cdp.evaluate("document.querySelector('.reader-status')?.textContent ?? ''")
        checker.check(
            "不正な JSON ではエラーが表示される",
            "解析" in bad_status or "失敗" in bad_status or "not" in bad_status.lower(),
            f"(status={bad_status!r})",
        )

    # インポート後も表示設定は残る。
    checker.check(
        "インポート後も表示設定は残る",
        await cdp.evaluate("localStorage.getItem('openrustbooks:settings') !== null"),
    )

    # 旧形式（トップレベルに position キーが並ぶ）もインポートできる。
    old_payload = {"openrustbooks:position:sample.epub:9:9": {
        "chapterIndex": 1, "fragment": None, "scrollTop": 9, "savedAt": 0
    }}
    old_path = SAMPLE.parent / "old-positions.json"
    old_path.write_text(_json.dumps(old_payload), encoding="utf-8")
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send("DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"})
    node_id = node["result"]["nodeId"]
    if node_id != 0:
        await cdp.send("DOM.setFileInputFiles", {"nodeId": node_id, "files": [str(old_path)]})
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent(new Event('change', {bubbles: true}))"
        )
        await asyncio.sleep(0.5)
        old_value = await cdp.evaluate(
            "localStorage.getItem('openrustbooks:position:sample.epub:9:9')"
        )
        checker.check("旧形式の JSON もインポートできる", old_value is not None and "9" in str(old_value))

    # Landmarks。
    checker.check(
        "Landmarks ドロップダウンが存在する",
        await cdp.evaluate("document.querySelector('#reader-landmarks') !== null"),
    )
    landmarks_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-landmarks option').length"
    )
    checker.check("Landmarks が読み込まれている", landmarks_count > 1, f"(count={landmarks_count})")

    # Landmarks から本文開始へ移動する。
    await cdp.evaluate(
        "(() => {"
        " const select = document.querySelector('#reader-landmarks');"
        " select.value = '0';"
        " select.dispatchEvent(new Event('change', {bubbles: true}));"
        " })()"
    )
    moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')", timeout=5
    )
    checker.check("Landmarks の選択で章が移動する", moved)

    # Page-list。
    checker.check(
        "Page 入力が存在する",
        await cdp.evaluate("document.querySelector('#reader-page-input') !== null"),
    )
    checker.check(
        "Page 移動ボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-page-go') !== null"),
    )

    # ページ番号を入力して移動する。
    await cdp.evaluate("document.querySelector('#reader-page-input').value = '2'")
    await cdp.evaluate("document.querySelector('#reader-page-go').click()")
    moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')", timeout=5
    )
    checker.check("Page-list の選択で章が移動する", moved)

    # 存在しないページ番号では状態メッセージが出る。
    await cdp.evaluate("document.querySelector('#reader-page-input').value = '999'")
    await cdp.evaluate("document.querySelector('#reader-page-go').click()")
    await asyncio.sleep(0.5)
    status = await cdp.evaluate("document.querySelector('.reader-status')?.textContent || ''")
    checker.check("存在しないページ番号では状態メッセージが出る", "not found" in status.lower() or "見つかりません" in status)

    # 履歴一覧。
    checker.check(
        "履歴一覧が存在する",
        await cdp.evaluate("document.querySelector('#reader-history') !== null"),
    )

    # 前のテストで履歴が蓄積しているため、履歴キーを消してリロードしてから
    # 蓄積を確認する。
    await cdp.evaluate(
        "Object.keys(localStorage)"
        " .filter((k) => k.startsWith('openrustbooks:history:') || k.startsWith('openrustbooks:position:'))"
        " .forEach((k) => localStorage.removeItem(k))"
    )
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.5)
    before_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("リロード直後の履歴は 1 件", before_count == 1, f"(count={before_count}, reopened={reopened})")

    # 章を 2 回移動して履歴を 3 件にする。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.5)
    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.5)
    history_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("履歴一覧に 3 件入る", history_count == 3, f"(count={history_count})")

    # 一覧から最初のエントリを選ぶと最初の章へ移動する。
    await cdp.evaluate(
        "(() => {"
        " const select = document.querySelector('#reader-history');"
        " select.value = '0';"
        " select.dispatchEvent(new Event('change', {bubbles: true}));"
        " })()"
    )
    await asyncio.sleep(0.5)
    moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')", timeout=5
    )
    checker.check("履歴一覧から最初のエントリへ移動する", moved)

    # 履歴のエクスポート・インポート。
    # 履歴を作ってからインポートファイルを準備する。
    await cdp.evaluate(
        "(() => {"
        " localStorage.setItem('openrustbooks:history:sample.epub:1:1',"
        "  JSON.stringify({entries: [{chapterIndex: 0, fragment: null, scrollTop: 0},"
        "  {chapterIndex: 1, fragment: null, scrollTop: 10}], index: 1}));"
        " })()"
    )
    # 新形式のエクスポートに history が含まれるかはダウンロードファイルで
    # 確認できないため、localStorage から同等の内容を組み立てられることを
    # 確認する（エクスポートは同じキー走査を使う）。
    hist_raw = await cdp.evaluate(
        "localStorage.getItem('openrustbooks:history:sample.epub:1:1')"
    )
    checker.check(
        "エクスポート対象の履歴が保存されている",
        hist_raw is not None and "entries" in str(hist_raw) and "index" in str(hist_raw),
    )

    # 新形式のインポートで履歴が復元される。
    await cdp.evaluate(
        "localStorage.removeItem('openrustbooks:history:sample.epub:1:1')"
    )
    hist_payload = {
        "positions": {},
        "settings": {"fontScale": 1, "lineHeight": 2, "dark": False},
        "history": {
            "openrustbooks:history:sample.epub:1:1": {
                "entries": [
                    {"chapterIndex": 0, "fragment": None, "scrollTop": 0},
                    {"chapterIndex": 1, "fragment": None, "scrollTop": 10},
                ],
                "index": 1,
            }
        },
    }
    hist_path = SAMPLE.parent / "with-history.json"
    hist_path.write_text(_json.dumps(hist_payload), encoding="utf-8")
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send("DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"})
    node_id = node["result"]["nodeId"]
    if node_id != 0:
        await cdp.send("DOM.setFileInputFiles", {"nodeId": node_id, "files": [str(hist_path)]})
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent(new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp, "document.querySelector('.reader-status')?.textContent.includes('Imported')", timeout=5
        )
        restored = await cdp.evaluate(
            "localStorage.getItem('openrustbooks:history:sample.epub:1:1')"
        )
        checker.check(
            "新形式のインポートで履歴が復元される",
            restored is not None and '"entries"' in str(restored) and '"index":1' in str(restored).replace(" ", ""),
        )

    # 履歴のクリア。
    checker.check(
        "履歴クリアボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-history-clear') !== null"),
    )

    # 履歴 3 件の状態を作る（直前の履歴テストの状態を利用）。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.5)
    before_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("クリア前に履歴が 2 件以上", before_count >= 2, f"(count={before_count})")

    await cdp.evaluate("document.querySelector('#reader-history-clear').click()")
    await asyncio.sleep(0.3)
    await cdp.evaluate("document.querySelector('#reader-delete-confirm').click()")
    await asyncio.sleep(0.5)
    after_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("クリア後に履歴が 1 件になる", after_count == 1, f"(count={after_count})")
    single_entry_saved = await cdp.evaluate(
        "(() => Object.keys(localStorage)"
        ".filter((k) => k.startsWith('openrustbooks:history:'))"
        ".map((k) => JSON.parse(localStorage.getItem(k)))"
        ".some((v) => v && v.entries && v.entries.length === 1 && v.index === 0))()"
    )
    checker.check(
        "履歴 1 件が localStorage に保存される",
        single_entry_saved is True,
        f"(saved={single_entry_saved!r})",
    )
    back_disabled = await cdp.evaluate(
        "document.querySelector('#reader-back')?.disabled"
    )
    checker.check("クリア後に Back ボタンが disabled になる", back_disabled is True)
    text_kept = await cdp.evaluate(
        "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("クリア後も表示中の章は変わらない", text_kept is True)

    # インポートのマージ。既存の savedAt が新しい場合は上書きしない。
    await cdp.evaluate(
        "(() => {"
        " const now = Date.now();"
        " localStorage.setItem('openrustbooks:position:merge-test',"
        "  JSON.stringify({chapterIndex: 0, fragment: null, scrollTop: 555, savedAt: now}));"
        " localStorage.removeItem('openrustbooks:position:merge-new');"
        " })()"
    )
    merge_payload = {
        "positions": {
            "openrustbooks:position:merge-test": {
                "chapterIndex": 1,
                "fragment": None,
                "scrollTop": 111,
                "savedAt": 1000,
            },
            "openrustbooks:position:merge-new": {
                "chapterIndex": 0,
                "fragment": None,
                "scrollTop": 222,
                "savedAt": 1000,
            },
        },
        "settings": {"fontScale": 1, "lineHeight": 2, "dark": False},
    }
    merge_path = SAMPLE.parent / "merge-positions.json"
    merge_path.write_text(_json.dumps(merge_payload), encoding="utf-8")
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send("DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"})
    node_id = node["result"]["nodeId"]
    if node_id != 0:
        await cdp.send("DOM.setFileInputFiles", {"nodeId": node_id, "files": [str(merge_path)]})
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent(new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp, "document.querySelector('.reader-status')?.textContent.includes('Imported')", timeout=5
        )
        kept = await cdp.evaluate(
            "JSON.parse(localStorage.getItem('openrustbooks:position:merge-test'))"
        )
        checker.check(
            "既存の savedAt が新しい位置は上書きされない",
            kept is not None and kept.get("scrollTop") == 555,
            f"(kept={kept!r})",
        )
        imported_new = await cdp.evaluate(
            "JSON.parse(localStorage.getItem('openrustbooks:position:merge-new'))"
        )
        checker.check(
            "既存キーが無い位置は取り込まれる",
            imported_new is not None and imported_new.get("scrollTop") == 222,
            f"(new={imported_new!r})",
        )
        # 後続のテストに影響しないよう片付ける。
        await cdp.evaluate(
            "localStorage.removeItem('openrustbooks:position:merge-test');"
            "localStorage.removeItem('openrustbooks:position:merge-new')"
        )

    # OS のカラースキーム追従。
    checker.check(
        "カラースキーム追従ボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-theme-auto') !== null"),
    )
    # エミュレーションで OS をダークにする。
    await cdp.send(
        "Emulation.setEmulatedMedia",
        {"features": [{"name": "prefers-color-scheme", "value": "dark"}]},
    )
    await cdp.evaluate("document.querySelector('#reader-theme-auto').click()")
    await asyncio.sleep(0.5)
    auto_pressed = await cdp.evaluate(
        "document.querySelector('#reader-theme-auto')?.getAttribute('aria-pressed')"
    )
    checker.check("追従ボタンが押下状態になる", auto_pressed == "true")
    follows_dark = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark') === true"
    )
    checker.check("OS がダークなら追従してダーク表示になる", follows_dark)

    # 設定が保存される。
    saved_follow = await cdp.evaluate(
        "(() => { const s = JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}');"
        " return s.followSystem === true; })()"
    )
    checker.check("追従の選択が保存される", saved_follow)

    # OS をライトに切り替えるとリアルタイムで追従する。
    await cdp.send(
        "Emulation.setEmulatedMedia",
        {"features": [{"name": "prefers-color-scheme", "value": "light"}]},
    )
    await asyncio.sleep(0.5)
    follows_light = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark') === false"
    )
    checker.check("OS がライトなら追従してライト表示になる", follows_light)

    # 手動トグルで追従が解除される。
    await cdp.evaluate("document.querySelector('#reader-theme').click()")
    await asyncio.sleep(0.5)
    manual_pressed = await cdp.evaluate(
        "document.querySelector('#reader-theme-auto')?.getAttribute('aria-pressed')"
    )
    checker.check("手動切替で追従が解除される", manual_pressed == "false")

    # エミュレーションを戻す。
    await cdp.send("Emulation.setEmulatedMedia", {"features": []})

    # 本文画像は非同期デコードを指定し、loading="lazy" は付けない。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')"
    )
    await asyncio.sleep(0.8)
    decoding_attrs = await cdp.evaluate(
        "Array.from(document.querySelectorAll('.reader-html img[src]')).map((img) => img.getAttribute('decoding'))"
    )
    checker.check(
        "本文画像に decoding=\"async\" が付いている",
        len(decoding_attrs) > 0 and all(attr == "async" for attr in decoding_attrs),
        f"(attrs={decoding_attrs!r})",
    )
    lazy_attrs = await cdp.evaluate(
        "Array.from(document.querySelectorAll('.reader-html img[src]')).map((img) => img.getAttribute('loading'))"
    )
    checker.check(
        "本文画像に loading=\"lazy\" は付いていない",
        all(attr is None for attr in lazy_attrs),
        f"(attrs={lazy_attrs!r})",
    )

    # 履歴エントリの個別削除。
    await select_chapter(cdp, 0)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    await asyncio.sleep(0.5)
    before_delete = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("削除前に履歴が 2 件以上", before_delete >= 2, f"(count={before_delete})")

    await cdp.evaluate("document.querySelector('#reader-history-delete').click()")
    await asyncio.sleep(0.3)
    await cdp.evaluate("document.querySelector('#reader-delete-confirm').click()")
    await asyncio.sleep(0.5)
    after_delete = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check(
        "個別削除で履歴が 1 件減る",
        after_delete == before_delete - 1,
        f"(before={before_delete}, after={after_delete})",
    )
    text_kept = await cdp.evaluate(
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    checker.check("個別削除後も表示中の章は変わらない", text_kept is True)

    # 履歴が 1 件になるまで削除を繰り返し、disabled を確認する。
    for _ in range(5):
        count = await cdp.evaluate(
            "document.querySelectorAll('#reader-history option').length"
        )
        if count <= 1:
            break
        await cdp.evaluate("document.querySelector('#reader-history-delete').click()")
        await asyncio.sleep(0.2)
        await cdp.evaluate("document.querySelector('#reader-delete-confirm').click()")
        await asyncio.sleep(0.3)
    final_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("履歴が 1 件まで減らせる", final_count == 1, f"(count={final_count})")
    delete_disabled = await cdp.evaluate(
        "document.querySelector('#reader-history-delete')?.disabled"
    )
    checker.check("履歴が 1 件のとき削除ボタンが disabled になる", delete_disabled is True)

    # 削除確認ダイアログ：キャンセルでは何も消えない。
    # 履歴を 3 件にする。
    await select_chapter(cdp, 1)
    await asyncio.sleep(0.5)
    await select_chapter(cdp, 0)
    await asyncio.sleep(0.5)
    before_cancel = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    await cdp.evaluate("document.querySelector('#reader-history-delete').click()")
    await asyncio.sleep(0.3)
    dialog_open = await cdp.evaluate(
        "document.querySelector('#reader-delete-dialog')?.open === true"
    )
    checker.check("削除ボタンで確認ダイアログが開く", dialog_open is True)
    # 安全な操作（Cancel）にフォーカスが当たる（誤って Delete を押さないように）。
    focused_cancel = await cdp.evaluate(
        "document.activeElement?.textContent === 'Cancel'"
    )
    checker.check("確認ダイアログで Cancel にフォーカスが当たる", focused_cancel is True)
    await cdp.evaluate(
        "Array.from(document.querySelectorAll('#reader-delete-dialog button'))"
        ".find((b) => b.textContent.includes('Cancel')).click()"
    )
    await asyncio.sleep(0.3)
    after_cancel = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check(
        "キャンセルすると履歴件数が変わらない",
        after_cancel == before_cancel,
        f"(before={before_cancel}, after={after_cancel})",
    )

    # Esc でもキャンセルとして扱われ、履歴は変わらない。
    await cdp.evaluate("document.querySelector('#reader-history-delete').click()")
    await asyncio.sleep(0.2)
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyDown", "key": "Escape", "code": "Escape", "windowsVirtualKeyCode": 27},
    )
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyUp", "key": "Escape", "code": "Escape", "windowsVirtualKeyCode": 27},
    )
    await asyncio.sleep(0.3)
    dialog_closed = await cdp.evaluate(
        "document.querySelector('#reader-delete-dialog')?.open === false"
    )
    checker.check("Esc で確認ダイアログが閉じる", dialog_closed is True)
    after_esc = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check(
        "Esc では履歴件数が変わらない",
        after_esc == before_cancel,
        f"(before={before_cancel}, after={after_esc})",
    )

    # 確認ボタンで削除が実行される（個別削除テストで確認済みだが、
    # ダイアログ経由であることを明示的に確認する）。
    await cdp.evaluate("document.querySelector('#reader-history-delete').click()")
    await asyncio.sleep(0.2)
    await cdp.evaluate("document.querySelector('#reader-delete-confirm').click()")
    await asyncio.sleep(0.4)
    after_confirm = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check(
        "確認ダイアログで削除が実行される",
        after_confirm == before_cancel - 1,
        f"(before={before_cancel}, after={after_confirm})",
    )

    # エクスポート形式のバージョニング。
    # version: 1 は取り込める。
    await cdp.evaluate("localStorage.removeItem('openrustbooks:position:version-test')")
    version_payload = {
        "version": 1,
        "positions": {
            "openrustbooks:position:version-test": {
                "chapterIndex": 0,
                "fragment": None,
                "scrollTop": 77,
                "savedAt": 1000,
            }
        },
        "settings": {"fontScale": 1, "lineHeight": 2, "dark": False},
        "history": {},
    }
    version_path = SAMPLE.parent / "version-1.json"
    version_path.write_text(_json.dumps(version_payload), encoding="utf-8")
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id = document["result"]["root"]["nodeId"]
    node = await cdp.send(
        "DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"}
    )
    if node["result"]["nodeId"] != 0:
        await cdp.send(
            "DOM.setFileInputFiles",
            {"nodeId": node["result"]["nodeId"], "files": [str(version_path)]},
        )
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent("
            "new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp,
            "document.querySelector('.reader-status')?.textContent.includes('Imported')",
            timeout=5,
        )
        imported_v1 = await cdp.evaluate(
            "JSON.parse(localStorage.getItem('openrustbooks:position:version-test'))"
        )
        checker.check(
            "version: 1 の JSON がインポートできる",
            imported_v1 is not None and imported_v1.get("scrollTop") == 77,
            f"(imported={imported_v1!r})",
        )

    # 未知のバージョンは拒否され、既存データが壊れない。
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:position:version-test',"
        " JSON.stringify({chapterIndex: 0, fragment: null, scrollTop: 999, savedAt: 9999}))"
    )
    unknown_payload = {"version": 999, "positions": {}, "settings": {}, "history": {}}
    unknown_path = SAMPLE.parent / "version-unknown.json"
    unknown_path.write_text(_json.dumps(unknown_payload), encoding="utf-8")
    node = await cdp.send(
        "DOM.querySelector", {"nodeId": root_id, "selector": "#reader-import-file"}
    )
    if node["result"]["nodeId"] != 0:
        await cdp.send(
            "DOM.setFileInputFiles",
            {"nodeId": node["result"]["nodeId"], "files": [str(unknown_path)]},
        )
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent("
            "new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp,
            "document.querySelector('.reader-status')?.textContent.includes('Unsupported')",
            timeout=5,
        )
        kept_v1 = await cdp.evaluate(
            "JSON.parse(localStorage.getItem('openrustbooks:position:version-test'))"
        )
        checker.check(
            "未知のバージョンは拒否され既存データが壊れない",
            kept_v1 is not None and kept_v1.get("scrollTop") == 999,
            f"(kept={kept_v1!r})",
        )
        await cdp.evaluate("localStorage.removeItem('openrustbooks:position:version-test')")

    # 初回起動の追従デフォルト。設定キーが無く OS がダークなら有効になる。
    await cdp.send(
        "Emulation.setEmulatedMedia",
        {"features": [{"name": "prefers-color-scheme", "value": "dark"}]},
    )
    await cdp.evaluate("localStorage.removeItem('openrustbooks:settings')")
    await reload_reader(cdp)
    await asyncio.sleep(0.5)
    # 初回デフォルトは localStorage に保存されず UI 状態だけに反映される。
    initial_follow = await cdp.evaluate(
        "document.querySelector('#reader-theme-auto')?.getAttribute('aria-pressed')"
    )
    checker.check(
        "初回起動で OS がダークなら追従が有効になる",
        initial_follow == "true",
        f"(aria-pressed={initial_follow!r})",
    )

    # 設定キーが存在し followSystem が無い場合は、従来どおり追従しない。
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:settings',"
        " JSON.stringify({fontScale: 1, lineHeight: 2, dark: false}))"
    )
    await reload_reader(cdp)
    await asyncio.sleep(0.5)
    legacy_follow = await cdp.evaluate(
        "document.querySelector('#reader-theme-auto')?.getAttribute('aria-pressed')"
    )
    checker.check(
        "既存設定に followSystem が無い場合は追従しない（後方互換）",
        legacy_follow == "false",
        f"(aria-pressed={legacy_follow!r})",
    )
    await cdp.send("Emulation.setEmulatedMedia", {"features": []})

    # 強制カラーモード（Windows ハイコントラスト）でも表示と操作ができる。
    await cdp.send(
        "Emulation.setEmulatedMedia",
        {"features": [{"name": "forced-colors", "value": "active"}]},
    )
    await asyncio.sleep(0.3)
    forced_active = await cdp.evaluate(
        "window.matchMedia('(forced-colors: active)').matches"
    )
    checker.check("強制カラーモードが有効になる", forced_active is True)
    visible = await cdp.evaluate(
        "(() => { const el = document.querySelector('#reader-theme');"
        " if (!el) return false;"
        " const st = getComputedStyle(el);"
        " return st.borderTopWidth !== '0px' && st.visibility !== 'hidden'; })()"
    )
    checker.check("強制カラーモードでもボタンの境界が残る", visible is True)
    # 直前のテストでリロードしているため、EPUB を開き直す。
    await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await asyncio.sleep(0.5)
    await select_chapter(cdp, 1)
    moved = await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')",
        timeout=5,
    )
    checker.check("強制カラーモードでも章移動ができる", moved)
    await cdp.send("Emulation.setEmulatedMedia", {"features": []})

    # 履歴サマリーの日時表示。章移動したエントリに日時が入る。
    await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await asyncio.sleep(0.5)
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')",
        timeout=5,
    )
    await asyncio.sleep(0.5)
    labels = await cdp.evaluate(
        "Array.from(document.querySelectorAll('#reader-history option')).map((o) => o.textContent)"
    )
    import re as _re
    has_time = any(
        _re.search(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}", str(label)) for label in labels
    )
    checker.check(
        "履歴一覧のラベルに保存日時が表示される",
        has_time,
        f"(labels={labels!r})",
    )

    # SVG Blob URL のサニタイズ。スクリプトを含む SVG を開いた章で確認する。
    # 直前のテストで ch1 にいるため ch2 へ移動する。
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('#evil-svg')?.src.startsWith('blob:') === true",
        timeout=5,
    )
    svg_src = await cdp.evaluate("document.querySelector('#evil-svg')?.src")
    svg_text = await cdp.evaluate(
        "fetch(document.querySelector('#evil-svg').src).then((r) => r.text())"
    )
    checker.check("SVG が Blob URL で表示される", str(svg_src or "").startswith("blob:"))
    lowered = str(svg_text or "").lower()
    checker.check(
        "SVG から script が除去される",
        "<script" not in lowered,
        f"(svg={str(svg_text)[:120]!r})",
    )
    checker.check(
        "SVG から on* 属性が除去される",
        "onload=" not in lowered and "onclick=" not in lowered,
        f"(svg={str(svg_text)[:120]!r})",
    )
    checker.check(
        "SVG から foreignObject が除去される",
        "foreignobject" not in lowered,
        f"(svg={str(svg_text)[:120]!r})",
    )
    checker.check(
        "SVG から javascript: リンクが除去される",
        "javascript:" not in lowered and "script:" not in lowered.replace("\n", ""),
        f"(svg={str(svg_text)[:120]!r})",
    )
    checker.check(
        "SVG から SMIL 属性書き換えが除去される",
        "<set" not in lowered and "<animate" not in lowered,
        f"(svg={str(svg_text)[:120]!r})",
    )
    checker.check(
        "SVG から外部スキーム参照が除去される",
        "attacker.example" not in lowered and "data:image" not in lowered,
        f"(svg={str(svg_text)[:200]!r})",
    )
    checker.check(
        "SVG の相対参照は維持される",
        "images/pic.png" in str(svg_text),
        f"(svg={str(svg_text)[:200]!r})",
    )
    checker.check(
        "SVG から xml-stylesheet 処理命令が除去される",
        "xml-stylesheet" not in str(svg_text).lower()
        and "attacker.example/style.css" not in str(svg_text).lower(),
        f"(svg={str(svg_text)[:200]!r})",
    )
    pwned = await cdp.evaluate("window.__pwned === 1")
    checker.check("SVG のスクリプトが実行されない", pwned is False)

    # エクスポート→インポートのラウンドトリップ。
    download_dir = SAMPLE.parent / "downloads"
    download_dir.mkdir(parents=True, exist_ok=True)
    await cdp.send(
        "Browser.setDownloadBehavior",
        {"behavior": "allow", "downloadPath": str(download_dir), "eventsEnabled": True},
    )
    # 書き出す内容を作る（設定・履歴・位置）。
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:settings',"
        " JSON.stringify({fontScale: 2, lineHeight: 1, dark: true, followSystem: false}))"
    )
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:position:sample.epub:1:1',"
        " JSON.stringify({chapterIndex: 1, fragment: null, scrollTop: 33, savedAt: 1000}))"
    )
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:history:sample.epub:1:1',"
        " JSON.stringify({entries: [{chapterIndex: 0, fragment: null, scrollTop: 0, savedAt: 100},"
        " {chapterIndex: 1, fragment: null, scrollTop: 0, savedAt: 200}], index: 1}))"
    )
    await cdp.evaluate("document.querySelector('#reader-export').click()")
    # ダウンロード完了まで待つ。
    exported = None
    deadline = time.time() + 10
    while time.time() < deadline:
        candidates = sorted(download_dir.glob("*.json"))
        if candidates:
            exported = candidates[0]
            break
        await asyncio.sleep(0.2)
    checker.check("エクスポートファイルがダウンロードされる", exported is not None)
    if exported is not None:
        import re as _re_export
        checker.check(
            "エクスポート名にタイムスタンプが付く",
            _re_export.fullmatch(
                r"openrustbooks-\d{8}-\d{6}[+-]\d{4}(?: \(\d+\))?\.json",
                exported.name,
            )
            is not None,
            f"(filename={exported.name!r})",
        )
        payload = _json.loads(exported.read_text(encoding="utf-8"))
        checker.check("エクスポートに形式バージョンが含まれる", payload.get("version") == 1)
        checker.check(
            "エクスポートに位置・設定・履歴が含まれる",
            "positions" in payload
            and payload.get("settings", {}).get("fontScale") == 2
            and "history" in payload,
        )
        # 書き出したファイルをそのままインポートする。
        await cdp.evaluate("localStorage.clear()")
        document = await cdp.send("DOM.getDocument", {"depth": -1})
        root_id2 = document["result"]["root"]["nodeId"]
        node = await cdp.send(
            "DOM.querySelector",
            {"nodeId": root_id2, "selector": "#reader-import-file"},
        )
        if node["result"]["nodeId"] != 0:
            await cdp.send(
                "DOM.setFileInputFiles",
                {"nodeId": node["result"]["nodeId"], "files": [str(exported)]},
            )
            await cdp.evaluate(
                "document.querySelector('#reader-import-file').dispatchEvent("
                "new Event('change', {bubbles: true}))"
            )
            await wait_until(
                cdp,
                "document.querySelector('.reader-status')?.textContent.includes('Imported')",
                timeout=5,
            )
            restored = await cdp.evaluate(
                "(() => ({"
                " settings: JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}'),"
                " history: localStorage.getItem('openrustbooks:history:sample.epub:1:1'),"
                " position: localStorage.getItem('openrustbooks:position:sample.epub:1:1')"
                " }))()"
            )
            checker.check(
                "ラウンドトリップで表示設定が復元される",
                restored["settings"].get("fontScale") == 2,
                f"(settings={restored['settings']!r})",
            )
            checker.check(
                "ラウンドトリップで履歴が復元される",
                restored["history"] is not None and '"entries"' in str(restored["history"]),
            )
            checker.check(
                "ラウンドトリップで読書位置が復元される",
                restored["position"] is not None,
            )
    shutil.rmtree(download_dir, ignore_errors=True)

    # 同一章内のフラグメント移動も履歴に記録される。
    await open_sample(cdp)
    await wait_until(
        cdp,
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await select_chapter(cdp, 1)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Part two text.')",
        timeout=5,
    )
    await asyncio.sleep(0.5)
    before_anchor = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    await cdp.evaluate(
        "document.querySelector('#part').scrollIntoView()"
    )
    await asyncio.sleep(0.3)
    await cdp.evaluate("document.querySelector('#to-start').click()")
    await asyncio.sleep(0.8)
    after_anchor = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check(
        "同一章のフラグメント移動で履歴が 1 件増える",
        after_anchor == before_anchor + 1,
        f"(before={before_anchor}, after={after_anchor})",
    )
    back_enabled = await cdp.evaluate(
        "document.querySelector('#reader-back')?.disabled === false"
    )
    checker.check("同一章のフラグメント移動後に戻れる", back_enabled is True)
    # 戻ると #part 付近へ戻る。
    await cdp.evaluate("document.querySelector('#reader-back').click()")
    await asyncio.sleep(1.0)
    restored_scroll = await cdp.evaluate(
        "document.querySelector('.reader-content')?.scrollTop ?? 0"
    )
    checker.check(
        "戻るでフラグメント移動前の位置へ戻る",
        restored_scroll > 0,
        f"(scrollTop={restored_scroll})",
    )

    # スワイプ閾値の設定。
    checker.check(
        "スワイプ閾値の入力が存在する",
        await cdp.evaluate("document.querySelector('#reader-swipe-threshold') !== null"),
    )
    await cdp.evaluate(
        "(() => {"
        " const input = document.querySelector('#reader-swipe-threshold');"
        " input.value = '80';"
        " input.dispatchEvent(new Event('change', {bubbles: true}));"
        " })()"
    )
    await asyncio.sleep(0.5)
    saved_threshold = await cdp.evaluate(
        "(() => { const s = JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}');"
        " return s.swipeThreshold; })()"
    )
    checker.check(
        "スワイプ閾値が設定に保存される",
        saved_threshold == 80,
        f"(swipeThreshold={saved_threshold!r})",
    )
    # 範囲外はクランプされる。
    await cdp.evaluate("localStorage.clear()")
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:settings',"
        " JSON.stringify({swipeThreshold: 999}))"
    )
    await reload_reader(cdp)
    await asyncio.sleep(0.5)
    clamped = await cdp.evaluate(
        "document.querySelector('#reader-swipe-threshold')?.value"
    )
    checker.check(
        "範囲外のスワイプ閾値は 120 にクランプされる",
        clamped == "120",
        f"(value={clamped!r})",
    )

    # 本文言語の設定。サンプル EPUB は言語 ja。
    await cdp.evaluate("localStorage.clear()")
    await open_sample(cdp)
    await wait_until(
        cdp,
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await asyncio.sleep(0.5)
    content_lang = await cdp.evaluate(
        "document.querySelector('.reader-html')?.getAttribute('lang')"
    )
    checker.check(
        "本文要素に EPUB の言語が設定される",
        content_lang == "ja",
        f"(lang={content_lang!r})",
    )
    html_lang = await cdp.evaluate("document.documentElement.getAttribute('lang')")
    checker.check(
        "UI 全体の言語は変更されない",
        html_lang == "ja",
        f"(html lang={html_lang!r})",
    )

    # 本文領域のアクセシブル名が章タイトルを指す。
    labelled_by = await cdp.evaluate(
        "document.querySelector('.reader-content')?.getAttribute('aria-labelledby')"
    )
    checker.check(
        "本文領域の aria-labelledby が章タイトルを指す",
        labelled_by == "reader-chapter-title",
        f"(aria-labelledby={labelled_by!r})",
    )
    label_text = await cdp.evaluate(
        "document.getElementById('reader-chapter-title')?.textContent || ''"
    )
    checker.check(
        "参照先の章タイトルが入っている",
        "Chapter" in str(label_text),
        f"(text={label_text!r})",
    )

    # 縦スクロール閾値の設定。
    checker.check(
        "縦スクロール閾値の入力が存在する",
        await cdp.evaluate("document.querySelector('#reader-vertical-threshold') !== null"),
    )
    await cdp.evaluate(
        "(() => {"
        " const input = document.querySelector('#reader-vertical-threshold');"
        " input.value = '60';"
        " input.dispatchEvent(new Event('change', {bubbles: true}));"
        " })()"
    )
    await asyncio.sleep(0.5)
    saved_vertical = await cdp.evaluate(
        "(() => { const s = JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}');"
        " return s.verticalScrollThreshold; })()"
    )
    checker.check(
        "縦スクロール閾値が設定に保存される",
        saved_vertical == 60,
        f"(verticalScrollThreshold={saved_vertical!r})",
    )
    await cdp.evaluate("localStorage.clear()")
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:settings',"
        " JSON.stringify({verticalScrollThreshold: 999}))"
    )
    await reload_reader(cdp)
    await asyncio.sleep(0.5)
    clamped_vertical = await cdp.evaluate(
        "document.querySelector('#reader-vertical-threshold')?.value"
    )
    checker.check(
        "範囲外の縦スクロール閾値は 80 にクランプされる",
        clamped_vertical == "80",
        f"(value={clamped_vertical!r})",
    )

    # ブラウザ標準のピンチズームが阻害されていない。
    viewport = await cdp.evaluate(
        "document.querySelector('meta[name=viewport]')?.content || ''"
    )
    checker.check(
        "viewport がズームを許可している",
        "user-scalable=no" not in viewport.lower()
        and "maximum-scale=1" not in viewport.lower(),
        f"(viewport={viewport!r})",
    )
    touch_action = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-content')).touchAction"
    )
    checker.check(
        "本文領域の touch-action が none でない",
        str(touch_action).lower() != "none",
        f"(touch-action={touch_action!r})",
    )

    # 旧形式インポート → エクスポート（version 付与）→ 再インポートの連鎖。
    await cdp.evaluate("localStorage.clear()")
    legacy_payload = {
        "openrustbooks:position:sample.epub:legacy": {
            "chapterIndex": 0,
            "fragment": None,
            "scrollTop": 17,
            "savedAt": 2000,
        }
    }
    legacy_path = SAMPLE.parent / "legacy-roundtrip.json"
    legacy_path.write_text(_json.dumps(legacy_payload), encoding="utf-8")
    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id_legacy = document["result"]["root"]["nodeId"]
    node = await cdp.send(
        "DOM.querySelector",
        {"nodeId": root_id_legacy, "selector": "#reader-import-file"},
    )
    if node["result"]["nodeId"] != 0:
        # 旧形式をインポートする。
        await cdp.send(
            "DOM.setFileInputFiles",
            {"nodeId": node["result"]["nodeId"], "files": [str(legacy_path)]},
        )
        await cdp.evaluate(
            "document.querySelector('#reader-import-file').dispatchEvent("
            "new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp,
            "document.querySelector('.reader-status')?.textContent.includes('Imported')",
            timeout=5,
        )
        legacy_pos = await cdp.evaluate(
            "JSON.parse(localStorage.getItem('openrustbooks:position:sample.epub:legacy'))"
        )
        checker.check(
            "旧形式をインポートすると位置が復元される",
            legacy_pos is not None and legacy_pos.get("scrollTop") == 17,
            f"(legacy={legacy_pos!r})",
        )

        # エクスポートして version: 1 が付くことを確認する。
        legacy_dl = SAMPLE.parent / "legacy-downloads"
        legacy_dl.mkdir(parents=True, exist_ok=True)
        await cdp.send(
            "Browser.setDownloadBehavior",
            {"behavior": "allow", "downloadPath": str(legacy_dl), "eventsEnabled": True},
        )
        await cdp.evaluate("document.querySelector('#reader-export').click()")
        exported_path = None
        deadline = time.time() + 10
        while time.time() < deadline:
            candidates = sorted(legacy_dl.glob("*.json"))
            if candidates:
                exported_path = candidates[0]
                break
            await asyncio.sleep(0.2)
        checker.check("旧形式インポート後にエクスポートできる", exported_path is not None)
        if exported_path is not None:
            exported_payload = _json.loads(exported_path.read_text(encoding="utf-8"))
            checker.check(
                "旧形式由来のエクスポートに version: 1 が付く",
                exported_payload.get("version") == 1
                and "openrustbooks:position:sample.epub:legacy"
                in exported_payload.get("positions", {}),
            )
            # そのファイルを再インポートしても位置が残る。
            await cdp.evaluate("localStorage.clear()")
            node = await cdp.send(
                "DOM.querySelector",
                {"nodeId": root_id_legacy, "selector": "#reader-import-file"},
            )
            if node["result"]["nodeId"] != 0:
                await cdp.send(
                    "DOM.setFileInputFiles",
                    {
                        "nodeId": node["result"]["nodeId"],
                        "files": [str(exported_path)],
                    },
                )
                await cdp.evaluate(
                    "document.querySelector('#reader-import-file').dispatchEvent("
                    "new Event('change', {bubbles: true}))"
                )
                await wait_until(
                    cdp,
                    "document.querySelector('.reader-status')?.textContent.includes('Imported')",
                    timeout=5,
                )
                reimported = await cdp.evaluate(
                    "JSON.parse(localStorage.getItem('openrustbooks:position:sample.epub:legacy'))"
                )
                checker.check(
                    "旧形式由来のエクスポートを再インポートできる",
                    reimported is not None and reimported.get("scrollTop") == 17,
                    f"(reimported={reimported!r})",
                )
        shutil.rmtree(legacy_dl, ignore_errors=True)

    # 閾値ちょうどの画像では警告が出ない（既存サンプル 4000px）。
    await cdp.evaluate("localStorage.clear()")
    await open_sample(cdp)
    await wait_until(
        cdp,
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await asyncio.sleep(1.0)
    no_warning = await cdp.evaluate(
        "(() => { const s = document.querySelector('.reader-status')?.textContent || '';"
        " return !s.includes('Large image detected'); })()"
    )
    checker.check("4000px の画像では警告が出ない", no_warning is True)

    # 閾値超えの画像を持つ EPUB を生成して開く。
    big_ch2 = CH2.replace(
        '<img id="tall" src="images/tall.png" alt="tall"/>',
        '<img id="huge" src="images/huge.png" alt="huge"/>',
    )
    big_epub = SAMPLE.parent / "huge.epub"
    with zipfile.ZipFile(big_epub, "w") as z:
        z.writestr(zipfile.ZipInfo("mimetype"), "application/epub+zip",
                   compress_type=zipfile.ZIP_STORED)
        z.writestr("META-INF/container.xml", CONTAINER, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/content.opf", OPF, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/nav.xhtml", NAV, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch1.xhtml", CH1, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch2.xhtml", big_ch2, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/pic.png", PNG, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/huge.png", tall_png(10, 5000), compress_type=zipfile.ZIP_DEFLATED)

    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id_big = document["result"]["root"]["nodeId"]
    node = await cdp.send(
        "DOM.querySelector", {"nodeId": root_id_big, "selector": "#epub-file"}
    )
    if node["result"]["nodeId"] != 0:
        await cdp.send(
            "DOM.setFileInputFiles",
            {"nodeId": node["result"]["nodeId"], "files": [str(big_epub)]},
        )
        await cdp.evaluate(
            "document.querySelector('#epub-file').dispatchEvent("
            "new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp,
            "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
            timeout=10,
        )
        await select_chapter(cdp, 1)
        await wait_until(
            cdp, "document.querySelector('#huge')?.naturalHeight === 5000", timeout=10
        )
        warned = await wait_until(
            cdp,
            "document.querySelector('.reader-status')?.textContent.includes('Large image detected')",
            timeout=5,
        )
        warning = await cdp.evaluate(
            "document.querySelector('.reader-status')?.textContent || ''"
        )
        checker.check(
            "5000px の画像で警告が出る",
            warned is True,
            f"(status={warning!r})",
        )

    # 既存サンプル（remote-resources 宣言なし）では警告が出ない。
    await cdp.evaluate("localStorage.clear()")
    await open_sample(cdp)
    await wait_until(
        cdp,
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await asyncio.sleep(1.0)
    no_remote_warning = await cdp.evaluate(
        "(() => { const s = document.querySelector('.reader-status')?.textContent || '';"
        " return !s.includes('remote resources'); })()"
    )
    checker.check("宣言なしでは remote-resources 警告が出ない", no_remote_warning is True)

    # remote-resources を宣言する EPUB を生成して開く。
    remote_opf = OPF.replace('properties="nav"', 'properties="nav remote-resources"')
    remote_epub = SAMPLE.parent / "remote.epub"
    with zipfile.ZipFile(remote_epub, "w") as z:
        z.writestr(zipfile.ZipInfo("mimetype"), "application/epub+zip",
                   compress_type=zipfile.ZIP_STORED)
        z.writestr("META-INF/container.xml", CONTAINER, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/content.opf", remote_opf, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/nav.xhtml", NAV, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch1.xhtml", CH1, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch2.xhtml", CH2, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/pic.png", PNG, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/tall.png", tall_png(), compress_type=zipfile.ZIP_DEFLATED)

    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id_remote = document["result"]["root"]["nodeId"]
    node = await cdp.send(
        "DOM.querySelector", {"nodeId": root_id_remote, "selector": "#epub-file"}
    )
    if node["result"]["nodeId"] != 0:
        await cdp.send(
            "DOM.setFileInputFiles",
            {"nodeId": node["result"]["nodeId"], "files": [str(remote_epub)]},
        )
        await cdp.evaluate(
            "document.querySelector('#epub-file').dispatchEvent("
            "new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp,
            "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
            timeout=10,
        )
        remote_warned = await wait_until(
            cdp,
            "document.querySelector('.reader-status')?.textContent.includes('remote resources')",
            timeout=5,
        )
        remote_status = await cdp.evaluate(
            "document.querySelector('.reader-status')?.textContent || ''"
        )
        checker.check(
            "remote-resources 宣言で警告が出る",
            remote_warned is True,
            f"(status={remote_status!r})",
        )

    # オプトインの画像縮小。幅が本文より大きい 4500x100 の画像を使う。
    checker.check(
        "画像縮小トグルが存在する",
        await cdp.evaluate("document.querySelector('#reader-downscale') !== null"),
    )
    wide_ch2 = CH2.replace(
        '<img id="tall" src="images/tall.png" alt="tall"/>',
        '<img id="wide" src="images/wide.png" alt="wide"/>',
    )
    wide_epub = SAMPLE.parent / "wide.epub"
    with zipfile.ZipFile(wide_epub, "w") as z:
        z.writestr(zipfile.ZipInfo("mimetype"), "application/epub+zip",
                   compress_type=zipfile.ZIP_STORED)
        z.writestr("META-INF/container.xml", CONTAINER, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/content.opf", OPF, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/nav.xhtml", NAV, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch1.xhtml", CH1, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/ch2.xhtml", wide_ch2, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/pic.png", PNG, compress_type=zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/images/wide.png", tall_png(4500, 100), compress_type=zipfile.ZIP_DEFLATED)

    document = await cdp.send("DOM.getDocument", {"depth": -1})
    root_id_wide = document["result"]["root"]["nodeId"]

    async def open_wide() -> None:
        doc = await cdp.send("DOM.getDocument", {"depth": -1})
        fresh_root = doc["result"]["root"]["nodeId"]
        node = await cdp.send(
            "DOM.querySelector", {"nodeId": fresh_root, "selector": "#epub-file"}
        )
        if node["result"]["nodeId"] == 0:
            return
        await cdp.send(
            "DOM.setFileInputFiles",
            {"nodeId": node["result"]["nodeId"], "files": [str(wide_epub)]},
        )
        await cdp.evaluate(
            "document.querySelector('#epub-file').dispatchEvent("
            "new Event('change', {bubbles: true}))"
        )
        await wait_until(
            cdp,
            "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
            timeout=10,
        )
        await select_chapter(cdp, 1)
        await wait_until(
            cdp, "document.querySelector('#wide')?.naturalWidth > 0", timeout=10
        )
        await asyncio.sleep(1.0)

    # 既定（オフ）では縮小されない。
    await cdp.evaluate("localStorage.clear()")
    await reload_reader(cdp)
    await open_wide()
    original_width = await cdp.evaluate("document.querySelector('#wide')?.naturalWidth ?? 0")
    checker.check(
        "既定では巨大画像が縮小されない",
        original_width == 4500,
        f"(naturalWidth={original_width})",
    )

    # トグルを有効にして再度開く。
    await cdp.evaluate("document.querySelector('#reader-downscale').click()")
    await asyncio.sleep(0.3)
    pressed = await cdp.evaluate(
        "document.querySelector('#reader-downscale')?.getAttribute('aria-pressed')"
    )
    checker.check("画像縮小トグルが有効になる", pressed == "true")
    await open_wide()
    # 縮小は spawn_local の非同期処理（Canvas → Blob → src 差し替え）で
    # 完了時刻に揺れがある。wait_until の戻り値を確認し、タイムアウト時は
    # 診断情報を出してから再待機する（期待値・チェック数は変更しない）。
    downscaled = "document.querySelector('#wide')?.naturalWidth <= 2000"
    resized = await wait_until(cdp, downscaled, timeout=10)
    if not resized:
        diag = await cdp.evaluate(
            "(() => {"
            " const img = document.querySelector('#wide');"
            " return {"
            " naturalWidth: img?.naturalWidth ?? 0,"
            " src: (img?.src || '').slice(0, 64),"
            " status: document.querySelector('.reader-status')?.textContent || '',"
            " };"
            " })()"
        )
        print(f"warn 画像縮小の完了待ちが 10s でタイムアウト: {diag}")
        resized = await wait_until(cdp, downscaled, timeout=10)
    resized_width = await cdp.evaluate("document.querySelector('#wide')?.naturalWidth ?? 0")
    checker.check(
        "有効時は巨大画像が 2000px 以下に縮小される",
        0 < resized_width <= 2000,
        f"(naturalWidth={resized_width})",
    )
    resized_mime = await cdp.evaluate(
        "fetch(document.querySelector('#wide').src)"
        ".then((r) => r.blob()).then((b) => b.type)"
    )
    checker.check(
        "縮小後も元画像の MIME が保持される",
        resized_mime == "image/png",
        f"(mime={resized_mime!r})",
    )

    # 表示設定の一括リセット。
    checker.check(
        "リセットボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-reset-settings') !== null"),
    )
    # 複数の設定を変更する。
    await cdp.evaluate("document.querySelector('#reader-font-larger').click()")
    await cdp.evaluate("document.querySelector('#reader-line-spacing').click()")
    await cdp.evaluate("document.querySelector('#reader-theme').click()")
    await cdp.evaluate(
        "(() => { const input = document.querySelector('#reader-swipe-threshold');"
        " input.value = '100'; input.dispatchEvent(new Event('change', {bubbles: true})); })()"
    )
    await asyncio.sleep(0.5)
    modified = await cdp.evaluate(
        "JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}')"
    )
    checker.check(
        "リセット前に設定が変更されている",
        modified.get("swipeThreshold") == 100 and modified.get("dark") is True,
        f"(settings={modified!r})",
    )
    await cdp.evaluate("document.querySelector('#reader-reset-settings').click()")
    await asyncio.sleep(0.5)
    reset = await cdp.evaluate(
        "JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}')"
    )
    checker.check(
        "リセットで全表示設定が既定値になる",
        reset.get("fontScale") == 1
        and reset.get("lineHeight") == 1
        and reset.get("dark") is False
        and reset.get("followSystem") is False
        and reset.get("swipeThreshold") == 50
        and reset.get("verticalScrollThreshold") == 30
        and reset.get("downscaleImages") is False,
        f"(settings={reset!r})",
    )
    reset_status = await cdp.evaluate(
        "document.querySelector('.reader-status')?.textContent || ''"
    )
    checker.check(
        "リセットの状態メッセージが出る",
        "settings reset" in str(reset_status).lower(),
        f"(status={reset_status!r})",
    )

    # JPEG 品質設定。
    checker.check(
        "JPEG 品質入力が存在する",
        await cdp.evaluate("document.querySelector('#reader-jpeg-quality') !== null"),
    )
    await cdp.evaluate(
        "(() => { const input = document.querySelector('#reader-jpeg-quality');"
        " input.value = '70'; input.dispatchEvent(new Event('change', {bubbles: true})); })()"
    )
    await asyncio.sleep(0.5)
    jpeg_saved = await cdp.evaluate(
        "JSON.parse(localStorage.getItem('openrustbooks:settings') || '{}').jpegQuality"
    )
    checker.check("JPEG 品質が設定に保存される", jpeg_saved == 70, f"(jpegQuality={jpeg_saved!r})")
    await cdp.evaluate("localStorage.clear()")
    await cdp.evaluate(
        "localStorage.setItem('openrustbooks:settings', JSON.stringify({jpegQuality: 999}))"
    )
    await reload_reader(cdp)
    await asyncio.sleep(0.5)
    jpeg_clamped = await cdp.evaluate("document.querySelector('#reader-jpeg-quality')?.value")
    checker.check(
        "範囲外の JPEG 品質は 100 にクランプされる",
        jpeg_clamped == "100",
        f"(value={jpeg_clamped!r})",
    )

    # キーボードショートカットのヘルプ。
    checker.check(
        "ヘルプボタンが存在する",
        await cdp.evaluate("document.querySelector('#reader-help') !== null"),
    )
    # 入力欄にフォーカスがあるときは ? を奪わない。
    await cdp.evaluate("document.body.focus()")
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyDown", "key": "?", "code": "Slash", "text": "?",
         "windowsVirtualKeyCode": 191, "modifiers": 8},
    )
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyUp", "key": "?", "code": "Slash",
         "windowsVirtualKeyCode": 191, "modifiers": 8},
    )
    await asyncio.sleep(0.3)
    help_open = await cdp.evaluate(
        "document.querySelector('#reader-help-dialog')?.open === true"
    )
    checker.check("? キーでヘルプが開く", help_open is True)
    help_text = await cdp.evaluate(
        "document.querySelector('#reader-help-dialog')?.textContent || ''"
    )
    checker.check(
        "ヘルプに主要なショートカットが含まれる",
        "Previous" in str(help_text) and "dark mode" in str(help_text),
        f"(text={str(help_text)[:120]!r})",
    )
    await cdp.evaluate("document.querySelector('#reader-help-dialog button').click()")
    await asyncio.sleep(0.3)
    help_closed = await cdp.evaluate(
        "document.querySelector('#reader-help-dialog')?.open === false"
    )
    checker.check("ヘルプを閉じられる", help_closed is True)

    # 章タイトルの検索フィルター。
    await cdp.evaluate("localStorage.clear()")
    await reload_reader(cdp)
    await open_sample(cdp)
    await wait_until(
        cdp,
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    await asyncio.sleep(0.5)
    checker.check(
        "章フィルター入力が存在する",
        await cdp.evaluate("document.querySelector('#reader-chapter-filter') !== null"),
    )
    all_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-chapter-select option').length"
    )
    await cdp.evaluate(
        "(() => { const input = document.querySelector('#reader-chapter-filter');"
        " input.value = 'Two'; input.dispatchEvent(new Event('input', {bubbles: true})); })()"
    )
    await asyncio.sleep(0.4)
    filtered_labels = await cdp.evaluate(
        "Array.from(document.querySelectorAll('#reader-chapter-select option'))"
        ".map((o) => o.textContent)"
    )
    checker.check(
        "フィルターで該当章が残る",
        any("Two" in str(label) for label in filtered_labels),
        f"(labels={filtered_labels!r}, all={all_count})",
    )
    # 一致しない語では、選択状態を保つため現在章だけが残る。
    await cdp.evaluate(
        "(() => { const input = document.querySelector('#reader-chapter-filter');"
        " input.value = 'ZZZNOTFOUND'; input.dispatchEvent(new Event('input', {bubbles: true})); })()"
    )
    await asyncio.sleep(0.4)
    no_match_labels = await cdp.evaluate(
        "Array.from(document.querySelectorAll('#reader-chapter-select option'))"
        ".map((o) => o.textContent)"
    )
    checker.check(
        "一致しないフィルターでは現在章だけが残る",
        len(no_match_labels) == 1,
        f"(labels={no_match_labels!r})",
    )
    await cdp.evaluate(
        "(() => { const input = document.querySelector('#reader-chapter-filter');"
        " input.value = ''; input.dispatchEvent(new Event('input', {bubbles: true})); })()"
    )
    await asyncio.sleep(0.4)
    restored_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-chapter-select option').length"
    )
    checker.check(
        "フィルターを消すと全章に戻る",
        restored_count == all_count,
        f"(count={restored_count}, expected={all_count})",
    )

    # / キーでフィルター入力にフォーカスする。
    await cdp.evaluate("document.body.focus()")
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyDown", "key": "/", "code": "Slash", "text": "/",
         "windowsVirtualKeyCode": 191},
    )
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyUp", "key": "/", "code": "Slash",
         "windowsVirtualKeyCode": 191},
    )
    await asyncio.sleep(0.3)
    focused_filter = await cdp.evaluate(
        "document.activeElement?.id === 'reader-chapter-filter'"
    )
    checker.check("/ キーで章フィルターにフォーカスする", focused_filter is True)
    # ヘルプに / の記載が増えている。
    await cdp.evaluate("document.querySelector('#reader-help').click()")
    await asyncio.sleep(0.3)
    help_with_slash = await cdp.evaluate(
        "document.querySelector('#reader-help-dialog')?.textContent.includes('chapter filter')"
    )
    checker.check("ヘルプに / の記載がある", help_with_slash is True)
    await cdp.evaluate("document.querySelector('#reader-help-dialog button').click()")

    # 外部リンクのクリックで window.open が noopener/noreferrer 付きで呼ばれる。
    # 実際にタブを開かず、呼び出し引数だけを記録する。
    await cdp.evaluate(
        "(() => {"
        " window.__openCalls = [];"
        " window.open = (url, target, features) => {"
        "  window.__openCalls.push({url, target, features}); return null;"
        " };"
        " })()"
    )
    await cdp.evaluate("document.querySelector('#external-link').click()")
    await asyncio.sleep(0.3)
    open_calls = await cdp.evaluate("window.__openCalls")
    checker.check("外部リンクのクリックで window.open が呼ばれる", len(open_calls or []) == 1,
                  f"(calls={open_calls!r})")
    if open_calls:
        call = open_calls[0]
        checker.check(
            "外部リンクが _blank / noopener,noreferrer で開かれる",
            call.get("url") == "https://example.com"
            and call.get("target") == "_blank"
            and "noopener" in str(call.get("features"))
            and "noreferrer" in str(call.get("features")),
            f"(call={call!r})",
        )
    # mailto: リンクも同じ分岐で noopener/noreferrer 付きで開かれる。
    await cdp.evaluate("window.__openCalls = []")
    mail_href = await cdp.evaluate(
        "document.querySelector('#mailto-link')?.getAttribute('href')"
    )
    checker.check("mailto リンクの href が保持される", mail_href == "mailto:test@example.com",
                  f"(href={mail_href!r})")
    await cdp.evaluate("document.querySelector('#mailto-link').click()")
    await asyncio.sleep(0.3)
    mail_calls = await cdp.evaluate("window.__openCalls")
    checker.check("mailto リンクのクリックで window.open が呼ばれる", len(mail_calls or []) == 1,
                  f"(calls={mail_calls!r})")
    if mail_calls:
        call = mail_calls[0]
        checker.check(
            "mailto が _blank / noopener,noreferrer で開かれる",
            call.get("url") == "mailto:test@example.com"
            and call.get("target") == "_blank"
            and "noopener" in str(call.get("features"))
            and "noreferrer" in str(call.get("features")),
            f"(call={call!r})",
        )

    # スタブを元に戻す（後続のテストに影響しないように）。
    await reload_reader(cdp)

    # Content-Security-Policy が設定されている。
    csp = await cdp.evaluate(
        "document.querySelector('meta[http-equiv=\"Content-Security-Policy\"]')"
        "?.getAttribute('content') || ''"
    )
    checker.check(
        "Content-Security-Policy が設定される",
        "default-src 'self'" in str(csp) and "wasm-unsafe-eval" in str(csp),
        f"(csp={csp!r})",
    )

    # IME 変換中（isComposing=true）はショートカットが効かない。
    await cdp.evaluate("document.body.focus()")
    dark_before_ime = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    await cdp.evaluate(
        "document.dispatchEvent(new KeyboardEvent('keydown', {"
        " key: 'd', bubbles: true, cancelable: true, isComposing: true}))"
    )
    await asyncio.sleep(0.3)
    dark_after_ime = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    checker.check(
        "IME 変換中はショートカットが発火しない",
        dark_before_ime == dark_after_ime,
        f"(before={dark_before_ime!r}, after={dark_after_ime!r})",
    )

    # モーダル表示中は背景のショートカットが止まる。
    await cdp.evaluate("document.querySelector('#reader-help').click()")
    await asyncio.sleep(0.3)
    dark_before_modal = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyDown", "key": "d", "code": "KeyD", "windowsVirtualKeyCode": 68},
    )
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyUp", "key": "d", "code": "KeyD", "windowsVirtualKeyCode": 68},
    )
    await asyncio.sleep(0.3)
    dark_after_modal = await cdp.evaluate(
        "document.querySelector('.reader-container')?.classList.contains('reader-dark')"
    )
    checker.check(
        "モーダル中はダーク切替ショートカットが発火しない",
        dark_before_modal == dark_after_modal,
        f"(before={dark_before_modal!r}, after={dark_after_modal!r})",
    )
    await cdp.evaluate("document.querySelector('#reader-help-dialog button').click()")

    # 保存データ 0 件のエクスポート経路。
    # 一時ダウンロード先を指定し、ファイルが生成されないことを確認する。
    empty_download_dir = SAMPLE.parent / "empty-downloads"
    empty_download_dir.mkdir(parents=True, exist_ok=True)
    await cdp.send(
        "Browser.setDownloadBehavior",
        {"behavior": "allow", "downloadPath": str(empty_download_dir), "eventsEnabled": True},
    )
    await cdp.evaluate("localStorage.clear()")
    await asyncio.sleep(0.3)
    # スクロール保存のデバウンスで位置が再登録される可能性があるため直前に再クリアする。
    await cdp.evaluate("localStorage.clear()")
    await cdp.evaluate("document.querySelector('#reader-export').click()")
    no_data_status = await wait_until(
        cdp,
        "document.querySelector('.reader-status')?.textContent"
        "?.includes('No saved reading positions to export.') === true",
        timeout=5,
    )
    checker.check("保存データ 0 件のエクスポートで状態メッセージが出る", no_data_status is True)
    # ダウンロードが始まる場合に備えて待ってからディレクトリを確認する。
    await asyncio.sleep(1.0)
    empty_files = [path.name for path in empty_download_dir.iterdir()]
    checker.check(
        "保存データ 0 件ではエクスポートファイルが生成されない",
        not empty_files,
        f"(files={empty_files!r})",
    )
    shutil.rmtree(empty_download_dir, ignore_errors=True)

    # 文字縮小: A+ で大きくした後に A- で元のサイズへ戻ることと、
    # 最小値で A- が disabled になることを確認する。
    # 後続テストへ影響しないよう、クリアしてからサンプルを開き直す。
    await cdp.evaluate("localStorage.clear()")
    await reload_reader(cdp)
    await open_sample(cdp)
    await wait_until(
        cdp,
        "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')",
        timeout=10,
    )
    base_font = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    await cdp.evaluate("document.querySelector('#reader-font-larger').click()")
    await wait_until(
        cdp,
        "getComputedStyle(document.querySelector('.reader-html')).fontSize === '20px'",
        timeout=5,
    )
    await cdp.evaluate("document.querySelector('#reader-font-smaller').click()")
    font_restored = await wait_until(
        cdp,
        "(() => { const el = document.querySelector('.reader-html');"
        f" return !!el && getComputedStyle(el).fontSize === {json.dumps(base_font)}; }})()",
        timeout=5,
    )
    checker.check("A- で文字サイズが元に戻る", font_restored, f"(base={base_font!r})")
    await cdp.evaluate("document.querySelector('#reader-font-smaller').click()")
    smaller_disabled = await wait_until(
        cdp,
        "document.querySelector('#reader-font-smaller')?.disabled === true",
        timeout=5,
    )
    checker.check("最小値まで下げると A- が disabled になる", smaller_disabled is True)

    # - キーでも文字が小さくなる（CDP の Input.dispatchKeyEvent を使用）。
    await cdp.evaluate("document.querySelector('#reader-font-larger').click()")
    await wait_until(
        cdp,
        "getComputedStyle(document.querySelector('.reader-html')).fontSize === '18px'",
        timeout=5,
    )
    before_minus = await cdp.evaluate(
        "getComputedStyle(document.querySelector('.reader-html')).fontSize"
    )
    await cdp.evaluate("document.body.focus()")
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyDown", "key": "-", "code": "Minus", "text": "-",
         "windowsVirtualKeyCode": 189},
    )
    await cdp.send(
        "Input.dispatchKeyEvent",
        {"type": "keyUp", "key": "-", "code": "Minus", "windowsVirtualKeyCode": 189},
    )
    minus_shrunk = await wait_until(
        cdp,
        "getComputedStyle(document.querySelector('.reader-html')).fontSize === '16px'",
        timeout=5,
    )
    checker.check("- キーで文字サイズが小さくなる", minus_shrunk, f"(before={before_minus!r})")

    # 設定画面: ブラウザ単体では window.__TAURI__ が無いため invoke せず、
    # 例外ではなく「デスクトップ版のみ利用可能」が表示されることを確認する。
    # エラーはページ読込前に注入したリスナーで収集する（SPA 遷移でも維持される）。
    await cdp.send(
        "Page.addScriptToEvaluateOnNewDocument",
        {
            "source": "window.__orbSettingsErrors = [];"
            "window.addEventListener('error', (e) => window.__orbSettingsErrors.push(String(e.message)));"
            "window.addEventListener('unhandledrejection', (e) => window.__orbSettingsErrors.push(String(e.reason)));"
        },
    )
    await cdp.send("Page.navigate", {"url": f"http://127.0.0.1:{PORT}/settings"})
    settings_mounted = await wait_until(
        cdp, "document.querySelector('.settings-form') !== null"
    )
    offline_notice = await wait_until(
        cdp,
        "document.querySelector('.settings-form')?.textContent.includes('デスクトップ版のみ利用可能')",
        timeout=5,
    )
    settings_errors = await cdp.evaluate(
        "Array.isArray(window.__orbSettingsErrors) ? window.__orbSettingsErrors : ['listener missing']"
    )
    checker.check(
        "ブラウザ版の設定画面は例外なくデスクトップ版のみと表示する",
        settings_mounted and offline_notice and settings_errors == [],
        f"(mounted={settings_mounted}, notice={offline_notice}, errors={settings_errors!r})",
    )

    return finish(checker)


def finish(checker: Checker) -> int:
    print()
    executed = checker.passed + len(checker.failures)
    print(f"passed: {checker.passed}, failed: {len(checker.failures)}")
    if checker.failures:
        print("failures:", checker.failures)
    if executed != checker.expected:
        print(f"expected {checker.expected} checks, but executed {executed}")
        return 1
    if checker.failures:
        return 1
    print("all checks passed")
    return 0


async def main() -> int:
    import websockets

    global DIST, SAMPLE, PORT, DEBUG_PORT
    parser = argparse.ArgumentParser(description="Web 版 EPUB リーダーの E2E 検証")
    parser.add_argument(
        "--dist",
        type=Path,
        default=DEFAULT_DIST,
        help="trunk build の出力ディレクトリ（既定: web/dist）",
    )
    parser.add_argument("--chrome", default="google-chrome", help="Chrome/Chromium の実行ファイル")
    parser.add_argument("--port", type=int, default=PORT, help="静的配信のポート")
    parser.add_argument(
        "--debug-port", type=int, default=DEBUG_PORT, help="Chrome DevTools のポート"
    )
    args = parser.parse_args()
    DIST = args.dist.resolve()
    PORT = args.port
    DEBUG_PORT = args.debug_port
    if not (DIST / "index.html").is_file():
        print(f"index.html が見つかりません: {DIST}", file=sys.stderr)
        return 2
    build_sample()
    server = ThreadingHTTPServer(("127.0.0.1", PORT), SpaHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    # 差分はヒントとしてだけ記録する（決定的な特定は起動 Chrome 自身の browser WS で行う）。
    known_ids = snapshot_target_ids()
    if known_ids:
        print(
            f"既存 Chrome の target を {len(known_ids)} 件検出（ポート {DEBUG_PORT}）。"
            "起動 Chrome 自身の browser WS を使って接続先を特定します。",
            flush=True,
        )

    profile = tempfile.mkdtemp(prefix="orb-chrome-")
    chrome = subprocess.Popen(
        [
            args.chrome,
            "--headless=new",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            f"--remote-debugging-port={DEBUG_PORT}",
            f"--user-data-dir={profile}",
            "--window-size=1280,900",
            "about:blank",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    browser_ws_queue: queue.Queue[str] = queue.Queue()
    stderr_thread = threading.Thread(
        target=drain_chrome_stderr, args=(chrome.stderr, browser_ws_queue), daemon=True
    )
    stderr_thread.start()
    try:
        browser_ws_url = wait_for_browser_ws(browser_ws_queue)
        target = await select_page_target(browser_ws_url, known_ids)
        print(f"接続先 page target: {target['id']} ({target['url']})", flush=True)
        async with websockets.connect(target["webSocketDebuggerUrl"], max_size=None) as ws:
            cdp = Cdp(ws)
            return await run_checks(cdp)
    except RuntimeError as exc:
        # run_checks 側で診断とサマリは出力済み。traceback を出さず rc=1 で終える。
        print(f"E2E 失敗 (RuntimeError): {exc}", file=sys.stderr, flush=True)
        return 1
    finally:
        chrome.send_signal(signal.SIGTERM)
        try:
            chrome.wait(timeout=10)
        except subprocess.TimeoutExpired:
            chrome.kill()
        server.shutdown()
        shutil.rmtree(profile, ignore_errors=True)
        shutil.rmtree(SAMPLE.parent, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
