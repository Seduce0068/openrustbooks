---
type: Decision
title: "ADR-0116: 最終 lint 不合格の扱い"
description: "AI pipeline の最終試行で lint 不合格でも Ok を維持し、呼び出し側が lint_passed を確認して差分を提示する方針を記録する。"
tags:
  - ADR
  - ai
  - lint
generated:
  by: dsh
  at: "2026-11-18T06:30:00+09:00"
status: accepted
---

## 背景

`execute_pipeline()` は `lint_result.passed || attempt == max_retries` で
ループを抜け、`passed=false` でも `Ok(PipelineExecutionResult)` を返す
（ADR-0111 の未決定）。`last_error` と `Err` 経路は最終試行では到達しない
デッドコードである。
REQ-010 は「品質ゲート: rumdl 検査合格まで最大3回再生成。
失敗時は差分提示しユーザー判断へ」を求める。

## 決定

- **既定は `Ok` ＋ `lint_passed=false` を維持する**（Err 化しない）。
- `lint_passed=false` は「生成物と差分を保持したまま、利用者の判断を促す」
  ステータスとして扱う。UI 接続時に次を必須条件とする。
  1. 4 コマンド（`generate` / `refine` / `to_table` / `front_matter`）の
     結果で `lint_passed` を確認する。
  2. `diff` と（可能なら）lint issues を提示する。
  3. 「このまま保存／再生成／破棄」を選ばせる（送信前同意 ADR-0104 とは
     別の、結果後の confirmation）。
- 単純な `Err` 化は採用しない。`CoreError::AiPipeline(String)` は
  diff / issues を保持せず、desktop の `map_err` も文字列化するため、
  「失敗時に差分提示」という要件を直接後退させる。
- オプトイン `strict_lint` も今回は追加しない
  （`warnings_as_errors` が未配線で strict 系フラグが既に死んでいるため、
    新フラグ前に整理する）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Ok 維持＋呼び出し側確認（採用） | diff が届く経路を活かし、要件「ユーザー判断へ」に整合 |
| 単純 Err 化 | diff/issues が消え、要件後退 |
| リッチなエラー型＋IPC 変更 | 正しいが範囲が大きい（将来の再設計課題） |
| オプトイン strict | 既存 strict 系未配線の整理が先 |

## 影響

- コード変更は無い（今回は調査と決定のみ）。
- 既存の 195 テスト、E2E 228 項目はそのまま。
- 真の不足は「呼び出し側が `lint_passed` を読んでいないこと」であり、
  UI 接続時に解消する。

## 検証

- 2 観点のエージェント調査（コードフローと UX / データ整合性）を実施。
- コード変更が無いためゲートは既存の緑を維持。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（将来の再設計課題）

- 構造化ペイロード（diff / issues / content / converged / fixes_applied /
  本文の出所）を保持するエラー型もしくはレスポンスへの移行。
- ~~`mod.rs` の到達不能な `last_error` / `Err` デッドコードの削除~~ ADR-0118 で削除済み。
- Web UI 未接続（AI ボタンはスタブ、wasm service は常に pass）。
- `conflict_cycle` 未保持、`WasmLintFixResult` スタブ、反復上限 10 の根拠。
- ~~`warnings_as_errors` の rumdl 側未配線~~ ADR-0123 で lint 判定に配線済み（`config_file` は ADR-0124 で未対応決定、`rule_config` は配線済み）。
- `refine` / `front_matter` のリトライ入力が元原稿で、修正後本文が
  AI 再生成に渡らない問題（ADR-0110 から継続）。
