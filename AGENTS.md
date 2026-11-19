---
type: Playbook
title: "AGENTS.md for openrustbooks"
description: "openrustbooks 固有の制約とグローバル規約との差分。"
tags:
  - agents
  - openrustbooks
  - rust
generated:
  by: opencode
  at: "2026-10-31T00:27:37+09:00"
---

## プロジェクト概要

Rust 製の閲覧・編集アプリ。`core`（native + wasm）、`desktop`（Tauri）、
`web`（Leptos）のワークスペース構成。設計判断は `docs/decisions/` の
ADR に従い、`Cargo.lock` はアプリケーションのため追跡する。

## グローバル設定との関係

- 共通規約はグローバル `~/.agents/AGENTS.md`（正本）を継承する。
- 本ファイルは本プロジェクト固有の差分だけを記述する。グローバルと
  重複する規約は追加せず、差分がある場合にのみ上書きする。
- 本プロジェクト固有のタスクは本リポジトリの `todo.md` を正として管理する。
  プロジェクト外のグローバル TODO はここでは参照しない。
- 文書形式・索引・log・lint の運用は `docs/documentation-operations.md`
  （ADR-0234）を正とする。
- 本ファイルとグローバルが矛盾する場合は、本ファイルの記述を優先する。

## プロジェクト固有の制約

- `core` は native と wasm の両ターゲットでビルドできる状態を保つ。
- EPUB/ZIP の入力は fail-closed で扱い、パス脱出や外部スキームを拒否する
  （ADR-0004 以降）。
- `GOAL.md` は goal-loop の契約。変更する場合は契約と実装を同時に更新する。

## 検証

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked
cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked
```

CI は `Seduce0068/workflows/rust-gates.yml@v1` に委譲している。
