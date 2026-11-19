---
type: Decision
title: "ADR-0211: 設定レイヤー境界の設計受容"
description: "3 設定レイヤーの権威・寿命分離を設計受容し、統合を却下する決定。"
tags:
  - ADR
  - config
  - web
  - desktop
generated:
  by: dsh
  at: "2026-11-19T11:31:05+09:00"
status: accepted
---

## 背景

PR-M08 / TD-M08 の残課題として、設定モデルが `AppConfig` / `ReaderSettings` /
`SettingsView` の 3 つ存在する点が指摘されていた。
第 4 のモデル `WasmConfig` は ADR-0205 で削除済みである。
監査の結果、残る 3 つは同じ値の重複コピーではなく、
core の永続設定 / web の localStorage UI 状態 / IPC 読み取り DTO という、
異なる権威と寿命を持つレイヤーであると判定された。
統合は ADR-0018 / 0038 / 0157 / 0193 の契約と E2E 251 を壊すリスクが高い。

## 決定

3 レイヤーを現状維持とし、レイヤー分離として設計受容する。

| レイヤー | 所在 | 権威と寿命 |
| --- | --- | --- |
| `AppConfig` | `core/src/config.rs` | core の権威。TOML / figment / env（`load` / `save` は native のみ） |
| `ReaderSettings` | `web/src/lib.rs` | ブラウザプロファイル単位。キー `openrustbooks:settings` |
| `SettingsView` | `web/src/lib.rs` | IPC 応答のスナップショット。非永続・秘密マスク |

- `AppConfig` は `config.toml` と `OPENRUSTBOOKS_*` 環境変数をディスク永続する
  （`#[cfg(not(target_arch = "wasm32"))]`）。
- `ReaderSettings` は wasm 単体で読み書き・永続が可能。
- `SettingsView` は `get_config` IPC の読み取り DTO で、
  未知フィールドは serde が無視する。

- localStorage 側のキー `openrustbooks:settings` と camelCase スキーマ 8 項目
  （`fontScale`, `lineHeight`, `dark`, `followSystem`, `swipeThreshold`,
  `verticalScrollThreshold`, `downscaleImages`, `jpegQuality`）を互換契約として固定する。
  IPC の `SettingsView` / `ConfigView` は snake_case の別スキーマであり、
  この camelCase 契約の適用範囲は localStorage 側に限定する。
- 8 項目のうち `fontScale`, `lineHeight`, `dark` は ADR-0018 由来、
  残り 5 項目は後続 ADR 由来である。
- 欠落は各項目の serde `default` で補い、範囲外は `normalized()`
  （`web/src/lib.rs:1607-1629`）で既定へ clamp する。
  不正 JSON は既定値へフォールバックする（`web/src/lib.rs:1649-1651`）。
- キーが無い初回起動のみ OS の dark に追従し `followSystem = true` とする
  （ADR-0038、`web/src/lib.rs:1641-1648`）。
  キーがある場合はインポート時に現在値を維持する
  （ADR-0157、`web/src/lib.rs:1955-1958`）。
- `SettingsView` は読み取り専用で、Web 側から `ConfigPatch` 保存は配線しない
  （desktop の `ConfigPatch` 境界は ADR-0131 / 0186 のとおり維持）。
- 既存 ADR の本文は履歴として保持し、本 ADR で設計受容を記録する。

## フィールド重複の扱い

重複と見える項目は `line_height` とテーマ系の 2 点に限られる。

- `lineHeight` は `ReaderSettings` の `LINE_HEIGHTS`（1.5 / 1.8 / 2.1）への
  usize インデックス（`web/src/lib.rs:1542`）、
  `AppConfig.reader.line_height` は f32 の倍率（`core/src/config.rs:352`）である。
  `core/src/config.rs:301` は `EditorConfig.line_height` であり、対象外である。
- `dark` / `followSystem` は Web の UI 状態（`web/src/lib.rs:1544-1548`）、
  `theme`（light / dark / system）は core の一般設定（`core/src/config.rs:38`）である。
- いずれも型・権威・寿命が異なり、値の一致を保証する契約は存在しない。
  そのため統合せず、各レイヤーの責任として扱う。

## 却下と根拠

`AppConfig` への一本化は以下の理由で却下する。

- wasm 単体で表示設定が永続化できなくなる（ADR-0018 / 0038 から後退）。
- ADR-0157 の現在値保護・`exportedAt` 拒否と衝突する。
- E2E 251 の設定関連チェックを壊す。
- DTO 境界の秘密マスキング（ADR-0128 / 0186）を損なう。

## 影響

- コード変更なし、E2E 251 維持。新規依存なし、unsafe なし。
- 既存の 3 モデルは API・データ形式ともに変更しない。
- 残課題は ADR の未決定として個別に追跡し、本 ADR では解決しない。

## 未決定

- `ReaderConfig` の死設定（TD-M19、`core/src/config.rs:291,342` /
  `web/src/lib.rs:3863,4004` の `SettingsReaderView.font_family`）。
  配線または削除の判断が必要。
- `ConfigPatch` の保存配線。ADR-0193 の supersede が必要。
- ADR-0179 の `auto_save_delay` の供給元。
- UI の設定保存接続（`withGlobalTauri` / CSP）。

## 参照

- [ADR-0018: リーダーの表示設定と読書データの削除](0018-reader-display-settings.md)
- [ADR-0038: 初回起動時のカラースキーム追従デフォルト](0038-initial-theme-follow.md)
- [ADR-0128: 設定フォームの読み取り表示と保存見送り](0128-settings-form-wiring.md)
- [ADR-0157: インポート時の設定・履歴保護](0157-import-preservation.md)
- [ADR-0186: 設定 IPC 境界の強化](0186-config-ipc-boundary.md)
- [ADR-0193: 設定フォームの読み取り専用化](0193-settings-form-readonly.md)
- [ADR-0205: WasmConfig の削除](0205-wasm-config-removal.md)
