# Goal

`Cdp.send()` の `recv()` にタイムアウトを設ける（ADR-0142 / 0143 の未決定）。

## Scope

### 背景

`tools/verify-web-reader.py:398` の `await self.ws.recv()` にはタイムアウトが無く、
CDP が応答を返さない場合 `wait_until` の 20 秒 wall-clock deadline も効かず
E2E 全体がハングし得る（ADR-0142 / 0143 の未決定）。

### 成果物

- `tools/verify-web-reader.py`: 変更（`asyncio.wait_for` による recv タイムアウト）
- `docs/decisions/0144-cdp-recv-timeout.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `Cdp.send()` の `recv()` を `asyncio.wait_for` で包み、既定タイムアウト（例 10 秒）を設定する。
2. タイムアウト時は `RuntimeError`（または同等）で失敗させ、診断情報を添える（fail-closed）。
3. 通常の CDP 応答・診断イベント退避は維持する。
4. E2E 期待総数 229 は維持する。
5. 5 回連続実行で 0 失敗、全ゲート成功。

## Acceptance criteria

- E2E 229 項目 5 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存を追加しない（Python 標準 `asyncio` のみ）。
- CDP プロトコルの意味（id 照合、イベント退避）は変更しない。
- `master` へ push する。force push はしない。
