---
type: Decision
title: "ADR-0097: ローカルストレージのパス封じ込め"
description: "LocalStorage::resolve_path を fail-closed 化し、root 外へのパス脱出を拒否する決定を記録する。"
tags:
  - ADR
  - security
  - core
generated:
  by: dsh
  at: "2026-11-18T02:10:00+09:00"
status: accepted
---

## 背景

`core/src/io.rs` の `LocalStorage::resolve_path()` は絶対パスを無条件に採用し、
`..` も正規化せずに root へ結合していた（todo.md 独自最優先、code-scanner #1）。
Tauri コマンド経由で任意パスの読み書きが可能になり得る潜在リスクだった。

## 決定

- `resolve_path()` の戻り値を `CoreResult<PathBuf>` に変更する（private API のみ）。
- root を `canonicalize()` し、失敗時は fail-closed で拒否する。
- `Component::ParentDir`（`..`）を含む入力は機械的に拒否する。
- 絶対パスも canonicalize 後に root の子孫（`Path::starts_with`）なら許可し、
  root 外なら拒否する（desktop 側が絶対パスを渡す可能性を考慮）。
- 既存部分のみ `canonicalize()` して symlink を解決し、存在しない末尾は
  lexical に連結する（write / create_dir の未作成パスに対応）。
- エラーは既存の `CoreError::validation` / `storage` / `io` を使用する。
- `list()` の相対パス生成も正規化済み root を使う。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| root 配下のみ許可（採用） | 既存の相対パスと desktop の絶対パスを両立しつつ脱出を防ぐ |
| 絶対パスを全面拒否 | desktop の呼び出しが動かなくなる |
| 正規化なしで文字列比較 | symlink 脱出を防げない |

## 影響

- root 外への読み書き・一覧・削除・移動・メタデータ取得が拒否される。
- 通常の相対パス操作は変わらない。
- 公開 API（StorageBackend trait 等）は不変。
- 新規依存は無い。

## 検証

- core に 5 件のテストを追加: 相対パス解決、`../escape` 拒否、
  root 外絶対パス拒否、root 配下絶対パス許可、symlink 脱出拒否。
- `cargo test --all-targets --locked`: core 131 件 + desktop 2 件、すべて成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・WASM パス検査・
  E2E 228 項目がすべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `desktop/src/commands/document.rs:22,102` は backend を通さず `tokio::fs` を
  直呼びしており、今回の封じ込めが効かない（別タスク）。
- TOCTOU（チェック後の symlink 差し替え）は `O_NOFOLLOW` / `openat2` 等が
  必要で、新規依存なしの範囲では残る。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
