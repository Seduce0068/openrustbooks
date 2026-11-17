# Goal

API キーなどの秘密情報の平文露出を減らす（todo.md D05 / code-scanner #2）。

## Scope

### 背景

`core/src/config.rs` の `api_key` / `client_secret` / `refresh_token` は
`config.toml` へ平文保存され、`desktop/src/commands/config.rs` の `get_config`
が `AppConfig` を丸ごと JS へ返す。UI 未接続のため顕在化はしていないが、
接続時に露出する潜在リスクがある（secret-auditor の精査結果）。

### 成果物

- `core/src/config.rs`: 変更（カスタム Debug で秘密を赤染め、保存ファイル権限 0600）
- `desktop/src/commands/config.rs`: 変更（get_config / reset_config をマスク済み DTO に）
- `docs/decisions/0098-secret-exposure-mitigation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 保存形式・読み込み互換性は維持する（`skip_serializing` は今回使わない。
   キーチェーン未実装で機能回帰するため）。
2. `get_config` / `reset_config` が返す JSON に秘密フィールド名・実値が含まれない
   許可リスト型 DTO を導入する。「設定済みか否か」は bool で伝える。
3. `update_config` / `update_ai_config` は、秘密フィールドがNone/空のとき
   保持中の実値をマージしてから保存し、キーが消えないようにする。
4. 秘密構造体の Debug を実値を出さない実装にする。
5. config.toml の保存を UNIX 系で 0600 にする。
6. 新規依存を追加しない。テストを追加する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- Web 版の UI 配線は変更しない（将来タスク）。
- OS キーチェーン移行は将来課題として残す。
- `master` へ push する。force push はしない。
