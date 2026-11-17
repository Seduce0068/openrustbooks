# Goal

共通ワークフローの `extra_gates` で `cargo audit` を実行できるか調査・判断する（ADR-0125 の未決定）。

## Scope

### 背景

`.github/workflows/ci.yml:41-43` は `extra_gates` に wasm32 check を渡している。
`cargo audit` をここに追加できれば、CI でも脆弱性回帰を検出できる。
ただしグローバル指示 5 では「CI の変更はまず共通ワークフロー側で対応できないか検討する」とされる。

### 成果物

- `.github/workflows/ci.yml`: 変更（可能なら audit ゲートを `extra_gates` に追加）
- `docs/decisions/0126-ci-audit-gate.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 共通ワークフロー `Seduce0068/workflows/rust-gates.yml@v1` の `extra_gates` 入力の実体を確認する（GitHub MCP または gh api）。
2. `extra_gates` が任意コマンドを実行できるなら `cargo audit` を追加する。cargo-audit のインストール手順も必要なら検討。
3. 変更不能／危険なら「未対応」を明文化する。
4. ローカル検証（verify.sh）は維持する。

## Acceptance criteria

- 判断が ADR に記録される。
- ローカル検証が成功する。

## Constraints

- 共通ワークフロー自体（`Seduce0068/workflows`）は変更しない。
- `@v1` ピン留めは維持する（グローバル指示 5）。
- `master` へ push する。force push はしない。
