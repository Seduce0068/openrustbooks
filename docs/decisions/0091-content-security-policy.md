---
type: Decision
title: "ADR-0091: Content-Security-Policy の追加"
description: "Web 版に CSP meta を追加し、リソース読み込み先を制限する決定を記録する。"
tags:
  - ADR
  - web
  - security
generated:
  by: dsh
  at: "2026-11-17T23:50:00+09:00"
status: accepted
---

## 背景

`web/index.html` に CSP が無く、inline `<style>` と Google Fonts の
スタイルシートが無制限に読み込まれていた（todo.md B01 / E03 の一部）。

## 決定

`web/index.html` に次を追加する。

```text
default-src 'self';
script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
font-src 'self' https://fonts.gstatic.com;
img-src 'self' blob: data:;
connect-src 'self' blob: data:
```

- `wasm-unsafe-eval` は Wasm の JIT に必要。
- `'unsafe-inline'` は Trunk が生成する module script と既存の inline style に必要。
- `blob:` / `data:` は本文画像の Blob URL とその MIME 確認に必要。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 限定的な CSP meta（採用） | 既存機能を壊さず、許可先を明示できる |
| strict-dynamic + nonce | Trunk のビルド設定変更が必要で大規模 |
| CSP なし | 外部リソースの読み込み先が制限されない |

## 影響

- スクリプト・スタイル・画像・ネットワークの読み込み先が制限される。
- 外部フォント（Google Fonts）は明示的に許可したため維持される。
- E2E に CSP 存在のチェックを 1 件追加（合計 221 項目）。

## 検証

- 初回の CSP では `script-src` に `'unsafe-inline'` が無くアプリが起動しなかった。
  Trunk 生成の module script を許可して解消した。
- 次に `connect-src` に `blob:` / `data:` が無く、Blob URL の MIME 確認と
  SVG の相対参照テストが失敗した。許可して解消した。
- E2E 221 項目が 5 回連続で成功。
- 6 ゲート・trunk build・WASM パス検査が成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 外部 CSS の SRI（todo.md B01 の残り）。
- `script-src` から `'unsafe-inline'` を外すための nonce 対応。
- `IntersectionObserver` による画像の遅延読み込み（ADR-0034 から継続）。
EOF
