# Goal

AI カスタムエンドポイント opt-in を設計受容として記録する（R6 TD-M17）。

## Scope

### 背景

`validate_base_url()` は OpenAI の公式ホストと Ollama loopback のみを許可する
allowlist を実装済み（ADR-0101）。カスタムホストを許可すると
`api_key` / `extra_headers` / 本文がそのまま送信され、
reqwest の既定リダイレクトや path/query の迂回も考慮が必要になる。
Web 設定フォームは読み取り専用（ADR-0128）、wasm AI は fail-closed で、
opt-in の需要先も設定面も未整備。

### 成果物

- `docs/decisions/0218-custom-ai-endpoint.md`: 追加
- `todo.md`: TD-M17 を完了（設計受容）へ移動
- `docs/index.md` / `docs/decisions/index.md` / `README.md` / `docs/log.md`: 更新
- `GOAL.md`: 本契約

### 必須事項

1. allowlist の現状契約と設計受容の根拠を ADR に明記する。
2. 将来 opt-in する場合の最小安全設計（フラグ配置、同意、許可対象、秘密、wasm）を記載する。
3. ADR-0101 の未決定を継続し、ADR-0100 / 0102 / 0104 / 0117 / 0128 を参照する。
4. markdownlint 0 issues、既存ゲート（E2E 253）を壊さない。

## Acceptance criteria

- markdownlint 0 issues、verify.sh 成功。

## Constraints

- コード・設定・E2E は変更しない。
- 秘密情報・トークンは ADR に書かない。
- `master` へ push する。force push はしない。
