# Goal

SVG サニタイズを web の DomParser 実装から core の共通実装へ移行する（R6 PR-H06、ADR-0181）。

## Scope

### 成果物

- `core/src/sanitize.rs`: sanitize_svg_xml、unit 4 件
- `core/src/wasm.rs`: sanitize_svg_wasm
- `web/src/lib.rs`: 薄いラッパーへ置換、旧実装削除
- `docs/decisions/0181-svg-sanitizer-unification.md`: 追加
- `todo.md`: PR-H06 を完了へ移動

### 決定

- ammonia 4.2.5 ベースの SVG 専用 Builder。
- clean_content_tags と tags の非重複、camelCase / local 名登録。
- url_schemes(空) + attribute_filter で //host とスキーム入口を拒否。
- E2E 249 件維持。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
