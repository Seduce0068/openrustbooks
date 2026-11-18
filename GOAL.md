# Goal

`THIRD_PARTY_NOTICES.md` を配布依存の実態へ合わせる（R6 PR-M12）。

## Scope

### 背景

現状の `THIRD_PARTY_NOTICES.md` は旧 MIT 本文と imagesize 0.15.0 のみを記載しており、
配布 WASM / デスクトップに含まれる多数の依存クレートの通知が無い。
AGPL-3.0-or-later 変更時も上流の著作権表示・許諾通知を保持する方針が書かれている。

### 成果物

- `THIRD_PARTY_NOTICES.md`: 変更（依存一覧／ライセンス出典の整備）
- `docs/decisions/0199-third-party-notices.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `cargo metadata` / `Cargo.lock` を出典に、配布に関係する依存を網羅した一覧を記載する。
2. 既存の旧 MIT 本文と imagesize 通知は保持する（履歴として明示）。
3. 各依存の名前・バージョン・ライセンス識別子・出典（crates.io URL）を記載する。
4. 自動生成セクションはコメントで区切り、手動追記と混同しないようにする。
5. markdownlint が通る。コード変更なし（E2E 251 維持）。

## Acceptance criteria

- markdownlint 0 issues、全ゲート（既存の緑）を維持。

## Constraints

- 新規依存・新規ツールを追加しない（既存の `cargo metadata` / `cargo deny` を利用）。
- ライセンス本文を推測で埋めない。確認できない項目は「未確認」と明記。
- `master` へ push する。force push はしない。
