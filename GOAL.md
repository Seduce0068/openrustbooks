# Goal

Web 版 EPUB リーダーに表示設定（文字サイズ・行間・ダーク表示）を追加し、
保存した読書データを削除できるようにする。

## Scope

### 背景

ADR-0012〜0017 で読書位置・履歴を扱うようにしたが、本文の表示は CSS 固定で
文字サイズ・行間を変えられず、ダーク表示も無い。また、保存した読書位置を
消す手段が無い。表示設定を localStorage に保存し、読書データを削除する UI を
追加する。

### 成果物

- `web/src/lib.rs`: 変更（設定の保存・適用、読書データ削除）
- `web/index.html`: 変更（CSS 変数とダーク表示）
- `tools/verify-web-reader.py`: 変更（設定と削除の E2E 検証を追加）
- `docs/decisions/0018-reader-display-settings.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 表示設定:
   - 文字サイズは 5 段階（1.0 / 1.125 / 1.25 / 1.5 / 1.75 rem、既定 1.125）。
   - 行間は 3 段階（1.5 / 1.8 / 2.1、既定 1.8）。
   - ダーク表示のオン・オフ（既定オフ）。
   - 設定は localStorage の `openrustbooks:settings` に JSON で保存し、
     起動時に読み込む。
   - 壊れた値・範囲外の値は既定へフォールバックする（fail-safe）。
   - 設定変更は即時に本文へ反映する（CSS 変数とクラス）。
   - 設定は本ごとではなく全体で共有する。
2. 読書データの削除:
   - `Clear data` ボタンで `openrustbooks:position:` で始まるキーを削除する。
   - 削除後に状態メッセージを表示する。
   - 表示設定（`openrustbooks:settings`）は削除しない。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 45 項目、サニタイズ、リンク、読書位置、履歴を
     変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 既定の文字サイズ（18px）で表示される。
  - A+ で文字サイズが上がり、localStorage に保存される。
  - リロード後も文字サイズが維持される。
  - 行間ボタンで行間が変わり、保存される。
  - ダーク表示のオン・オフが反映され、保存される。
  - `Clear data` で読書位置のキーが消え、設定は残る。
- 既存の 119 テストと E2E 45 項目が通る。

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
