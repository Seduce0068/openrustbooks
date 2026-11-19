# Goal

検証済み配布物 manifest にブラウザ識別子を記録する（ADR-0265 の未決定を解消）。

## Scope

### 背景

ADR-0265 で Chrome / Edge の E2E 255/0 を実機確認したが、
`scripts/verify.sh --dist` の manifest には e2e expected / result のみが記録され、
どのブラウザ・ポートで検証したかが残らない。
ADR-0219 の完全性記録と ADR-0265 の cross-browser 証跡を接続する。

### 成果物

- `scripts/verify.sh`: 変更（E2E 引数の明示化と manifest へのブラウザ情報記録）
- `docs/decisions/0266-browser-metadata-manifest.md`: 追加
- `docs/runbook-verified-build.md`: 変更（manifest 項目の説明）
- `docs/decisions/0265-cross-browser-e2e.md`: 変更（未決定の解消）
- `todo.md`: ブラウザメタデータ記録を完了へ反映
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. E2E 起動で使用するブラウザ実行ファイル / ポートを変数へ集約し、
   `python3 tools/verify-web-reader.py` へ明示的に渡す。
2. manifest に `browser executable` / `browser version` / `static port` /
   `cdp port` を記録する。
3. ブラウザ version 取得失敗は fail-closed とし、成功風の `unknown` を残さない。
4. E2E check 名・件数は変更しない（255 維持）。
5. 全ゲート成功、負向 test（version 取得失敗）で失敗すること。

## Acceptance criteria

- 通常 run で全ゲート成功、`--dist` の manifest に 4 項目が記録される。

## Constraints

- `tools/verify-web-reader.py` / E2E check / Cargo.* は変更しない。
- 外部ダウンロード・ブラウザインストールは行わない。
- `master` へ push する。force push はしない。
