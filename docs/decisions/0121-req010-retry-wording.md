---
type: Decision
title: "ADR-0121: REQ-010 リトライ回数の明文化"
description: "REQ-010 の「最大3回再生成」を実装と一致する文言に明文化する決定を記録する。"
tags:
  - ADR
  - requirements
  - ai
generated:
  by: dsh
  at: "2026-11-18T07:35:00+09:00"
status: accepted
---

## 背景

`docs/requirements.md:132` は「rumdl 検査合格まで最大3回再生成」としていたが、
実装は `for attempt in 0..=max_retries`（既定 3）で
初期 1 回＋リトライ 3 回＝最大 4 回生成する。
ADR-0105 / 0120 で解釈差が未決定として残っていた。

## 決定

- REQ-010 の品質ゲートを次に変更する。
  - 生成回数は `1 + max_retries`。
  - 既定 3 なら合計最大 4 回、上限 10 指定時は合計最大 11 回。
  - `max_retries = 0` なら初期 1 回のみ（`attempts = 1`）。
  - `max_retries` は保存値を制限せず、service 境界で実行時に
    0..=10 へクランプする（ADR-0120）。
  - 最終不合格なら lint 状態フラグと差分を返し、ユーザー判断へ委ねる
    （ADR-0116）。
- 実装のループ境界は変更しない（コード変更なし）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 要件文言を実装に合わせる（採用） | 実装を変えずに解釈差を解消できる |
| ループを `0..max_retries` に変更 | 既存の既定挙動（4 回）が変わり回帰リスク |
| 解釈差を残す | 要件と実装が食い違い続ける |

## 影響

- REQ-010 の文言が ADR-0002 / 0105 / 0116 / 0120 と整合する。
- ADR-0002 の「再生成」は本決定の「リトライ」と同義と解釈する。
- ADR-0116 が引用する旧要件文言は、本 ADR の新文言に置き換えられる
  （決定内容自体は不変）。
- コード変更なし。既存の 200 テスト（native 200 + wasm 3）と E2E 228 項目は維持。

## 検証

- `markdownlint-cli2 docs/requirements.md` 0 issues。
- コード変更が無いためゲートは既存の緑を維持。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（req-wording-auditor の指摘）

- 上限 10 の客観的根拠（コスト実測等）と ADR-0120 の再評価。
- 保存値の正規化タイミング（load / merge / save）と範囲外値の扱い。
- `PipelineConfig` 直接生成経路はクランプされない（ADR-0120 から継続）。
- `max_retries = 0` の pipeline 回帰テスト未追加（ADR-0118 / 0120 から継続）。
- ~~環境変数 `OPENRUSTBOOKS_AI_MAX_RETRIES` の figment マッピング~~ ADR-0122 で解消済み。
- Web UI 未接続のため「ユーザー判断へ委ねる」は要件として維持されるが
  実装は未達（ADR-0116 から継続）。
