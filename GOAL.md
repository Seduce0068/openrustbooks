# Goal

初回起動時に OS がダークモードなら、カラースキーム追従を既定で有効にする。

## Scope

### 背景

ADR-0033 で `prefers-color-scheme` への追従を実装したが、初期値は常に
「追従しない」だった。OS がダークモードの環境では、初回起動時に読み手が
わざわざ追従ボタンを押す必要がある（ADR-0033 の未決定「追従の初期値
（OS がダークなら初回から追従を有効にするか）」）。

### 成果物

- `web/src/lib.rs`: 変更（`load_settings` の初回デフォルト）
- `tools/verify-web-reader.py`: 変更（初回デフォルトの E2E 検証を追加）
- `docs/decisions/0038-initial-theme-follow.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 初回デフォルト:
   - 設定キー（`openrustbooks:settings`）が **存在しない** 場合に限り、
     OS が `prefers-color-scheme: dark` なら `followSystem: true` を既定にする。
   - OS がライトなら従来どおり `followSystem: false`。
2. 後方互換（重要）:
   - 設定キーが **存在する** 場合は、JSON に `followSystem` が無ければ
     従来どおり `false` とし、既存ユーザーの選択を変えない。
   - 既に保存された `dark` / `fontScale` / `lineHeight` の値は変えない。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 138 項目、読書位置、履歴、表示設定、掃除、
     キーボード操作、タッチ操作、landmarks、page-list、履歴一覧、
     履歴クリア・個別削除、エクスポート・インポート、マージ、
     画像の非同期デコード、スクロール順序テストを変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 設定キーが無く OS がダークの初回起動で、追従が有効になる。
  - 設定キーが存在し `followSystem` が無い場合は、追従が無効のまま
    （後方互換）。
- 既存の 119 テストと E2E 138 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。保存先は localStorage。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
