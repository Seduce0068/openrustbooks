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
章送りやテキスト描画をこの枠組みへ載せるのが最も確実な経路である。
**本リポジトリでのコンパイル検証は未実施**（公式ドキュメントに基づく提案）。

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
| Wayland ネイティブのキー・クリック | `ydotool` / `wtype` | 未インストール。`/dev/uinput` は root 所有で非対話利用不可 |

`kdotool` v0.3.0 には `key` / `click` コマンドが無く、ウィンドウ制御と
KWin JavaScript 実行（`kwinscript`）のみ。Wayland ネイティブへの物理入力が必要なら、
`ydotool` の導入と `/dev/uinput` 権限（対話 `sudo` または udev ルール）の承認が必要。

### 実測（未解決）

GPUI ビューアを起動すると `TRACE: open_window ok=true` と `TRACE: activated` が出力される一方、
`kdotool search` / `wmctrl -l` / `xwininfo -root -tree` のいずれにもウィンドウが現れず、
`spectacle` の全画面 OCR にも書誌タイトルが検出されなかった。
つまり「アプリは起動しているが、エージェントから画面を確認できない」状態で、
原因（Wayland ウィンドウのマッピング／コンポジタ側の登録）は未確認。
このため OS レベル方式は現状の GPUI 版には使えず、手法 2 が重要になる。

## 推奨フロー

- 機能テストは GPUI の `#[gpui::test]` と `render_to_image` で再現性を確保する。
- Web 画面の操作確認は Playwright MCP のスナップショット→クリック→撮影で行う。
- ネイティブ Window の実機確認が必要な場合のみ `kdotool` + `spectacle` + OCR を使い、
  取得できない場合は理由を添えてユーザーに目視確認を依頼する（共通指示の「実行環境での自己確認」に従う）。

## 未確認事项

- GPUI の `test-support` feature を本リポジトリで有効化した際のコンパイル成否。
- Wayland 上で GPUI ウィンドウが KWin に登録されない原因。
- Wayland ネイティブへのキー／クリック注入手段（`ydotool` 導入と権限）。
