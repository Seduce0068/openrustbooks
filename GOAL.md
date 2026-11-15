# Goal

リーダーのダーク表示トグルをアプリ全体（ツールバー・一覧画面を含む）に
拡張する。

## Scope

### 背景

ADR-0018 まででダーク表示は `.reader-content` にしか適用されず、ツールバー
や一覧画面はライトのままだった（ADR-0018 の未決定「アプリ全体のテーマ
（ツールバー・一覧画面を含む）」）。手動トグルを `<html>` のクラスへ反映し、
CSS 変数を切り替える。

### 成果物

- `web/src/lib.rs`: 変更（`<html>` へのクラス反映）
- `web/index.html`: 変更（`.dark` クラスで CSS 変数を切り替える）
- `tools/verify-web-reader.py`: 変更（全体テーマの E2E 検証を追加）
- `docs/decisions/0023-app-wide-theme.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. トグル:
   - `Dark` / `Light` ボタンで `<html>` に `dark` クラスを付け外しする。
   - 設定は localStorage `openrustbooks:settings` の `dark` に保存する
     （既存）。
   - 起動時に保存値を読んで `<html>` のクラスへ反映する。
   - `prefers-color-scheme: dark` の既定は維持する（手動トグルが無い場合）。
2. CSS:
   - `html.dark` で `--color-bg` / `--color-fg` / `--color-muted` /
     `--color-border` / `--color-primary` / `--color-secondary` をダーク値へ
     上書きする。
   - `.reader-content` の個別ダークスタイルは `html.dark` に統合する。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 93 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - `Dark` で `<html>` に `dark` クラスが付く。
  - リロード後も `dark` クラスが維持される。
  - `Light` で `dark` クラスが外れる。
  - ダーク時にツールバーの背景がダークになる。
- 既存の 119 テストと E2E 93 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --locked`
  - `cargo test --all-targets --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  - `trunk build --offline --dist <一時出力先>`（web）
  - `python3 tools/verify-web-reader.py --dist <出力先>`（E2E）
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## Constraints

- 新規依存を追加しない。保存先は localStorage のみ。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
