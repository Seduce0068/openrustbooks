# Goal

Web 版 EPUB リーダーにタッチ操作（スワイプで章を移動）を追加する。

## Scope

### 背景

`docs/requirements.md` の品質要件に「日本語IME、タッチ操作、キーボード操作を
検証する」があるが、リーダーはクリックとキーボードだけで操作でき、タッチ
デバイスではスワイプで章を移動できない（ADR-0021 の未決定「タッチ操作・
ジェスチャーの対応」）。

### 成果物

- `web/src/lib.rs`: 変更（タッチイベントの監視とスワイプ判定）
- `web/Cargo.toml`: 変更（`web-sys` のタッチ機能を追加）
- `tools/verify-web-reader.py`: 変更（スワイプの E2E 検証を追加）
- `docs/decisions/0025-touch-gestures.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. スワイプ:
   - 本文領域（`.reader-content`）の `touchstart` / `touchend` を監視する。
   - 水平方向の移動が 50px 以上、垂直方向の移動が 30px 未満のとき
     スワイプと判定する。
   - 左スワイプ（指を左へ動かす）で次の章、右スワイプで前の章へ移動する。
   - 縦スクロール中のジェスチャーは無視する（垂直移動が 30px 以上）。
   - マルチタッチ（2本指以上）は無視する。
   - スワイプ中は `prevent_default` しない（縦スクロールを妨げない）。
2. ARIA:
   - 本文領域に `aria-label` を維持する（既存）。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 97 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 左スワイプで次の章へ進む。
  - 右スワイプで前の章へ戻る。
  - 縦移動が大きいジェスチャーでは章が変わらない。
  - マルチタッチでは章が変わらない。
- 既存の 119 テストと E2E 97 項目が通る。

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

- 新規依存を追加しない。既存の `web-sys` 機能の範囲で実装する。
- 外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
