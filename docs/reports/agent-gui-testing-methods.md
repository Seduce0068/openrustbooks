---
type: Report
title: "エージェントが GUI テストを実行する方法の調査"
description: "Kubuntu/Wayland 環境で DSH エージェントが Web・GPUI・OS レベル GUI 検証を行う手段と制約の整理。"
tags:
  - investigation
  - gui
  - automation
  - playwright
  - gpui
generated:
  by: dsh (lead)
  at: "2026-11-28T01:11:40+09:00"
verified:
  - at: "2026-11-28T01:11:40+09:00"
    by: dsh (lead)
  - at: "2026-11-28T02:05:15+09:00"
    by: dsh (lead)
status: stable
---

## 結論（推奨順）

1. **Web GUI は Playwright MCP を使う**。アクセシビリティスナップショットで要素を特定し、クリック・入力・撮影をエージェントが直接実行できる。
2. **GPUI ネイティブ UI は Rust 側のテストサポートを使う**。GPUI の `test-support` feature でキー入力シミュレーションとフレーム描画画像の取得ができ、OS ウィンドウを触らずに検証できる。
3. **OS レベルの画面操作は最後の手段**。Wayland では `kdotool` でウィンドウ検索・活性化、`spectacle` で撮影、`tesseract` で OCR が可能だが、キー／クリック注入は現状の権限では不可。

## 手法 1: Web GUI（Playwright MCP）

DSH セッションには `browser_snapshot` / `browser_click` / `browser_type` / `browser_press_key` /
`browser_take_screenshot` / `browser_network_requests` / `browser_evaluate` が用意されている。
公式ドキュメントでも `browser_snapshot` は「アクションの前に優先すべき」とされ、
スナップショットの要素参照（ref）を使ってクリックする手順が示されている。

- 利的点: ブラウザを別途起動・操作せず、要素参照とアクセシビリティツリーで決定論的に操作できる。
- 制約: 撮影ファイル名は許可ルート（`~/.playwright-mcp` など）配下に置く必要がある。
  `/tmp` 指定はファイルアクセス拒否になった（実測）。
- 適用先: openrustbooks の Leptos Web UI、E2E の補完、設定画面の操作確認。

### Runtime Proof

`data:text/html` のテストページへ移動し、`browser_snapshot` で
`heading "GUI test probe"` と `button "Click me"` を取得、
`browser_click` でボタンを操作し、`~/.playwright-mcp/gui-probe.png` に撮影成功。

## 手法 2: GPUI ネイティブ UI（Rust テストサポート）

GPUI 公式ドキュメント（`gpui-pre` ソース）では、`test` または `test-support` feature のもとで
次が提供される。

- `#[gpui::test]` と `TestAppContext`: ウィンドウ生成と描画をテスト内で再現。
- `cx.dispatch_keystroke(*window, Keystroke::parse("ctrl-g").unwrap())`: キー入力をシミュレート。
- `Window::render_to_image() -> Result<image::RgbaImage>`: 現在のフレームを RGBA 画像化（画面へ表示せず検証可能）。
- `Window::painted_quads()`: ラスタライズせずに描画クワッドを検証。

本リポジトリの `gpui/` クレートでは `gpui-pre` の `test-support` を dev-dependency として有効化し、
描画色・固定ステータス・左右キーによる章送り（`arrow_keys_move_chapter`）を検証している。
`cargo test` は **6 passed / 0 failed**。`simulate_keystrokes` / `dispatch_keystroke` は
基本的に KeyDown を送る（simulated IME により text input が追加される場合がある）ため、
検証対象には KeyDown で発火する `on_key_down` ハンドラが必要。
また、キーはフォーカスした要素にのみ届くため `window.focus` が必須。

出典: <https://docs.rs/gpui-pre/latest/src/gpui/window.rs.html> /
<https://docs.rs/gpui-pre/latest/src/gpui/interactive.rs.html>

## 手法 3: OS レベル（Kubuntu / Wayland）

