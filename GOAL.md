# Goal

`cargo-audit` 未インストール時に脆弱性ゲートを警告だけでスキップする問題を修正する（R4 指摘）。

## Scope

### 背景

`scripts/verify.sh` は `command -v cargo-audit` が失敗すると
`WARNING` を出すだけで `fail=1` にしない。
ローカルにツールが無いだけで脆弱性検査が黙ってスキップされ、
CI（`audit: true`）と結果が食い違う。

### 成果物

- `scripts/verify.sh`: 変更（未インストール時もゲート失敗）
- `README.md`: 必要なら導入手順の明記
- `docs/decisions/0185-audit-tool-failclosed.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `cargo-audit` が無い場合、明確なエラーを出して `fail=1` にする。
2. advisory DB が無い／古い場合の fail-closed 契約は維持する。
3. 既知 baseline（警告 10 件）は従来どおり許可する。
4. `./scripts/verify.sh` が成功すること（ローカルに cargo-audit が入っている前提）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない。
- CI 共通ワークフロー変更は行わない。
- `master` へ push する。force push はしない。
