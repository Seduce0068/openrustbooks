---
type: Decision
title: "ADR-0361: AI generate / refine の desktop UI 接続"
description: "ADR-0127 段階 4 として generate_markdown / refine_markdown を desktop UI のエディタへ接続し、結果確認と redaction 契約を維持する決定。"
tags:
  - ADR
  - ai
  - web
generated:
  by: dsh (lead)
  at: "2026-11-28T18:24:52+09:00"
status: accepted
---

## 背景

desktop には AI command が登録済みだが web UI から到達不能だった。段階 3 の lint は
ADR-0357 で完了済みで、次は送信を伴う generate / refine の接続である。

## 決定

- 送信前の同意は既存どおり command 側の `confirm_send` が担い、JS から `consent` は受け取らない。
- ブラウザビルドでは AI ボタンと入力を disabled に保ち、title / aria の desktop only 文言を維持する。
- desktop ランタイム検出は `__TAURI__.core.invoke` の存在で行い、busy 中は再呼び出しを弾く。
- 結果は Deserialize mirror へ変換し、`lint_passed=false` のときだけ確認ダイアログを開く。
- ダイアログの選択は「このまま保存」「再生成」「破棄」の 3 つ。再生成は必ず command を再呼びし、再度同意を要求する。
- status と DOM 属性には固定日本語文言と行数だけを出し、本文全文・path・秘密値・code は出さない。
- 差分表示は行数のみとし、内容の抜粋も出力しない（redaction を優先）。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check` / `cargo check --all-targets --locked` / `cargo test --all-targets --locked`（37 passed）/
  `cargo clippy --all-targets --locked -- -D warnings` / wasm32 check: rc=0。
- `bash scripts/verify.sh`: rc=0（E2E 273 passed / 0 failed、WDIO 2 spec 成功）。
- 外部 AI API への送信 test は行っていない。

## 未決定

- lint issues のホワイトリスト表示（現在は `lint_passed` と行数のみ）。
- 実プロバイダへの送信とネイティブ確認ダイアログの実機検証（送信はユーザー確認が必要）。

## 参照

- ADR-0127: editor UI 接続の段階方針
- ADR-0104 / 0116 / 0343: 送信同意・結果確認・redaction
- ADR-0357: 段階 3 の lint UI 接続
