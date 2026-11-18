---
type: Decision
title: "ADR-0191: 外部フォントの削除"
description: "Google Fonts の preconnect / stylesheet を廃止し、CSP とフォントスタックをローカル完結にする決定を記録する。"
tags:
  - ADR
  - web
  - privacy
generated:
  by: dsh
  at: "2026-11-19T06:57:00+09:00"
status: accepted
---

## 背景

`web/index.html` は Google Fonts の `preconnect` 2 件と stylesheet を読み込み、
CSP も `https://fonts.googleapis.com` / `https://fonts.gstatic.com` を許可していた。
`preconnect` は CSP の対象外で、ページ読み込み時に IP と TLS 情報が外部へ渡る
（R6 PR-M07）。

modern-web-guidance の `privacy` / `secure-external-subresources` では
データ最小化と第三者リソースの削減を推奨する。
電子書籍の本文を扱うアプリでは、不要な第三者送信を避けることが安全側である。

## 決定

- Google Fonts の `preconnect` 2 件と stylesheet の `<link>` を削除する。
- CSP を `style-src 'self' 'unsafe-inline'; font-src 'self';` に変更する
  （他ディレクティブは維持）。
- CSS 変数をシステムフォントスタックへ変更する:
  - `--font-mono`: `ui-monospace, 'Cascadia Code', 'Source Code Pro', Consolas, monospace`
  - `--font-sans`: `system-ui, -apple-system, 'Segoe UI', 'Noto Sans JP', sans-serif`
- `scripts/verify.sh` に静的ゲート `no-external-font-refs` を追加し、
  `web/index.html` に `fonts.googleapis` / `fonts.gstatic` / `preconnect`
  が含まれたら fail-closed でゲート失敗とする。
- `web/dist` は再ビルドしたが、git 追跡対象外のため差分には含まない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 外部フォントを全削除（採用） | 第三者送信と SRI 管理が不要になる |
| SRI / referrerpolicy を付けて維持 | 依然として IP / 接続情報が外部へ渡る |
| 同梱フォントを追加 | 本次は送信削除を優先（未決定） |

## 影響

- ADR-0091 / 0115 / 0130 の「Google Fonts 許可」記述は本 ADR で上書きされる
  （旧 ADR は履歴として残す）。
- 日本語表示はローカルの `system-ui` / `Noto Sans JP` 等の
  fallback に依存する。CJK フォント未導入の Linux では表示が劣化し得る
  （**推測**、実機 5 ブラウザ未検証）。
- E2E の文字サイズ・画像寸法・CLS チェックは rem / 画像属性 / 画像ロード
  ベースのため、フォント変更の影響は受けにくい（**推測**）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `grep -n 'fonts.googleapis\|fonts.gstatic\|preconnect' web/index.html web/dist/index.html`
  は両方 0 件。
- `bash -n` / shellcheck / shfmt -d scripts/verify.sh: 成功。
- verify.sh に `no-external-font-refs` ゲートを追加。
  ネガティブテスト（`preconnect` 入りの偽 index.html）で rc=1 を確認。
- `./scripts/verify.sh` rc=0、E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 同梱フォント（例: Noto Sans JP のサブセット）を追加するか。
- CJK フォント未導入環境の受容方針。
- 実機 5 ブラウザでのフォント表示確認。
- 外部 host 再追加を検知するゲートを `web/dist` にも拡張するか。
