# Goal

`remote-resources` を宣言する EPUB を開いたとき、利用者に警告を表示する。

## Scope

### 背景

ADR-0060 の未決定「`remote-resources` 宣言を根拠に、利用者へ警告を表示するか」。
宣言を持つ EPUB は受け付ける方針だが、外部資源を参照する可能性があることを
利用者に知らせることで、_network アクセスへの意識を高める。

### 成果物

- `core/src/epub.rs`: 変更（`has_remote_resources()` を追加）
- `core/src/wasm.rs`: 変更（wasm バインディングを追加）
- `web/src/lib.rs`: 変更（警告表示）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0066-remote-resources-warning.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 検出:
   - `EpubBook::has_remote_resources()` を追加し、manifest item の
     `properties` に `remote-resources` を含む場合に true を返す。
   - wasm バインディング経由で Web から参照できるようにする。
2. 警告:
   - 本を開いたとき、宣言があれば状態メッセージに
     `This EPUB declares remote resources.` を表示する。
   - 宣言が無い場合はメッセージを出さない（既存サンプルは対象外）。
   - 警告は表示のみで、読み込みや表示を阻止しない（ADR-0060 の方針維持）。
3. 既存の挙動を変えない:
   - 既存の 125 テスト、E2E 187 項目（巨大画像警告を含む）を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 既存サンプル（宣言なし）では警告が出ない。
  - `remote-resources` を宣言する EPUB で警告が出る。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
