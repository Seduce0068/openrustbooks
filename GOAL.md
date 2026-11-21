# Goal

Markdown front matter の往復で元バイト列を保全する（R6 レビュー / R2-ER01）。

## Scope

### 背景

extract_front_matter は lines()/join("\n") で CRLF を LF に変換し、
full_text は serde_norway で YAML を再シリアライズするため、
コメント・クォート・キー順・数値表記・末尾改行が変更され得る。
編集していない Markdown を開いて保存するだけで差分ノイズが発生する。

### 成果物

- `core/src/document.rs`: 変更（元 front matter テキスト保持と再利用）
- `desktop/src/commands/document.rs`: 変更（本番 open 経路を preserving 版へ接続）
- `docs/decisions/0297-front-matter-byte-preservation.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. front matter を構造化値に加えて元テキストも保持する。
2. full_text は body が未変更なら元の front matter 区切り・改行スタイル・
   末尾改行をそのまま返す。
3. body が変更されても front matter は元テキストを保持する。
4. 既存 API 互換を壊さない（必要なら内部フィールドのみ変更）。
5. 単体 test: CRLF / 末尾改行 / YAML コメント・キー順を保持。
6. verify.sh 全ゲート成功。

## Constraints

- 外部サービス / 秘密情報は扱わない。
- E2E check 名・件数を変更しない。
- `master` へ push する。force push はしない。

## 完了記録

- front matter を raw 文本で保持し、無編集 open→save のバイト列を保全。
- desktop 本番 open 経路を preserving 版へ接続、往復 test を追加。
- core 327 / desktop 115 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 構造化値更新 / 旧データ移行 / save_document body 契約は ADR-0297 の未決定。
