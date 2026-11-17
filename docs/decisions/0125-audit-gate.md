---
type: Decision
title: "ADR-0125: 脆弱性監査のローカルゲート"
description: "cargo audit をローカル検証スクリプトに追加し、既知脆弱性の回帰を防ぐ決定を記録する。"
tags:
  - ADR
  - security
  - tooling
generated:
  by: dsh
  at: "2026-11-18T08:50:00+09:00"
status: accepted
---

## 背景

`cargo audit` で脆弱性 0 件を確認済みだが（ADR-0086）、
`scripts/verify.sh` にはゲートが無く、CI も billing issue で実行できない。

## 決定

- `scripts/verify.sh` の wasm-test ゲート直後に次を追加する。

```bash
if command -v cargo-audit >/dev/null 2>&1; then
    run audit cargo audit --no-fetch
else
    echo "WARNING: cargo-audit not installed; skipping audit gate"
fi
```

- `--no-fetch` を採用（オフライン／キャッシュ優先、再現性重視）。
- `--deny warnings` は使わない（既存の unmaintained 警告 10 件は baseline、
  ADR-0086:49）。脆弱性 1 件以上は常に rc!=0 でゲート失敗。
- advisory DB が無い／取得失敗の場合もスキップせずゲート失敗（安全側）。
- `cargo-audit` 未インストール時は警告を出してスキップ
  （`|| true` で rc を握り潰さない。脆弱性検出の fail-open を避ける）。
- 既存の 6 ゲート＋wasm-test は維持。CI ワークフローは当初変更しない方針だったが、ADR-0126 で `audit: true` を追加して上書きした。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| ローカルゲート＋`--no-fetch`（採用） | 再現性が高く、依存更新時の回帰を防げる |
| fetch あり | 鮮度は高いがネットワーク障害で失敗し得る |
| `--deny warnings` | 既存 baseline 警告 10 件で即失敗する |

## 影響

- `./scripts/verify.sh` が脆弱性スキャンを含むようになった。
- 未インストール環境では監査がスキップされる（警告表示）。
- `--no-fetch` は advisory DB の stale 判定を無効化するため、
  鮮度は運用責任（定期的な `cargo audit update-db`）に委ねる。

## 検証

- `bash -n` / `shellcheck` / `shfmt -d` すべて通過。
- `./scripts/verify.sh` rc=0:
  - audit: 821 crate をスキャン、脆弱性 0 件、警告 10 件（baseline）。
  - E2E 228 項目すべて成功。
- 未インストールブランチも PATH を隠して確認（警告表示＋続行）。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（audit-gate-auditor の指摘）

- `cargo audit` の rc=1 は「脆弱性検出」と「advisory DB エラー」で共通。
  `run()` は両方を失敗扱いにしており支障はないが、ログの読み分けが必要。
- CI（共通ワークフロー `rust-gates.yml@v1`）への audit ゲート追加。
  `extra_gates` 経由で渡せるかは未確認（推測）。
- advisory DB の更新頻度と運用ルール。
- 警告 10 件（unmaintained / unsound / yanked）の対応。
