# Goal

Reader の CoverImage DTO から未使用フィールドを削除する（R6 TD-L04、ADR-0203）。

## Scope

### 成果物

- `web/src/lib.rs`: CoverImage を href / bytes のみへ縮小、allow(dead_code) 削除
- `docs/decisions/0203-cover-image-dto.md`: 追加
- `todo.md`: TD-L04 を完了へ移動

### 決定

- core の WasmEpubCoverImage は変更せず、serde の未知フィールド無視に依存。
- Blob URL 生成経路は不変、MIME 推定は href 拡張子のまま。

## Acceptance criteria

- E2E 251 項目、全ゲート成功（検証済み）。
