---
type: Decision
title: "ADR-0161: 最小限の textarea エディタ UI"
description: "CodeMirror を導入せず、textarea と既存 Preview を接続した最小エディタを実装する決定を記録する。"
tags:
  - ADR
  - web
  - ui
generated:
  by: dsh
  at: "2026-11-18T22:10:00+09:00"
status: accepted
---

## 背景

ADR-0127 は CodeMirror を導入せず、textarea ベースの最小構成で
テキスト編集・保存・Undo を実現する方針を決定した。
しかし `web/src/lib.rs` の Editor は空 div とスタブボタンのままで、
Preview へ入力が届かなかった。

## 決定

- `editor-pane` を `<textarea id="editor-input" class="editor-pane">` に置換。
- `on:input` で `HtmlTextAreaElement.value()` を取得し `set_content` へ反映。
  - `prop:value` は意図的に使用しない。同じ文字列を再代入すると
    キャレットが末尾へ戻るため、textarea 自身を入力値の唯一の保持元とする。
  - `on:change` ではなく `on:input` でキー入力ごとに反映。
- 既存の `<Preview content set_content/>` を再利用し、
  `sanitize_epub_html(&content.get())` をリアルタイム表示する。
  - Preview は Markdown レンダラではなく、EPUB 用サニタイザを再利用する。
- Save / AI Generate / AI Refine / Lint / format select は恒久 disabled とし、
  title / aria-label に "desktop only" / "not implemented" を明示。
  Save 文言は "Save (desktop only)"。
- `web/index.html` の `#editor` を `#editor-input` へ統一し、
  textarea 用 CSS（box-sizing / width / height / border / resize / overflow / フォント）を設定。
  2 カラムレイアウトは維持。
- Tauri invoke は有効化しない（ADR-0128 の判断を維持）。
- E2E を 2 件追加（入力→Preview 反映、ボタン disabled と理由付き）。
  期待総数 235 → **237**（check 220 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| textarea ＋ `on:input`（採用） | 新規依存なしで即座に入力・Undo が使える |
| CodeMirror | ADR-0127 で見送り済み |
| `prop:value` で同期 | キャレット位置が壊れるため却下 |
| ボタンを有効化 | Tauri 未接続で誤動作するため却下 |

## 影響

- ショートカットガード（textarea 除外）は既存のまま維持。
- 巨大入力のデバウンス／入力長上限は未実装（**推測**の性能懸念、
  未計測のため機能バグ扱いしない）。
- IME 変換／Undo 履歴への `on:input` の影響は未検証
  （ADR-0127 / 0128 の未決定を継続）。
- 新規依存なし。

## 検証（Runtime Proof）

- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm32 web check、
  `./scripts/verify.sh`: すべて成功。
- E2E **237 項目すべて成功**。
  - 「エディタの textarea 入力が Preview に反映される」
  - 「エディタのデスクトップ専用操作が disabled で理由付き」
- 全単体テスト core 196 + desktop 56 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ~~Markdown の本格レンダリング~~ ADR-0162 で comrak + 二段サニタイズを実装。
- ~~保存先と保存接続~~ ADR-0164 で localStorage ドラフト保存を実装（Tauri は未接続）。
- disabled 理由の可視提示（`title` 依存で十分か、要実機確認）。
- `withGlobalTauri` / CSP / capability（ADR-0128 から継続）。
- 巨大入力のデバウンス、IME / Undo の実機検証。
