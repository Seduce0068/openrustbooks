---
type: Decision
title: "ADR-0047: 確認ダイアログの Esc キャンセル検証"
description: "削除確認ダイアログの Esc キーによるキャンセル動作を E2E で検証する決定を記録する。"
tags:
  - ADR
  - web
  - testing
  - accessibility
generated:
  by: dsh
  at: "2026-11-17T02:50:00+09:00"
status: accepted
---

## 背景

ADR-0040 でネイティブ `<dialog>` による削除確認を実装したが、
「Esc キーのキャンセル動作の明示的な E2E 検証」が未決定として残っていた。
ネイティブモーダルの Esc 処理に依存しているため、実際に履歴が消えずに
閉じることを確認する必要がある。

## 決定

- E2E に以下を追加する。
  - 削除ボタンでダイアログを開き、CDP の `Input.dispatchKeyEvent` で
    Escape キーを送ると、ダイアログが閉じる。
  - Esc では履歴件数が変わらない（キャンセルとして扱われる）。
- プロダクションコードは変更しない。ネイティブ `<dialog>` の標準動作を
  そのまま利用する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| CDP で Escape を送って検証（採用） | 実際のキーボード操作に近い形で確認できる |
| `dialog.close()` を直接呼ぶ | Esc の動作を検証できない |
| Esc ハンドラを独自実装 | ネイティブ機能の重複実装になる |

## 影響

- 確認ダイアログのキーボード操作（Esc キャンセル）が回帰から保護される。
- プロダクションコードの挙動は変わらない。
- 追加依存は無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 159 項目が
  5 回連続で成功。Esc でダイアログが閉じ、履歴件数が変わらないことを確認。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- ダイアログのフォーカスが Cancel に当たっていることの明示的な検証。
- 複数エントリの選択削除（ADR-0035 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
