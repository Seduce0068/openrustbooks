# Goal

エディタ Preview で Markdown を実際にレンダリングする（ADR-0161 の未決定）。

## Scope

### 背景

ADR-0161 で textarea と Preview を接続したが、Preview は
`sanitize_epub_html()` のみで `# Heading` が見出しにならない。
core は `pulldown-cmark 0.13` と `comrak 0.59` を宣言済みだが未使用。

### 成果物

- `core/Cargo.toml` / `Cargo.lock`: 依存整理（不要なら削除、必要なら feature 調整）
- `core/src/lib.rs` / 新規モジュール: Markdown → HTML 変換とサニタイズ
- `core/src/wasm.rs`: Wasm 公開 API
- `web/src/lib.rs`: Preview での利用
- `tools/verify-web-reader.py`: E2E 1 件追加、237 → 238
- `docs/decisions/0162-markdown-preview-rendering.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 既存依存のいずれか（推奨: comrak の featureless 設定）を使い、
   Markdown → HTML 変換を core に追加する。
2. 生成 HTML は `sanitize_epub_html()` を通してから `inner_html` に渡す
   （XSS fail-closed）。
3. Web の Preview が `# Heading` を `<h1>` として表示する。
4. E2E を 1 件追加（heading / リンク / script 除去など）、期待総数 237 → 238。
5. 既存 237 項目と全ゲートを壊さない。

## Acceptance criteria

- E2E 238 項目が成功する。
- 6 ゲート + audit + wasm-test が成功する。

## Constraints

- 新規依存は追加しない（既存の comrak / pulldown-cmark を整理して利用）。
- `unsafe` 禁止。
- `master` へ push する。force push はしない。
