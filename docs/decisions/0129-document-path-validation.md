---
type: Decision
title: "ADR-0129: document command のパス封じ込め"
description: "desktop の document command で AppConfig.local_roots によるパス検証を行う決定を記録する。"
tags:
  - ADR
  - security
  - desktop
generated:
  by: dsh
  at: "2026-11-18T09:50:00+09:00"
status: accepted
---

## 背景

ADR-0097 で `LocalStorage::resolve_path()` を fail-closed 化したが、
`desktop/src/commands/document.rs` の `open_document` / `save_document` は
backend を通さず `tokio::fs` を直呼びしていた（todo.md 最優先の残り）。
UI 未接続のため顕在化していないが、接続時に任意パス読書きのリスクがある。

## 決定

- `AppConfig.storage.local_roots` を root として、入力パスを検証する
  `resolve_within_roots()` を追加する（ADR-0097 と同じ方針）。
  - root を `canonicalize()`（正規化できない root はスキップ）。
  - 有効な root が無ければ `CoreError::storage("No storage root configured")`
    で fail-closed 拒否。
  - `Component::ParentDir`（`..`）は機械的に拒否。
  - 絶対パスも root 配下なら許可。相対パスは root に join して判定。
- 既存部分を canonicalize し、存在しない末尾を lexical に連結する
  `canonicalize_existing_prefix()` を使い、**未作成の書き込み先にも対応**。
- 検証で得た解決済みパスを I/O にそのまま使用する（相対パスを検証だけして
  CWD 基準で読む不一致を防ぐ）。`DocumentMetadata.path` も解決済みパスになる。
- `core/src/io.rs` の `resolve_path()` と backend 経由 command は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| local_roots で検証（採用） | backend に依存せず回帰なく封じ込められる |
| backend 経由に置換 | local_roots 空で回帰するとの旧判断を維持 |
| 検証なし | 任意パス読書きリスクが残る |

## 影響

- `local_roots` 未設定（既定は空）の環境では `open_document` /
  `save_document` が新たにエラーになる。UI 未接続のため即時の実回帰は無いが、
  ADR-0127 段階 2 の Open File 接続時には root 設定が必要。
- 相対パスの解決先が CWD 基準から root 基準に変わる（Tauri の
  ファイルダイアログは絶対パスを返すため実害は想定されない）。
- 新規依存なし、unsafe なし。

## 検証

- 追加テスト 5 件（root 配下相対許可、`..` 拒否、root 外絶対拒否、
  root 配下絶対許可、空 root 拒否）。
- `cargo test --all-targets --locked`: core 186 + desktop 33 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（doc-path-auditor の指摘）

- ファイルダイアログの scope（`$HOME/*` 等）と local_roots の信頼境界を
  一体で設計する（ダイアログ開始ディレクトリを root に制限するか、
  root 外選択時に明示同意で追加するか等）。
- `local_roots[].path` が相対／存在しない場合の扱い（今回はスキップ、
  fs 側は `create_dir_all` で暗黙作成、という挙動差が残る）。
- `save_document` の `read_only` フラグ反映（今回は未実装）。
- TOCTOU（check と I/O の間の symlink 差し替え）は ADR-0097 と同様に未解決。
- 実行中に root が変更された場合の、開済み document の save 扱い。
- エラー文言への絶対パス含有（core/src/io.rs の既存メッセージは
  パスを露出しており ADR-0103 の方針と緊張関係にある）。
