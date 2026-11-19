---
type: Decision
title: "ADR-0215: Dependabot による依存更新の自動検出"
description: "Cargo と GitHub Actions の patch / minor 更新を週次で自動検出し、major は手動判断に残す決定を記録する。"
tags:
  - ADR
  - deps
  - ci
generated:
  by: dsh
  at: "2026-11-19T12:45:25+09:00"
status: accepted
---

## 背景

`.github/dependabot.yml` が無く、Cargo 依存と GitHub Actions の更新を手動で
検出していた（R6 TD-M14）。公式の [Dependabot options reference][options] では
`version: 2`、`updates[].package-ecosystem`、`directory`、`schedule.interval` が
必須で、`cargo` と `github-actions` がサポート対象と明記されている。

## 決定

- `.github/dependabot.yml` を追加し、`version: 2` で `cargo` と
  `github-actions` をルート `/` から週次（月曜 09:00 Asia/Tokyo）でチェックする。
- 両エコシステムとも `open-pull-requests-limit: 5` を設定する。
- cargo は `allow` で `dependency-type: all` かつ
  `version-update:semver-patch` / `version-update:semver-minor` のみを許可し、
  major 更新は手動判断に残す。`versioning-strategy: increase-if-necessary` を明示する。
- cargo はグループ `patch-and-minor` で patch / minor を 1 PR へ集約し、
  github-actions はグループ `all-actions` で全更新を 1 PR へ集約する。
- 秘密情報・registries は設定しない（GitHub 標準機能のみ、新規依存なし）。

## 根拠

- [Dependabot options reference][options] — 必須キー（`version` /
  `package-ecosystem` / `directory` / `schedule.interval`）、サポート対象
  エコシステム、`allow` / `groups` / `versioning-strategy` /
  `open-pull-requests-limit` の公式仕様。
- patch / minor に絞ることで破壊的変更（major）の判断を人手に残し、
  `Cargo.lock` 追跡と `--locked` ゲート（ADR-0214）との整合を保つ。
- グループ化により PR の洪水を防ぎ、レビュー負荷を抑える。
- `allow` は version updates にのみ適用され、Dependabot security updates の
  既定動作は変えない。セキュリティ更新は従来どおり継続する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| Dependabot（採用） | GitHub 標準機能で追加依存・秘密情報なし、cargo / github-actions を直接サポート |
| Renovate | 利点はあるが、新規サービスの導入とトークン管理が必要なため却下 |
| 手動検出の継続 | 更新検知の遅れと見落としが残るため却下 |

## 影響

- 月曜 09:00（Asia/Tokyo）に更新 PR が作成され、cargo は patch / minor が 1 PR、
  github-actions は全更新が 1 PR にまとまる。
- major 更新は手動で検出・判断する。セキュリティ更新は Dependabot の既定で継続する。
- 共通ワークフロー `rust-gates.yml@v1` と `scripts/verify.sh` は変更しない。
- 新規依存・秘密情報・registries は追加しない。

## 検証（Runtime Proof）

- `python3 -c 'import yaml; yaml.safe_load(...)'` で YAML 構文が OK。
- `markdownlint-cli2 --config /tmp/mdlint.jsonc` で全対象 0 issues。
- 既存ゲート（E2E 253）を変更しない。

## 未決定・注意点

- リポジトリの billing issue により GitHub Actions（CI）が実行できないため、
  Dependabot PR のゲート結果はローカル検証で代替する（ADR-0213 / 0214 と同様）。
- CI が走らない間は、PR の自動マージやゲート成功を前提にした運用を採らない。
- グループ化した PR に major が混ざる場合の切り分けと、major 手動更新の運用は未決定。
- cargo の 0.x クレートにおける minor 更新の扱い（semver の慣例に依存）は未検証。
- Dependabot security updates と `allow` 指定の相互作用は実運用で確認する。

## 参照

- [Dependabot options reference][options]
- [ADR-0214: MSRV 1.95 の宣言](0214-msrv.md)

[options]: https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference
