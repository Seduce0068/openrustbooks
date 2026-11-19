# Goal

現行 WASM バンドルのサイズを実測し、履歴 ADR の未決定を解消する（R6 バンドルサイズ計測）。

## Scope

### 背景

ADR-0243 / 0246 / 0256 / 0259 / 0260 は
「バンドルサイズ before / after 実測」を未決定として残していた。
`web/dist` の既存生成物は古いため、現行ソースを
`trunk build --offline --locked` で再ビルドし、
WASM / JS / 総量のベースラインを実測して記録する。

### 成果物

- `docs/decisions/0267-bundle-size-baseline.md`: 追加
- `docs/decisions/0243` / `0246` / `0256` / `0259` / `0260`: 変更（未決定の参照先を追記）
- `todo.md`: バンドルサイズ実測を完了へ反映
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 現行ソースから `--offline --locked` で dist を生成する。
2. WASM / JS / HTML / 総バイト数と SHA-256 を実測する。
3. 過去の before 値は存在しないため「比較不能」と明記し、
   今回を今後の before とする。
4. E2E check 名・件数・コードは変更しない（255 維持）。
5. markdownlint 0 issues、verify.sh（255）成功。

## Acceptance criteria

- 実測値が ADR に記録され、全ゲート成功。

## Constraints

- `web/dist`（git 対象外）は上書きしない（一時 dist を使う）。
- 外部ダウンロード・依存追加は行わない。
- `master` へ push する。force push はしない。
