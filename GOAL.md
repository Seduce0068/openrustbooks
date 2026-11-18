# Goal

Google Fonts への外部送信を削除する（R6 PR-M07）。

## Scope

### 背景

`web/index.html` は Google Fonts の preconnect 2 件と stylesheet を読み込み、
CSP も `https://fonts.googleapis.com` / `https://fonts.gstatic.com` を許可する。
preconnect は CSP の対象外で、ページ読み込み時に IP と TLS 情報が外部へ渡る。

### 成果物

- `web/index.html`: 変更（外部 link 削除、CSP からフォントホスト削除、
  システムフォントスタックへ変更）
- `docs/decisions/0191-no-external-fonts.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `preconnect` 2 件と Google Fonts stylesheet の link を削除する。
2. CSP の `style-src` / `font-src` から external host を削除する。
3. `--font-mono` / `--font-sans` をローカルで利用可能なフォントへ変更する。
   - mono: `ui-monospace, 'Cascadia Code', 'Source Code Pro', Consolas, monospace`
   - sans: `system-ui, -apple-system, 'Segoe UI', 'Noto Sans JP', sans-serif`
4. E2E 249 項目を壊さない（trunk build で `web/dist` が更新されること）。
5. 全ゲート成功。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目成功。

## Constraints

- 新規依存・同梱フォントファイルを追加しない（本次は外部送信の削除のみ）。
- EPUB 本文の CSS 方針（ADR-0115）は変更しない。
- `master` へ push する。force push はしない。
