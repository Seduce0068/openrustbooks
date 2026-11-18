# Goal

E2E の起動待ち早期 abort（ADR-0140 第 10 回）を調査・安定化する。

## Scope

### 背景

10 回連続実行の第 10 回で `Reader ルートが表示される` が 20 秒待っても false となり、
`expected 229 checks, but executed 1` で早期終了した。
位置復元ロジックではなく Wasm 起動／描画待ちのフレークと見られる。

### 成果物

- `tools/verify-web-reader.py`: 変更（必要なら診断ログと再試行）
- `web/src/lib.rs`: 変更（必要なら起動_error の可視化）
- `docs/decisions/0142-startup-flake.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 20 秒待っても `.reader-container` が無い場合の原因を切り分ける（Wasm 読込失敗／JS エラー／サーバ応答遅延／ブラウザ起動競合）。
2. 診断ログ（console / pageerror / レスポンス）を追加して再現時に理由が分かるようにする。
3. 必要なら起動待ちを再試行するが、期待総数 229 は維持する。
4. 5 回連続実行で 0 失敗を確認する。
5. 全ゲートが成功する。

## Acceptance criteria

- E2E 229 項目、5 回連続 0 失敗。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- アプリの機能は変更しない（診断と待機の安定化のみ）。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
