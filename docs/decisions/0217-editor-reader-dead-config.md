---
type: Decision
title: "ADR-0217: Editor / Reader 死設定の正直化"
description: "config.toml の Editor / Reader 値を DTO に残しつつ、動作に未接続であることを設定画面へ明記する決定を記録する。"
tags:
  - ADR
  - web
  - config
generated:
  by: dsh
  at: "2026-11-19T13:26:00+09:00"
status: accepted
---

## 背景

`AppConfig.editor` / `.reader` は `ConfigView` 経由で設定画面に表示されるが、
Editor / Reader の実動作には接続されていなかった（R6 TD-M19）。
読書表示の権威は localStorage の `ReaderSettings`（`openrustbooks:settings`、
ADR-0211）、エディタ下書きは別キー（ADR-0179 / 0180）であり、
`streaming` 設定（ADR-0216）と同型の誤認リスクがあった。

## 決定

- フィールドは削除しない。`ConfigView` / `ConfigPatch` /
  `SettingsEditorView` / `SettingsReaderView` の DTO 形状を維持する。
- 設定画面へ注記を追加する:
  - Editor: `エディタ設定: config.toml の設定値です。エディタの動作には未接続です。`
  - Reader: `リーダー設定: config.toml の設定値です。実際の表示はブラウザに保存された表示設定が権威です（ADR-0211）。`
- 既存の値表示・`reader_info`・既存 `settings-notice` は維持する。
- E2E は追加せず、期待総数 253 を維持する（新文言への依存なし）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設計受容＋注記（採用） | DTO / config 互換を壊さず誤認も解消できる |
| フィールド削除 | config.toml の値が次回 save で黙って消え、DTO / テストも横断変更が必要 |
| 放置 | 利用者が「読書に効いている」と誤認する |

## 影響

- 設定画面に本来の権威（localStorage / 下書き）への案内が表示される。
- `config.toml` の `[editor]` / `[reader]` 値自体は保持される。
- 新規依存なし、unsafe なし、E2E 253 維持。

## 検証（Runtime Proof）

- E2E に新文言の依存なし（`tools/` / `scripts/` grep 一致 0）。
- `cargo fmt --all -- --check`、
  `cargo check --all-targets --locked`、
  `cargo test --all-targets --locked`（core 261 + desktop 79 + web 2）、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check: すべて成功。
- `./scripts/verify.sh`: 全ゲート成功、E2E **253 項目すべて成功**。
- 検証中に `/tmp` の tmpfs が満杯になり E2E が
  `Errno 28` で一度失敗したが、古い一時キャッシュを削除後、
  同じコードで rc=0 を再確認した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `EditorConfig` / `ReaderConfig` のフィールド削除または実際の配線
  （config.toml 移行方針を含む）。
- `auto_save_delay`（config 既定 2000ms に対し web 実装は固定 800ms、ADR-0179）。
- `vertical_writing` / `scroll_mode` / `hyphenation` / `column_width` の段階導入
  （ADR-0145 継続）。
- `ConfigPatch` 保存配線（ADR-0193 supersede、ADR-0211 未決定）。
- env 3 キー（`OPENRUSTBOOKS_EDITOR_MINIMAP` /
  `OPENRUSTBOOKS_READER_THEME` /
  `OPENRUSTBOOKS_READER_HYPHENATION`）の維持・削除。
