# Goal

エクスポート形式 version 2 の必要性をレビューし、現状維持の根拠を記録する。

## Scope

### 背景

ADR-0076 の未決定「version 2 で追加する具体的なフィールド（現時点で必要な情報は無い）」。
ADR-0076 の方針では、version 1 のデータモデルで表現できない情報が必要になった
ときにのみ version を上げる。現時点でそのような要件が無いことを確認し、
version 1 維持の判断を記録する。

### 成果物

- `docs/decisions/0077-version-1-sufficiency.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. レビュー:
   - 現在の position / settings / history モデルが、実装済み機能の
     バックアップ要件を満たすことを確認する。
   - version 2 が必要になる条件（例: 複数ボックスの状態、注釈、外部
     ストレージ参照）を将来の候補として列挙する。
2. コードは変更しない。

## Acceptance criteria

- 判断が ADR に記録される。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- `master` へ push する。force push はしない。
