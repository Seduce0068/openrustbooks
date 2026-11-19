# Goal

利用可能なブラウザで Web E2E 255 件を実機実行し、結果と環境ブロッカーを記録する（R6 B04-B06 / C01）。

## Scope

### 背景

`tools/verify-web-reader.py` は CDP を用い、
ブラウザ実行ファイルを `--chrome` で差し替えられる。
環境には Chrome / Edge / Firefox が存在するが、
E2E の CDP 依存により Chromium 系のみ同一 check セットを実行可能。
Brave は未インストール、Safari は Linux で起動不可。

### 成果物

- `docs/decisions/0265-cross-browser-e2e.md`: 追加
- `todo.md`: B04-B06 / C01 を「Chromium 系実機検証完了・他ブラウザは環境ブロッカー」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 現行ソースから新規 dist をビルドする。
2. Chrome と Edge で同一 255 check を実行し、255/0 を確認する。
3. Firefox / Brave / Safari の結果は pass とせず、
   プロトコル不一致 / 未インストール / OS 制約として記録する。
4. E2E check 名・件数・コードは変更しない（255 維持）。
5. markdownlint 0 issues、verify.sh（255）成功。

## Acceptance criteria

- Chrome / Edge の E2E がそれぞれ 255 passed / 0 failed。

## Constraints

- Brave / Safari のインストールや外部ダウンロードは行わない。
- E2E スクリプト / check 名 / 件数は変更しない。
- `master` へ push する。force push はしない。
