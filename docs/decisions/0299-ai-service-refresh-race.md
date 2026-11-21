---
type: Decision
title: "ADR-0299: refresh_ai_service の revision 再確認"
description: "AI service 生成中に config が更新されても古い service で上書きしない決定。"
tags:
  - ADR
  - desktop
  - concurrency
generated:
  by: dsh
  at: "2026-11-22T06:26:00+09:00"
status: accepted
---

## 背景

refresh_ai_service は disable → config 取得 → prepare_ai_service().await → 代入の順で動き、
await 中に commit_config が新 config / revision / service を確定すると、
古い config から生成した service が後から上書きされた。

## 決定

- 開始時に `config_with_revision()` で config と revision を同一時点で取得する。
- 生成中は旧 service を維持し、即時 disable しない。
- await 完了後、config write guard を保持したまま revision を再確認する。
- revision が変化していれば生成結果を捨て、commit 側の config / service を守る。
- revision が不変なら service を置換し、生成失敗時は fail-closed で None にする。
- config write guard を使うのは、確認と置換の間の TOCTOU を塞ぐため。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| write guard で再確認 + 置換（採用） | CAS と service の不可分性を保てる |
| read guard で再確認 | 確認後に commit が割り込むため却下 |
| init 完了まで config 更新を待つ | 起動時のUXを阻害するため見送り |

## 検証（Runtime Proof）

- 競合再現 test 3 件（外部 AI API なし）:
  - 古い service が上書きされない。
  - 生成失敗時も commit 済み service を守る。
  - revision 不変なら通常通り置換される。
- desktop test: **124 passed / 0 failed**。
- `cargo fmt --all -- --check`: rc=0。
- `cargo clippy --all-targets --locked -- -D warnings`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功、E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- config write guard を取る設計を共通の CAS helper へ集約するか。
- init_ai_service の spawn を await に変えるか。
