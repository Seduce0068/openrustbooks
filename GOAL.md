# Goal

THIRD_PARTY_NOTICES.md を配布依存の実態へ合わせる（R6 PR-M12、ADR-0199）。

## Scope

### 成果物

- `THIRD_PARTY_NOTICES.md`: 旧 MIT / imagesize を保持し、外部依存 781 件の表を追加
- `docs/decisions/0199-third-party-notices.md`: 追加
- `todo.md`: PR-M12 を完了へ移動

### 決定

- 出典は cargo metadata / Cargo.lock（cargo-deny は未導入・未使用）。
- normal/build 依存のみ辿り、dev 9 件を除外。
- ライセンス ID と crates.io URL を記載し、本文は推測補完しない。

## Acceptance criteria

- markdownlint 0 issues、既存ゲート・E2E 251 維持（検証済み）。
