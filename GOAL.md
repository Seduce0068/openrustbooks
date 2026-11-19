# Goal

ADR-0003 に rumdl feature の現状注記を追加した（R6 R2-crates#2、ADR-0242）。

## Scope

### 成果物

- `docs/decisions/0003-native-wasm-target-split.md`: 現状注記を追加（既存本文は保持）
- `docs/decisions/0242-rumdl-feature-status.md`: 追加
- `todo.md`: R2-crates#2 を完了、#3-#5 は残存
- README / docs index / decisions index / log: 242 件へ更新

### 決定

- ADR-0003 は当時の履歴、現状は ADR-0242 を正とする。
- native / wasm とも rumdl featureless、wasm feature は init export 衝突で禁止。
- rumdl の workspace 化はしない。workspace 一元化（rumdl 以外）と Wasm API 非公開化は未決定。

## Acceptance criteria

- markdownlint 0 issues、verify.sh 成功（検証済み）。
