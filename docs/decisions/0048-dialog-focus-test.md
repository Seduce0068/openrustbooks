---
type: Decision
title: "ADR-0048: 確認ダイアログのフォーカス検証"
description: "削除確認ダイアログで安全な操作（Cancel）にフォーカスが当たることを E2E で検証する決定を記録する。"
tags:
  - ADR
  - web
  - testing
  - accessibility
generated:
  by: dsh
  at: "2026-11-17T03:02:00+09:00"
status: accepted
---

## 背景

ADR-0047 の未決定「ダイアログのフォーカスが Cancel に当たっていることの明示的な検証」。
`autofocus` 属性を付けているが、ネイティブ `<dialog>` の `showModal()` では
効かないケースもあるため、実際のフォーカスを確認する必要がある。

## 決定

- E2E に以下を追加する。
  - 削除ボタンでダイアログを開いた直後、`document.activeElement` の
    テキストが `Cancel` であること。
- プロダクションコードは変更しない。現状の `autofocus` 属性が
  Chrome 157 では正しく機能していることをデバッグで確認済み。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `document.activeElement` を検証（採用） | 実際のフォーカス位置を直接確認できる |
| `autofocus` 属性の存在を検証 | 実際に効いているか分からない |
| 手動で `focus()` を呼ぶ | ネイティブ動作の重複実装になる |

## 影響

- 破壊的な操作（Delete）に誤ってフォーカスが当たる回帰を防ぐ。
- プロダクションコードの挙動は変わらない。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 160 項目が
  5 回連続で成功。ダイアログを開いた直後に Cancel にフォーカスが当たる
  ことを確認。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 複数エントリの選択削除（ADR-0035 から継続）。
- CSS の `url()` による外部参照（EPUB 内 CSS の安全な適用と併せて検討）。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
