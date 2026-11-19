---
type: Decision
title: "ADR-0266: 検証済み配布物へのブラウザ情報記録"
description: "verify.sh の manifest に E2E のブラウザ実行ファイル・解決パス・バージョン・ポートを記録する決定。"
tags:
  - ADR
  - e2e
  - verified-build
generated:
  by: dsh
  at: "2026-11-20T04:01:00+09:00"
status: accepted
---

## 背景

ADR-0265 で Chrome / Edge の E2E 255/0 を実機確認したが、
`scripts/verify.sh --dist` の manifest は e2e expected / result のみで、
どのブラウザ・ポートで検証したかが残らなかった。

## 決定

- `scripts/verify.sh` で E2E の設定を変数へ集約し、明示引数で渡す:
  - `E2E_BROWSER`（既定 `google-chrome`）
  - `E2E_STATIC_PORT`（既定 `8099`）
  - `E2E_CDP_PORT`（既定 `9222`）
- manifest に 5 項目を記録する:
  - `browser executable`
  - `browser path`（`command -v` の解決結果）
  - `browser version`
  - `static port`
  - `cdp port`
- ブラウザ version は `--version` の終了コードを検査し、
  空 / 空白のみ / 大小文字無視の `unknown` / 制御文字を拒否する。
- ブラウザ名・パス・ポートにも制御文字チェックを適用し、
  port は `^[0-9]+$` を要求する（manifest 行注入の防止）。
- E2E check 名・件数（255）・`docs/e2e-traceability.md` は変更しない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| manifest へブラウザ情報を記録（採用） | ADR-0265 の実機証跡と配布物完全性を接続できる |
| 外部ログのみ | 配布物から検証環境を再現できない |
| 既定値のまま暗黙利用 | 引数と manifest がずれ、証跡が曖昧になる |

## 影響

- 引数なし verify.sh でもブラウザ version 取得が必須になり、
  未インストール環境では fail-closed で停止する。
- manifest スキーマに 5 項目が増えるが、
  SHA256 検証（`grep -E '^[0-9a-f]{64} ' | sha256sum -c -`）は壊れない。
- CI は verify.sh の E2E を実行しないため、/browser 情報はローカル検証の証跡。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck`: OK。
- 負向 test 6 パターン（存在しないブラウザ / 改行 version / `UNKNOWN` /
  空白 version / 改行 port / 不正 port `abc`）: すべて rc=1。
- 通常 `./scripts/verify.sh`: 全ゲート成功、E2E **255 passed / 0 failed**。
- `./scripts/verify.sh --dist /tmp/orb-metadata-dist`: rc=0、
  manifest に 5 項目と `e2e expected: 255` /
  `e2e result: passed: 255, failed: 0` を確認。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- env 変数によるブラウザ / ポート上書きを正式 CLI インターフェースとするか
  （現状は既定値維持のための内部変数として導入）。
- CI で verify.sh の E2E を有効化した際のブラウザ選択と
  manifest スキーマの下流互換性。
- browser version の末尾空白を正規化するか
  （現状は `--version` 出力をそのまま記録）。
