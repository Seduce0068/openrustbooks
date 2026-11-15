# Goal

Web 版 EPUB リーダーにキーボード操作を追加し、リーダーの主要な操作を
スクリーンリーダーへ伝える（ARIA）。

## Scope

### 背景

`docs/requirements.md` の品質要件に「日本語IME、タッチ操作、キーボード操作を
検証する」があるが、リーダーはボタンのクリックと目次の選択だけで操作でき、
キーボードだけでは章を移動できない。また、アイコンだけのボタン（`◀` `▶`）や
本文領域にアクセシブルな名前が無く、読み上げで用途が分からない。

### 成果物

- `web/src/lib.rs`: 変更（キーボード操作、ARIA 属性、ハンドラの共通化）
- `tools/verify-web-reader.py`: 変更（キーボードと ARIA の E2E 検証を追加）
- `docs/decisions/0021-keyboard-and-aria.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. キーボード操作（window の keydown を監視する）:
   - `]` → 次の章、`[` → 前の章。
   - `Alt+ArrowLeft` → 戻る、`Alt+ArrowRight` → 進む。
   - `+` / `=` → 文字サイズを大きく、`-` / `_` → 小さく。
   - `d` → ダーク表示の切り替え。
   - フォーカスが `input`・`select`・`textarea` にあるときは無視する
     （目次選択中の文字入力を奪わない）。
   - `Ctrl` / `Meta` 併用時は無視する（ブラウザのショートカットを奪わない）。
   - 処理したキーは `prevent_default` する（`[`/`]` などでページが
     スクロールしないようにする）。
   - リーダーを離れるときにリスナーを解除する（`on_cleanup`）。
2. ARIA:
   - アイコンだけのボタンに `aria-label` を付ける（`◀` `▶`、`A-` `A+` など）。
   - 状態メッセージに `aria-live="polite"` を付ける。
   - 本文領域を `role="region"`・`aria-label`・`tabindex="0"` にし、
     キーボードでフォーカスしてスクロールできるようにする。
   - 章の選択に `aria-label` を付ける。
   - キー操作に対応するボタンへ `aria-keyshortcuts` を付ける。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 73 項目、読書位置、履歴、表示設定、掃除を変えない。
   - ボタンの見た目と `id` を変えない（E2E が参照する）。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - `]` で次の章へ進み、`[` で前の章へ戻る。
  - `Alt+ArrowLeft` で履歴を戻り、`Alt+ArrowRight` で進む。
  - `+` で文字サイズが上がり、`-` で下がる。
  - `d` でダーク表示が切り替わる。
  - 目次の `select` にフォーカスがあるときは `d` が効かない。
  - `Ctrl` 併用時はキー操作が起きない。
  - `#reader-font-larger` に `aria-keyshortcuts` が付く。
  - 本文領域に `role`・`aria-label`・`tabindex` が付く。
  - 状態メッセージに `aria-live` が付く。
- 既存の 119 テストと E2E 73 項目が通る。

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
