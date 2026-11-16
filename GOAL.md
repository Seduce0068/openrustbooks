# Goal

wasm32 ターゲットの clippy 警告を解消し、コードをクリーンに保つ。

## Scope

### 背景

native の 6 ゲートは `-D warnings` で通っているが、wasm32 ターゲットの
`cargo clippy` には警告が残っている。最近の変更で 2 件（初期化外のフィールド
代入、冗長な signal 再定義）が増えたため、まとめて解消する。

### 成果物

- `web/src/lib.rs`: 変更（警告の解消）
- `core/src/wasm.rs`: 変更（`WasmConfig` に `Default` を実装）
- `docs/decisions/0042-wasm-clippy-cleanup.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 警告の解消:
   - `WasmConfig` に `Default` を実装し、`new()` を `Default::default()` にする。
   - `load_settings()` の初回デフォルトを、代入ではなく構造体リテラルで作る。
   - `import_positions()` の `set_item(&key, ...)` を `set_item(key, ...)` にする。
   - 冗長な `let set_system_dark = set_system_dark;` を削除する。
   - `render_chapter` の複雑な型に `type` エイリアスを導入する。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 147 項目、読書位置、履歴、表示設定、日時表示、
     確認ダイアログ、カラースキーム追従、画像の非同期デコード、
     強制カラーモード対応を変えない。
3. 検証:
   - wasm32 の `cargo clippy` が警告 0 件になる（依存クレート由来の
     警告を除く）。

## Acceptance criteria

- 上記の必須事項が確認できる。
- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- 挙動を変えるリファクタリングをしない（警告解消のみ）。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
