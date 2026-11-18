# Goal

Ollama ストリームの行境界処理と HTTP タイムアウトを修正する（R6 PR-M01 / TD-M04）。

## Scope

### 背景

`OllamaTokenStream::next_token()` はチャンクごとに `text.lines()` をパースするため、
JSON 行がチャンク境界で分断されると不正として捨てられる。
複数行が1チャンクに入ると最初の response だけ返し残りを失う。
また `reqwest::Client::new()` に timeout が無く、無応答の Ollama で
Tauri コマンドが戻らない。

### 成果物

- `core/src/ai_markdown/client.rs`: 変更（行バッファ、timeout、テスト）
- `docs/decisions/0189-ollama-stream-boundary.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. Ollama ストリームに未完了行を保持するバッファを追加し、
   改行で終わった完全な JSON 行だけをパースする。
2. 1 チャンクに複数行入っても、キューに残して順次 `next_token()` が返す。
3. `OllamaClient::new()` で `Client::builder().timeout(...)` を設定する。
   タイムアウト値は既存 `timeout_secs` 設定が利用可能ならそれを使う。
4. UTF-8 は累積バイトに対してデコードし、途中のマルチバイト文字を保持する。
5. unit test を追加（分断行、複数行、done、途中エラー）。
6. 全ゲート成功（E2E 249 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存を追加しない（bytes / futures / tokio 既存を利用）。
- OpenAI ストリームの挙動は変更しない。
- 設定に timeout が無い場合は安全な既定値（例 60 秒）を使う。
- `master` へ push する。force push はしない。
