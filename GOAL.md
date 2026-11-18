# Goal

CodeMirror エディタ UI 接続の現状と段階導入方針を調査・判断する（todo.md A05 / ADR-0012 から継続）。

## Scope

### 背景

`web/src/lib.rs` の Editor コンポーネントは Save / AI Generate / AI Refine /
Lint ボタンがすべてハンドラ無しのスタブで、CodeMirror の統合も
`Effect::new` のログだけ。デスクトップ側には document 系 command が
登録済みだが、Web 側に Tauri invoke は無い。

### 成果物

- `docs/decisions/0127-editor-ui-integration.md`: 追加（調査と判断）
- `GOAL.md`: 本契約

### 必須事項

1. Web Editor / SettingsForm の未配線箇所を列挙する。
2. デスクトップ側 command と Web UI をつなぐ最小経路を設計する。
3. 新規依存（CodeMirror の wasm バインディング等）が必要か調査する。
4. 実装する／見送る／段階導入するかを判断し記録する。
5. コード変更は行わない（調査と決定のみ）。

## Acceptance criteria

- 判断が ADR に記録される。
- markdownlint が通る。

## Constraints

- コードを変更しない。
- 新規依存を追加しない（調査のみ）。
- `master` へ push する。force push はしない。
