---
type: Decision
title: "ADR-0087: Markdown 保存時の front matter 保全"
description: "Markdown 文書の保存時に front matter が失われるバグを修正する決定を記録する。"
tags:
  - ADR
  - bugfix
  - desktop
generated:
  by: dsh
  at: "2026-11-17T22:20:00+09:00"
status: accepted
---

## 背景

`desktop/src/commands/document.rs` の保存処理は `Document::as_text()` を使っていた。
`as_text()` は `DocumentContent::Markdown { content, .. }` の body のみを返すため、
`front_matter` がディスク書き込み時に失われていた（todo.md 独自 R2-arch-state#1）。

## 決定

- `Document::full_text()` を追加する。
  - Markdown: front matter を `serde_norway` でシリアライズし、
    `---\n<yaml>\n---\n<body>` の形で返す。
  - front matter が `None` の場合は body をそのまま返す。
  - Text は従来どおり本文を返す。
- 保存処理は `as_text()` の代わりに `full_text()` を使う。
- `as_text()` は編集用 API として維持し、挙動を変えない。
- core に `full_text_restores_front_matter` テストを追加する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| front matter を再シリアライズ（採用） | YAML パース済みのデータをそのまま活かせる |
| 元の文字列を保持 | 元の書式・コメントを保てるが、データ構造が別管理になる |
| `as_text()` を変更 | 編集用 API の意味が変わり、呼び出し側に影響する |

## 影響

- Desktop 版で Markdown を保存しても front matter が失われない。
- Web 版のリーダーには影響しない。
- 追加依存は無い（既存の `serde_norway` を使用）。

## 検証

- `cargo test -p openrustbooks-core --lib --locked full_text` が成功。
- `./scripts/verify.sh` を実行し、6 ゲート・trunk build・E2E 223 項目が
  すべて成功した。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- front matter の元の書式（コメント・順序・quoting）を保存するか。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