ローカル環境は `XDG_SESSION_TYPE=wayland`、`XDG_CURRENT_DESKTOP=KDE`。

| 目的 | 手段 | 現状 |
| --- | --- | --- |
| ウィンドウ検索・活性化・移動・サイズ | `kdotool` v0.3.0 | 利用可。`search` / `windowactivate` / `getwindowgeometry` 等 |
| 全画面／アクティブウィンドウ撮影 | `spectacle -b -o <path>` | 利用可（Wayland ポータル経由） |
| 画面文字の検出 | `tesseract -l eng+jpn` | 利用可。OCR で表示テキストを検証可能 |
| 画像比較 | Python `cv2` / `numpy` | 利用可 |
| X11/XWayland のキー・クリック | `xdotool` / `wmctrl` / `xwininfo` / `xprop` | 利用可（参考指示にも記載） |
| Wayland ネイティブのキー・クリック | `ydotool` 1.0.4 | 導入済み。`/dev/uinput` は root:input 0660、ユーザーを `input` グループへ追加、`ydotoold --socket-perm 0666` を起動。キーは `key -d 50 106:1 106:0`（右）/ `105:1 105:0`（左） |
| `wtype` / `dotool` / `wdotool` | 未導入 | KDE では `wtype` は virtual-keyboard protocol 未対応で非実用。`ydotool` で十分 |

`kdotool` v0.3.0 には `key` / `click` コマンドが無く、ウィンドウ制御と
KWin JavaScript 実行（`kwinscript`）のみ。Wayland ネイティブへの物理入力は
`ydotool` で行う。`/dev/uinput` は `sudo apt-get install ydotool` で udev ルールが
適用され root:input 0660 になり、`sudo usermod -aG input koppe` とデーモン起動で
エージェントから利用できた。なお `--socket-perm 0666` は既定 0600 の緩和であり、
任意ローカルユーザーが仮想入力へイベントを注入できる点に注意。恒久運用では 0600 へ戻す。

### 実測（解決済み）

GPUI ウィンドウが KWin に現れなかった原因は、`gpui-pre-platform` 依存に
`wayland` / `x11` feature が無く headless バックエンドが選ばれていたこと。
feature 追加後に `kdotool search --name openrustbooks` が
`openrustbooks — GPUI`（class=`openrustbooks-gpui`、geometry 1000x788）を検出し、
`spectacle` + `tesseract` が書誌タイトル・著者・章タイトル・本文を OCR した。
さらに `ydotool mousemove` で「次の章」ボタン中央へポインタを移動し、
`ydotool click 0xC0` を送ると章表示が `1 / 2` → `2 / 2`、本文が「第二章」へ変化した。
  キー入力では `kdotool windowactivate` 後に active window ID の一致を確認し、
  `ydotool key -d 50 106:1 106:0` で `2 / 2`、`105:1 105:0` で `1 / 2` への変化も OCR 確認した。
  OS レベル方式で Wayland ネイティブ GUI のクリック／キー操作を検証可能。

## 推奨フロー

- 機能テストは GPUI の `#[gpui::test]`、`painted_quads`、`simulate_keystrokes` で再現性を確保する（6 tests 成功）。
- Web 画面の操作確認は Playwright MCP のスナップショット→クリック→撮影で行う。
- ネイティブ Window の実機確認は `kdotool` → `spectacle` + OCR → `ydotool` の順で行う。
  座標は `ydotool mousemove` が相対移動のため、`kdotool getmouselocation` で
  現在位置を確認しながら目的座標へ合わせる。キーは `windowactivate` 後に
  `getactivewindow` が対象 ID と一致することを確認してから送る（`--sync` は非対応）。

## 未確認事項

- `render_to_image` を使ったスクリーンショット検証（本リポジトリでは `painted_quads` を採用）。
- `wdotool` / RemoteDesktop portal 経由の代替入力（`ydotool` で要件を満たしたため未検証）。
