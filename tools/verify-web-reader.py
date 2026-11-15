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
<img id="tall" src="images/tall.png" alt="tall"/>
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
    checker.check(
        "保存された履歴が無いときは Back が disabled",
        await cdp.evaluate("document.querySelector('#reader-back')?.disabled === true"),
    )
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
    attempts = [entry for entry in attempts if entry.get("top", 0) > 0]
    checker.check(
        "位置の適用は画像の読み込み完了を待つ",
        bool(attempts) and all(entry.get("complete") is True for entry in attempts),
        f"(attempts={attempts!r})",
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
    await asyncio.sleep(0.5)
    after_count = await cdp.evaluate(
        "document.querySelectorAll('#reader-history option').length"
    )
    checker.check("クリア後に履歴が 1 件になる", after_count == 1, f"(count={after_count})")
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
