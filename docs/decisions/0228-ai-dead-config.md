---
type: Decision
title: "ADR-0228: AI 死設定の設計受容"
description: "月間上限の未配線と timeout の Ollama 限定適用を設定画面へ明示し、実装を見送る決定を記録する。"
tags:
  - ADR
  - ai
  - config
generated:
  by: dsh
  at: "2026-11-19T16:40:00+09:00"
status: accepted
---

## 背景

R1-ai#2,#5 / R2-ai#3,#4 として「死設定」が指摘されていた（R6）。
監査の結果、以下の状況が判明した:

| 設定 | 状態 |
| --- | --- |
| `max_retries` | `clamp_max_retries()` 0..=10 を経てリトライループへ配線済み |
| `timeout_secs` | Ollama の `reqwest::Client::timeout()` にのみ配線（OpenAI 未配線） |
| `streaming` | 未配線を ADR-0216 で設計受容、UI は「未配線」表示 |
| `check_updates` | 未配線を ADR-0222 で設計受容、UI は「未配線」表示 |
| `max_tokens_per_month` | 設定値が DTO と「月間上限: 値」表示に現れるだけで、集計／保存／超過判定は皆無 |

`max_tokens_per_month` は利用者に「上限が働いている」と誤認させるため、
ADR-0216 / 0217 / 0222 と同型の正直化が必要だった。

## 決定

- 設定画面の AI 情報を次の固定文言へ変更する:
  - `月間上限: 未配線`
  - `タイムアウト: {n}s（Ollama のみ）`
- `max_tokens_per_month` の参照が消えたため、
  Web ローカル DTO `SettingsAiView` の同フィールドを削除する
  （serde の未知フィールド無視により IPC ペイロードのパースは不変）。
- core の `AppConfig` / config.toml / desktop DTO は変更しない。
- 月間上限の実装（使用量ストアと月次境界）は別タスクへ分離する。
- `max_retries` は配線済みとして完了、`streaming` / `check_updates` は既存 ADR の設計受容を維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設計受容＋表示の正直化（採用） | 誤認を防ぎつつ設定互換を壊さない |
| 月間上限を今すぐ実装 | 月定義・計測対象・保存先が未定義で誤課金表示のリスク |
| フィールドを削除 | config.toml と IPC 互換を壊す |

## 影響

- 利用者は「月間上限: 未配線」「タイムアウト: Ns（Ollama のみ）」を見る。
- OpenAI 側の timeout 未配線も表示上明示される。
- E2E は文言に依存せず、期待総数 254 を維持する。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo check --all-targets --locked`、
  `cargo test --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check: すべて成功。
- `./scripts/verify.sh`: 全ゲート成功、E2E **254 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 月の定義とタイムゾーン（カレンダ月／請求サイクル、ローカル／UTC）。
- 計測対象（prompt + completion の実トークン vs リクエスト数、
  `max_tokens_per_request` との混同を避ける）。
- lint リトライの消費を月上限へ算入するか（ADR-0116 を継続）。
- 使用量の保存先・排他・破損時の挙動（native ProjectDirs vs wasm localStorage）。
- wasm 版で同一上限を適用するか（ADR-0011 / 0211 を継続）。
- 超過時の挙動（fail-closed／警告／フォールバック）と ADR-0104 の同意契約。
- OpenAI 側 timeout を配線するか（現状は未設定のまま）。
