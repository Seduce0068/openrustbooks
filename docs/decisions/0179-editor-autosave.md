---
type: Decision
title: "ADR-0179: エディタ下書きのデバウンス自動保存"
description: "明示 Save に加えてデバウンス自動保存を導入し、画面遷移時の下書き損失を防ぐ決定を記録する。"
tags:
  - ADR
  - web
  - ux
generated:
  by: dsh
  at: "2026-11-19T03:35:00+09:00"
status: accepted
---

## 背景

ADR-0164 で localStorage ドラフト保存を実装したが、保存は `Save draft`
クリック時のみだった。入力直後に画面遷移／リロードすると本文を失う
（R6: TD-H05）。

## 決定

- `on:input` 後、**800ms**（`EDITOR_AUTOSAVE_DELAY_MS`）のデバウンスで
  `save_editor_draft()` を呼ぶ。
- 入力直後は「未保存の変更があります」、自動保存成功で
  「下書きを自動保存しました」を `.editor-status`（`role="status"`
  / `aria-live="polite"`）へ表示する。
- 同一文言の再 `set` はスキップし、読み上げ過多を防ぐ。
  エラー時は ADR-0165 の共通通知のみで、成功文言を出さない。
- 上限超の入力は自動保存せず、固定文言
  「下書きが大きすぎて保存できません」のみを表示する。
- タイマーは `Rc<Cell<Option<i32>>>` を `SendWrapper` で保持し、
  `doc_id` 変更時と `on_cleanup` の両方で `clear_timeout_with_handle()` する。
  発火時はハンドルを `None` にし、世代カウンタと `doc_id` 照合で
  古い発火を破棄する。
- 手動 `Save draft` は保留タイマーを解除し、文言の上書きを防ぐ。
- `beforeunload` は **実装しない**。
  E2E は `Page.reload()` / `Page.navigate()` を 9 箇所で使うのに
  `Page.javascriptDialogOpening` ハンドラが無く、未保存中に
  `beforeunload` を発火させるとダイアログ待ちでハングするため。
  自動保存がデータ損失を防ぐ方針に統一する（GOAL 契約を変更）。
- 遅延は Web 側で固定値を使う（`core/src/config.rs` の
  `auto_save_delay: 2000` は web 未接続）。
- E2E に 1 件追加し、期待総数 247 → **248**（check 231 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| デバウンス自動保存（採用） | 遷移時の損失を防ぎ、読み上げも抑制できる |
| beforeunload のみ | E2E をブロックし、自動保存より信頼性が低い |
| 明示 Save のみ | データ損失リスクが残る |

## 影響

- 自動保存成功時も `console.error` は出さない。
- 上限超では `set_item` も共通通知も行わない（不要な error 連発を防止）。
- 同一本文の再試行はスキップし、不要な書き込みを防ぐ。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「Save クリックなしの自動保存でリロード後に本文が復元される」が ok。
- E2E **248 項目すべて成功**。
- 全単体テスト core 212 + desktop 59 + web 2、wasm-test 10、
  audit 脆弱性 0 件。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm check、
  `./scripts/verify.sh`: すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `auto_save_delay` を `WasmConfig` 等から渡すか（現状は固定 800ms）。
- `beforeunload` の再導入（CDP ダイアログハンドラが必要）。
- 未保存表示の non-live 領域への分離（現在は同一 polite 領域）。
- 下書きの寿命／prune、暗号化、複数タブ競合（既存未決定を継続）。
