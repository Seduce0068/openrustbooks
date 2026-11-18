# Goal

cargo-audit 未インストール時の検証スキップを防ぐ（R4 指摘、ADR-0185）。

## Scope

### 成果物

- `scripts/verify.sh`: 未インストール時に ERROR + fail=1
- `README.md`: 前提条件を明記
- `docs/decisions/0185-audit-tool-failclosed.md`: 追加
- `todo.md`: 当該指摘を完了へ移動

### 決定

- 未インストール時は E2E へ進まず rc=1。
- --no-fetch / baseline 警告 10 件は維持。
- DB 鮮度は運用責任（--no-fetch が stale 判定を無効化）。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
