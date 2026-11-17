---
type: Decision
title: "ADR-0021: キーボード操作と ARIA 属性"
description: "リーダーのキーボードショートカットとアクセシビリティ属性の決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
  - accessibility
generated:
  by: dsh
  at: "2026-11-15T12:27:00+09:00"
status: accepted
---

## 背景

`docs/requirements.md` の品質要件に「日本語IME、タッチ操作、キーボード操作を
検証する」があるが、リーダーはボタンのクリックと目次の選択だけで操作でき、
キーボードだけでは章を移動できない。また、アイコンだけのボタン（`◀` `▶`）や
本文領域にアクセシブルな名前が無く、読み上げで用途が分からない。

## 決定

### キーボード操作

`window` の `keydown` を監視する（`window_event_listener`）。リスナーは
コンポーネントの寿命に紐付け、`on_cleanup` で解除する。

- `]` → 次の章、`[` → 前の章。
- `Alt+ArrowLeft` → 戻る、`Alt+ArrowRight` → 進む。
- `+` / `=` → 文字サイズを大きく、`-` / `_` → 小さく。
- `d` → ダーク表示の切り替え。
- フォーカスが `input`・`select`・`textarea` にあるときは無視する
  （目次選択中の文字入力を奪わない）。
- `Ctrl` / `Meta` 併用時は無視する（ブラウザのショートカットを奪わない）。
- `Alt` は矢印キーの戻る・進むにだけ使う。
- 処理したキーは `prevent_default` する（`[`/`]` などでページが
  スクロールしないようにする）。

### ARIA

- アイコンだけのボタンに `aria-label` を付ける（`◀` `▶`、`A-` `A+` など）。
- 対応するボタンへ `aria-keyshortcuts` を付ける。
- 状態メッセージに `aria-live="polite"` を付ける。
- 本文領域を `role="region"`・`aria-label`・`tabindex="0"` にし、
  キーボードでフォーカスしてスクロールできるようにする。
- 章の選択に `aria-label` を付ける。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `window` の `keydown` を監視（採用） | SPA 全体でキーを受け取れ、フォーカス位置に依存しない |
| ボタンの `on:keydown` | フォーカスが当たっている要素にしか届かない |
| Leptos の `on:keydown` on container | フォーカスが無いと届かない。`window` リスナーが確実 |
| スクリーンリーダー向けのライブリージョンを増やす | まずは状態メッセージの `aria-live` で十分 |

## 影響

- キーボードだけで章移動・表示変更ができる。
- スクリーンリーダーがボタンの用途と状態変化を読み上げられる。
- `Ctrl` / `Meta` 併用はブラウザに渡すため、既存のショートカットを壊さない。
- 入力欄にフォーカスがあるときはキーを奪わない。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 86 項目が
  3 回連続で成功。キーボードについて、`]` / `[` の章移動、`+` / `-` の文字
  サイズ変更、`d` のダーク表示切り替え、`select` フォーカス中の `d` 無効、
  `Ctrl` 併用の無効、`Alt+ArrowLeft` / `Alt+ArrowRight` の履歴移動を確認
  した。ARIA について、`aria-label`・`aria-keyshortcuts`・`aria-live`・
  `role`・`aria-label`・`tabindex` を確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## 解決済み

- タッチ操作・ジェスチャーの対応: ADR-0025 でスワイプによる章移動を実装した。
- スクリーンリーダーでの本文読み上げの最適化: ADR-0057 で本文要素への
  言語設定、ADR-0058 で本文領域への `aria-labelledby` 設定を行った。

## 未決定

- 縦書き・ページ送り（ADR-0012 から継続）。
- EPUB 内 CSS の安全な適用（ADR-0012 から継続）。
