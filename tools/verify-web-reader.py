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

CH2 = f"""<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Chapter Two</title></head>
<body>
<h1 id="start">Chapter Two</h1>
<p>Second chapter text.</p>
{FILLER}
<h2 id="part">Part Two</h2>
<p>Part two text.</p>
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


class Cdp:
    def __init__(self, ws):
        self.ws = ws
        self.counter = 0

    async def send(self, method: str, params: dict | None = None) -> dict:
        self.counter += 1
        request_id = self.counter
        await self.ws.send(json.dumps({"id": request_id, "method": method, "params": params or {}}))
        while True:
            message = json.loads(await self.ws.recv())
            if message.get("id") == request_id:
                return message

    async def evaluate(self, expression: str):
        result = await self.send(
            "Runtime.evaluate",
            {"expression": expression, "returnByValue": True, "awaitPromise": True},
        )
        return result.get("result", {}).get("result", {}).get("value")


class Checker:
    def __init__(self):
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
    """ページを再読み込みし、Reader のマウントを待つ。"""
    await cdp.send("Page.reload", {"ignoreCache": True})
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


async def wait_saved(cdp: Cdp, predicate, timeout: float = 5.0):
    """保存された読書位置が条件を満たすまで待つ。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        saved = await saved_position(cdp)
        if saved is not None and predicate(saved):
            return saved
        await asyncio.sleep(0.1)
    return None


async def run_checks(cdp: Cdp) -> int:
    checker = Checker()

    await cdp.send("Page.enable")
    await cdp.send("Runtime.enable")
    await cdp.send("DOM.enable")
    await cdp.send("Page.navigate", {"url": f"http://127.0.0.1:{PORT}/reader/e2e"})

    mounted = await wait_until(cdp, "document.querySelector('.reader-container') !== null")
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
    if not opened:
        print("status:", await cdp.evaluate("document.querySelector('.reader-status')?.textContent"))
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
        await cdp.evaluate("document.querySelectorAll('.reader-chapter-select option').length"),
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
        await cdp.evaluate("document.querySelector('.reader-chapter-select')?.value"),
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
        "保存する値は章・フラグメント・スクロール位置だけ",
        saved is not None
        and sorted(saved.keys()) == ["chapterIndex", "fragment", "scrollTop"],
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
        await cdp.evaluate("document.querySelector('.reader-chapter-select')?.value"),
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
    await cdp.evaluate(
        "(() => {"
        " const key = Object.keys(localStorage).find((k) => k.startsWith('openrustbooks:position:'));"
        " localStorage.setItem(key, JSON.stringify({chapterIndex: 999, fragment: null}));"
        " })()"
    )
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    fallback = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Hello from chapter one.')"
    )
    checker.check("範囲外の保存値では先頭の章になる", bool(fallback))

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
    checker.check(
        "章の切り替えでスクロール位置が先頭になる",
        (await cdp.evaluate("document.querySelector('.reader-content')?.scrollTop ?? -1")) == 0,
    )

    # scrollTop の無い旧形式の保存値でも章は復元される。
    await cdp.evaluate(
        "(() => {"
        " const key = Object.keys(localStorage).find((k) => k.startsWith('openrustbooks:position:'));"
        " localStorage.setItem(key, JSON.stringify({chapterIndex: 1, fragment: null}));"
        " })()"
    )
    await reload_reader(cdp)
    reopened = await open_sample(cdp)
    legacy = reopened and await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    checker.check("旧形式の保存値でも章が復元される", bool(legacy))

    # 移動履歴（戻る・進む）。
    checker.check(
        "開いた直後は Back が disabled",
        await cdp.evaluate("document.querySelector('#reader-back')?.disabled === true"),
    )
    checker.check(
        "開いた直後は Forward が disabled",
        await cdp.evaluate("document.querySelector('#reader-forward')?.disabled === true"),
    )

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
        await cdp.evaluate("document.querySelector('.reader-chapter-select')?.value"),
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

    # 本を開き直すと履歴がリセットされる。
    reopened = await open_sample(cdp)
    await wait_until(
        cdp, "document.querySelector('.reader-html')?.textContent.includes('Second chapter text.')"
    )
    await asyncio.sleep(0.2)
    checker.check(
        "開き直すと Back が disabled になる",
        reopened and await cdp.evaluate("document.querySelector('#reader-back')?.disabled === true"),
    )
    checker.check(
        "開き直すと Forward が disabled になる",
        await cdp.evaluate("document.querySelector('#reader-forward')?.disabled === true"),
    )

    return finish(checker)


def finish(checker: Checker) -> int:
    print()
    print(f"passed: {checker.passed}, failed: {len(checker.failures)}")
    if checker.failures:
        print("failures:", checker.failures)
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
        stderr=subprocess.DEVNULL,
    )
    try:
        wait_for_debug_port()
        with urllib.request.urlopen(f"http://127.0.0.1:{DEBUG_PORT}/json") as response:
            targets = json.load(response)
        pages = [target for target in targets if target["type"] == "page"]
        if not pages:
            raise RuntimeError("page ターゲットが見つかりませんでした")
        async with websockets.connect(pages[0]["webSocketDebuggerUrl"], max_size=None) as ws:
            cdp = Cdp(ws)
            return await run_checks(cdp)
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
