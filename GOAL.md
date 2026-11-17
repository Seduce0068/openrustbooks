# Goal

アーキテクチャ設計書に現在の実装状況との対応関係を注記する。

## Scope

### 背景

調査レポートで「docs/architecture.md:9 が status: proposed のまま」と指摘された。
本文は初期提案（Leptos が候補、Tauri バージョン未決定など）のままで、
現在の実装（Leptos 0.8 / Tauri 2 / rumdl 採用など）と食い違う。
提案書として残す価値はあるため、status は変更せず、現状への注記を追加する。

### 成果物

- `docs/architecture.md`: 変更（現状対応の注記セクションを追加）
- `GOAL.md`: 本契約

### 必須事項

1. 文書の冒頭（フロントマターの直後）に、次を明記する。
   - 本文は初期提案時点のもので、現在の実装状況とは異なる。
   - 現在の採用技術は README と ADR を参照（Leptos 0.8、Tauri 2、rumdl など）。
   - status: proposed は初期提案書として残す意図によるもの。
2. 本文の既存記述（候補一覧など）は書き換えない。
3. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- 提案書としての履歴を保持する。
- `master` へ push する。force push はしない。
