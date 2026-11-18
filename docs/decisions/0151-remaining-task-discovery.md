---
type: Decision
title: "ADR-0151: 残タスク探索結果（R1）"
description: "5 観点のエージェント探索で発見した残タスクを集約し、次に実行するタスクを記録する。"
tags:
  - ADR
  - planning
generated:
  by: dsh
  at: "2026-11-18T19:45:00+09:00"
status: accepted
---

## 背景

継続ゴール「タスクが終わったら別のタスクを探して実行する」に従い、
`swarm-todo-discovery` の R1 考え方で 5 観点（静的シグナル / 機能ギャップ /
テストギャップ / 依存・保守 / UX・運用）のエージェントを並列起動した。

`todo.md` は別セッションの成果物のため更新せず、結果を本 ADR に記録する。

## 発見の集約（重複排除後）

### 高優先度・事実

- Wasm AI Markdown サービス／linter が固定成功値を返すスタブ
  （`core/src/wasm.rs:382-499`）。ただし Web UI から到達不能。
- Web エディタの Save / Open / AI / Lint ボタンが未配線
  （`web/src/lib.rs:204-244`、`docs/decisions/0127:18-29`）。
- About 画面が「License: MIT」「PDF/ZIP 対応」と誤表示
  （`web/src/lib.rs:170-175`）。実際は AGPL / EPUB のみ。
- OS 追従時にダーク切替ボタン文言が実表示と矛盾し得る
  （`web/src/lib.rs:1952-1957,3187-3188`）。
- localStorage 保存失敗が黙視される（例: `web/src/lib.rs:1241-1270`）。
- ~~Clear data の確認不足~~ ADR-0171 で confirm 追加済み（import は残存）。
- `verify.sh` の WASM パス検査が glob 非マッチ時に fail-open
  （旧 `scripts/verify.sh:71`）。

### 中優先度・事実／推論

- rumdl の `default-features = true` が CLI / LSP / jemalloc を持ち込み、
  serde_yaml 0.9.34+deprecated も入る（`core/Cargo.toml:63-70`、Cargo.lock）。
- `cargo audit` 未インストール時に黙ってスキップ（`scripts/verify.sh:41-45`）。
- 署名付き E2E の `set_sample_position()` が成否を返さない。
- 作者指定 `width` / `height` を自然寸法で上書きする挙動は
  ADR-0149 の未決定のまま。
- EPUB 内 CSS のオプトイン導入、Google Drive、PDF、フォルダ直接参照、
  月額上限／ローカル優先の enforcement などが未実装。
- i18n / エラー文言の平坦化、`prefers-reduced-motion` 未対応。

## 判断

- 今回は **fail-closed の検証ゲート**を優先し、
  `verify.sh` の WASM パス検査を修正する（ADR-0152）。
- 次候補は、 About 画面の誤表示（小さく確実な UX 修正）と
  localStorage 保存失敗の可視化、rumdl feature 絞り込み。
- Wasm AI / lint の実装は、ブラウザ側 AI 送信同意（ADR-0104）と
  秘密の扱いを先に設計する必要があるため後回し。

## 検証

- 5 エージェントすべてが完了し、JSON 形式で結果を返した。
- 親が根拠 path を抽样確認し、重複・推測を整理した。
- コード変更は本 ADR と ADR-0152 に限定。

## 未決定

- 発見項目の優先順位は親判断であり、ユーザー指定があれば変更する。
- `todo.md` は別セッションの成果物のため未更新。
