---
type: Decision
title: "ADR-0181: SVG サニタイズの共通化"
description: "Web 側の DomParser 手動サニタイズを廃止し、core の ammonia ベース実装へ一本化する決定を記録する。"
tags:
  - ADR
  - core
  - web
  - security
generated:
  by: dsh
  at: "2026-11-19T04:33:00+09:00"
status: accepted
---

## 背景

SVG は web/src/lib.rs の `sanitize_svg_bytes()`（DomParser + 手動属性削除）と
core/src/sanitize.rs の `sanitize_epub_html()`（ammonia 許可リスト）で
別基準のサニタイズが行われていた。片方の修正が他方へ反映されず drift する
リスクがあった（R6: PR-H06）。

## 決定

- core に `sanitize_svg_xml(svg: &str) -> Option<String>` を追加する。
  - 空入力／XML 処理命令の閉じ欠落／SVG ルート不在／出力空は
    fail-closed で `None`。
  - ammonia 4.2.5 Builder で安全 SVG タグ 25 種のみ許可。
  - `clean_content_tags` で script / style / foreignObject / set / animate* /
    discard / mpath / title / template / noscript / iframe / object / embed / a
    を内容ごと削除。
  - `tags` / `tag_attributes` と `clean_content_tags` は重複させない
    （ammonia 4.2.5 は重複時に assert panic: lib.rs:1924-1929）。
  - `generic_attributes` は `id` のみ、`tag_attributes` は明示上書き。
  - `url_schemes(HashSet::new())` + `UrlRelative::PassThrough` +
    `strip_comments(true)` + `link_rel(None)`。
  - `attribute_filter` で `href` / `src` に既存 `is_local_reference()` を適用し、
    `//host/path` や制御文字入りスキームも拒否する。
  - `viewBox` / `xmlns` / `xlink` は html5ever の local 名でホワイトリスト化。
  - `set_tag_attribute_value("svg", "xmlns", ...)` で namespace を強制。
  - `<?...?>` はパーサ依存を避け入力側で明示除去。
- core/src/wasm.rs に `sanitize_svg_wasm(bytes: &[u8]) -> Option<Vec<u8>>` を追加。
- web の `sanitize_svg_bytes()` を core API の薄いラッパーへ置き換え、
  旧 DomParser 手動実装（約 90 行）を削除。
- `svg_dimensions()` / `resolve_images()` の DomParser は本次対象外（据え置き）。
- E2E は既存 EVIL_SVG 11 項目で十分とし、249 件を維持。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| core 共通 ammonia（採用） | XHTML と同じ許可リスト方式で drift を防げる |
| web DomParser 維持 | 二重実装の drift リスクが残る |
| 新规依存追加 | 不要（ammonia 既存） |

## 影響

- ammonia は HTML パーサーを使うため、SVG の属性名やバイト列が
  DomParser(XML) 时代と変わり得る。E2E で描画と全悪意ケースを確認済み。
- `core/Cargo.toml` の ammonia 表記は `"4.2.4"` のまま
  （Cargo.lock は 4.2.5、セマンティック互換のため本次では変更しない）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- core unit test 4 件追加（EVIL_SVG 相当、空／不正入力、
  外部・ルート相対参照、フラグメント／camelCase 保持）。
- `cargo test --all-targets --locked`: core **216** + desktop 59 + web 2。
- wasm-test 10、audit 脆弱性 0 件。
- E2E **249 項目すべて成功**。
  - script / on* / foreignObject / javascript / SMIL / 外部スキーム /
    相対参照維持 / xml-stylesheet / スクリプト未実行 / width,height 維持。
- `./scripts/verify.sh` rc=0。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `core/Cargo.toml` の ammonia 表記を 4.2.5 へ更新するか。
- `svg_dimensions()` / `resolve_images()` の DomParser を
  共通 API へ移行するか。
- CSS `url()` / `<style>`、`feImage` / `<use>` の参照範囲の精緻化。
