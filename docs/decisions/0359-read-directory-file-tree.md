---
type: Decision
title: "ADR-0359: read_directory を FileTree へ接続"
description: "document / fs IPC の最初のスライスとして read_directory をブラウザの FileTree へ接続し、View DTO と静的ゲートを更新する決定。"
tags:
  - ADR
  - ipc
  - web
generated:
  by: dsh (lead)
  at: "2026-11-28T16:15:17+09:00"
status: accepted
---

## 背景

desktop には document / fs command が登録済みだが、web の FileTree は固定文言のみだった。
ADR-0127 の段階方針に沿い、書き込みを伴わない一覧取得から接続する。

## 決定

- 接続対象は `read_directory` のみ。`open_document` / `save_document` 等は段階 2 で別途判断する。
- 新規依存は追加しない。既存の `invoke_update` / `serde_json` / `serde_wasm_bindgen` を再利用する。
- 引数は `path: ""`（既定 root）と固定オプションのみ。生 path やユーザー名を JS から組み立てない。
- web 側 mirror は desktop の `FileEntryView` と同一 6 フィールドとし、`deny_unknown_fields` で path 混入を拒否する。
- DOM に出すのは `name` と `is_dir` 由来の表示のみ。失敗時は固定日本語文言だけを status へ出す。
- `check-capability-scope.sh` の許可集合へ `read_directory` を追加し、設定 4 command と fs 側を別検査する。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked`（35 passed）/
  `cargo clippy --all-targets --locked -- -D warnings` / wasm ターゲット check: rc=0。
- `bash scripts/check-capability-scope.sh`: rc=0（5 command に限定、source inspection のみ）。
- `bash scripts/verify.sh`: rc=0（E2E 273 passed / 0 failed、WDIO 2 spec 成功）。

## 未決定

- ファイル選択→`open_document`→`/editor/<UUID>` の段階 2 接続と、localStorage ドラフトの優先順位。
- 複数 storage root の明示選択（ADR-0321 の未決定を継承）。

## 参照

- ADR-0127: editor UI 接続の段階方針
- ADR-0350: document / fs の生 path 除外
- ADR-0355: capability 静的ゲート（4 command は当時点の記録）
