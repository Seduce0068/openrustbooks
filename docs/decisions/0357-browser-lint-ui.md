---
type: Decision
title: "ADR-0357: ブラウザ版の静的検査ボタンを Wasm lint へ接続"
description: "WasmMarkdownLinter をブラウザのエディタへ接続し、静的検査ボタンで問題件数を表示する決定。"
tags:
  - ADR
  - web
  - wasm
  - lint
generated:
  by: dsh (lead)
  at: "2026-11-28T04:42:11+09:00"
status: accepted
---

## 背景

ADR-0160 で featureless rumdl の実 lint が wasm に配線済みだったが、web 側は未参照で
「静的検査」ボタンは disabled のままだった（FG-01）。

## 決定

- ボタンの disabled を外し、クリック時に `WasmMarkdownLinter::new(JsValue::NULL).lint(text)`
  を呼び、問題件数を固定日本語の status へ表示する。
- lint 設定は Web UI から渡さず既定（`LintConfig::default()`）を使う。
- エラー内容・本文・パスは status へ出さず、失敗時は固定文言のみ表示する。
- AI 生成 / 推敲 / 書式選択は従来どおり desktop only で disabled を維持する。

## 検証（Runtime Proof）

- `cargo check --all-targets --locked`: rc=0。
- `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`: rc=0。
- `./scripts/verify.sh`: 全ゲート成功（E2E 273 passed / 0 failed、WDIO gate rc=0）。
- 既存 E2E の toolbar チェックを 3 つの desktop only 操作と lint 有効に合わせて更新し、
  さらに lint ボタンを実クリックして「静的検査: N 件」の表示を確認（件数 273 不変）。

## 結果

- ブラウザ版でも rumdl 品質ゲートを利用可能になった。
- ADR-0160 の「JS 側の呼び出し経路なし」は失効し、本 ADR が追跡先となる。
  ADR-0232 の「Lint ボタン disabled」も当時点の記録として追記済み。

## 未決定

- lint 設定（LintConfig）の編集 UI と auto_fix のブラウザ側公開。
