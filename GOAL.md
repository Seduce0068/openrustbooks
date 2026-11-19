# Goal

OS キーチェーン移行を設計受容として記録する（R6 D05）。

## Scope

### 背景

ADR-0098 は API キー / client_secret / refresh_token の
平文 config.toml 保存を維持し、キーチェーン移行を「新規依存と設計判断が必要」
として見送っていた。監査で、redaction / DTO 境界 / Unix 0600 は維持済みだが、
Wasm 対象外・移行手順未定義・CI で実キーストア検証不可・
コードコメントの「encrypted」と実装の矛盾が確認された。

### 成果物

- `docs/decisions/0261-os-keychain-deferred.md`: 追加
- `docs/decisions/0098-secret-exposure-mitigation.md`: 変更（未決定の追記）
- `todo.md`: D05 を設計受容として完了へ移動
- `docs/runbook-release.md`: 既知問題の根拠を ADR-0261 へ追記
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 平文保存 + redaction + Unix 0600 + api_key のみ IPC 書き込み可を
   受容済みリスクとして明記する。
2. Wasm はキーチェーン対象外、CI では実キーストアを検証できないことを記録する。
3. 新規依存（keyring / secret-service）・移行順序・ローテーション・
   プラットフォーム差を未決定として ADR-0098 / ADR-0261 へ集約する。
4. 「encrypted」コメントと実装の矛盾を開示する（コードは変更しない）。
5. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 規約・索引・todo が整合し、全ゲート成功。

## Constraints

- コード / Cargo.toml / Cargo.lock / E2E は変更しない。
- 新規依存・外部サービス・秘密情報は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- ADR-0261 で平文保存 + redaction + Unix 0600 を受容済みリスクとして記録。
- ADR-0098 の未決定 6 項目を ADR-0261 へ集約、runbook / todo / 索引を追従。
- `./scripts/verify.sh`: 全ゲート成功、E2E 255 passed / 0 failed。
