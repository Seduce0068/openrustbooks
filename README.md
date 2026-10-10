---
type: Reference
title: "openrustbooks"
description: "Rustを活用する、ブラウザでも使えるリーダー兼テキストエディタ。"
tags:
  - Rust
  - Reader
  - Editor
status: planning
---

Rustを活用して、本や文書を読むためのリーダーと、
テキスト系ファイルを編集するエディタを作るプロジェクトです。

現在は構想・要件整理の段階です。
以下は目標と設計方針であり、実装済み機能の一覧ではありません。

## 目指すこと

- EPUB・ZIP・PDF・TXT・Markdownを扱う。
- TXT・Markdownなどのテキスト系ファイルを編集する。
- デスクトップアプリとブラウザから利用する。
- フォルダを簡単に開き、文書を扱えるようにする。
- Google Driveなどと連携し、フォルダやアーカイブとして保存する。
- Markdown Preview Enhanced、markdownlint、
  Markdown All in Oneに相当する機能を利用できるようにする。

## 決定済みの方針

Rustを積極的に活用します。
デスクトップ版はTauriの標準的なOS WebView方式（wry）を採用します。

Leptos・Yew・axumや個別ライブラリの採用、
Tauriのバージョンは未決定です。

## ドキュメント

- [要件と未決定事項](docs/requirements.md)
- [構成案と開発順序の案](docs/architecture.md)
- [ADR-0001: OS WebView方式の採用](docs/decisions/0001-use-system-webview.md)

利用者が求めた要件、合意済みの決定、設計上の提案を区別して記録しています。

## ライセンス

[LICENSE](LICENSE)を参照してください。
