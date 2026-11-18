# Goal

SVG サニタイズを web の DomParser 実装から core の共通実装へ移行する（R6 PR-H06）。

## Scope

### 背景

SVG は web/src/lib.rs の `sanitize_svg_bytes()`（DomParser + 手動属性削除）と、
core/src/sanitize.rs の `sanitize_epub_html()`（ammonia 許可リスト）で
別基準のサニタイズが行われている。片方の修正が他方へ反映されず drift する。

### 成果物

- `core/src/sanitize.rs`: 変更（`sanitize_svg_xml()` 追加）
- `core/src/wasm.rs`: 変更（wasm API 追加）
- `web/src/lib.rs`: 変更（共通 API を利用、旧ローカル実装削除）
- `tools/verify-web-reader.py`: 変更（必要なら E2E 1 件、249 → 250）
- `docs/decisions/0181-svg-sanitizer-unification.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. core に SVG 専用の fail-closed サニタイザを追加し、
   既存 web 実装と少なくとも同等の脅威を防ぐ:
   - script / foreignObject / SMIL 除去
   - on* 属性除去
   - javascript / http / https / data / blob / file スキーム除去
   - XML 処理命令除去
2. web の `sanitize_svg_bytes()` を共通 API の薄いラッパーへ置き換える。
3. 既存 E2E の SVG 悪意ケース（EVIL_SVG 等）を壊さない。
4. 既存 249 項目と全ゲートを壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 249 項目（必要なら 250）成功。

## Constraints

- 新規依存を追加しない（ammonia 既存を利用）。
- `unsafe` 禁止。
- XHTML 用 `sanitize_epub_html()` の契約は維持する。
- `master` へ push する。force push はしない。
