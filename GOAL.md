# Goal

E2E の CDP 接続先を起動した Chrome プロセスに限定する（ADR-0090 / 0142 の未決定）。

## Scope

### 背景

`tools/verify-web-reader.py:3355-3360` は `/json` の `pages[0]` に無条件接続する。
固定 DEBUG_PORT=9222 に既存 Chrome が残っている場合、誤ったページへ接続して
起動待ちが 20 秒で失敗する可能性がある（ADR-0142 の原因候補 A）。

### 成果物

- `tools/verify-web-reader.py`: 変更（起動 PID でターゲットを絞る）
- `docs/decisions/0143-cdp-target-matching.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `/json` の target から、起動した Chrome に属する page を特定する。
   - `webSocketDebuggerUrl` の `devtools/browser/<id>` と
     `/json/version` の `webSocketDebuggerUrl` を照合する方法、
     あるいは起動時に `--user-data-dir` を既知の一意パスにして
     target の `url` が `about:blank` であることを確認する方法等を検討。
2. 既存 Chrome が残存していても誤接続しないことをテスト／検証で確認する。
3. E2E 期待総数 229 は維持する。
4. 5 回連続実行で 0 失敗を確認する。
5. 全ゲートが成功する。

## Acceptance criteria

- 誤接続を防ぎ、E2E 229 項目 5 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 固定ポートの変更は不要（_pid 照合で解消する）。
- 新規依存を追加しない（Python 標準ライブラリのみ）。
- `master` へ push する。force push はしない。
