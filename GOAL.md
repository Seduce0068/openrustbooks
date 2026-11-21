# Goal

IPC エラーの redaction を全 command へ統一する（R6 レビュー / R1-ER01）。

## Scope

### 背景

legacy config / AI / fs command は `map_err(e.to_string())` で
CoreError の Display 文字列をそのまま Tauri IPC へ返す。
CoreError は host / path / provider レスポンス本文を保持し得るため、
生文字列の露出経路が残っている。

### 成果物

- `core/src/error.rs`: 必要なら固定 code / 文言の追加
- `desktop/src/commands/*.rs`: 変更（生文字列 map_err の廃止）
- `web/src/lib.rs`: 必要なら code 解釈の追加
- `docs/decisions/0287-ipc-error-redaction.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. desktop command の Result<T, String> から CoreError::to_string() を返す箇所を廃止する。
2. AI provider エラーは固定 code + 固定文言へ変換し、host / body / API key を出さない。
3. fs / legacy config エラーは code 由来の固定文言へ変換する。
4. web 側 status が code を解釈できる場合、未知 code は安全な既定文言へ落とす。
5. 単体 test・全ゲート検証・E2E 256 件・WDIO ゲート維持。

## Acceptance criteria

- 生エラー文字列・host・path・response body が IPC 応答に現れない。

## Constraints

- 外部送信 / 課金 / 署名 / 公開設定は扱わない。
- E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。
