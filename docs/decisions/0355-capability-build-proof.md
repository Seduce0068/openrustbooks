---
type: Decision
title: "ADR-0355: capability 最小権限の実ビルド完了証跡"
description: "既定ビルドと debug+wdio ビルドの capability 混入差分を実測し、A05 前提 2 の実ビルド証跡を確定する決定。"
tags:
  - ADR
  - capability
  - desktop
  - tauri
generated:
  by: dsh (lead)
  at: "2026-11-28T02:54:55+09:00"
status: accepted
---

## 背景

A05 前提 2（capability 最小権限）は、静的スコープゲート（ADR-0278）と wdio capability JSON
ゲート（ADR-0348）が verify.sh / CI へ配線済みで、負向 test 11/11 も記録されている。
しかし「静的ゲート成功」は実効権限の証明ではなく、実ビルド成果物の capability 設定が
未確認として残っていた（todo.md / ADR-0274 / ADR-0332）。

## 決定

- 本番（既定 feature）ビルドと debug+wdio ビルドを実際に生成し、成果物バイナリの
  capability 文字列を比較して証跡とする。生成設定の確認は ADR-0348 の静的ゲートに委譲する。
- 実 WebView の許可外 command / plugin 拒否経路（WDIO spec の neg 経路）は E2E 期待件数を
  変更する必要があるため今回の完了条件に含めず、未決定として ADR-0332 に残す。
- 静的ゲートは source inspection であり runtime ACL ではない旨を、完了記録にも明記する。

## 検証（Runtime Proof）

- `bash scripts/check-capability-scope.sh`: rc=0（4 command に限定、source inspection のみ）。
- `bash scripts/check-tauri-plugin-config.sh`: rc=0（plugins / scope / allowedPaths 不在、wdio は test 用ファイルに閉じる）。
- `cargo build -p openrustbooks-desktop --locked`: rc=0。
- `cargo build -p openrustbooks-desktop --features wdio --locked`: rc=0。
- debug+wdio ビルドの `strings target/debug/openrustbooks-desktop | grep -c wdio-webdriver`: 56。
- 直後に既定 feature で再ビルドし、同じ検査: 0。wdio permission が本番バイナリへ混入しないことを確認。
- `desktop/gen/schemas/capabilities.json` のキーは `wdio` のみで、test 用 capability の分離を確認。

## 結果

- 良い影響: A05 前提 2 の「実ビルド完了証跡」が確定。A05 全体ではなく、前提 2 の未達 1 項目を解消
  （前提 1 の nonce/hash と meta/config CSP 一本化は ADR-0264 の未決定として残る）。
- 悪い影響: 実 WebView の拒否経路は未検証のまま。withGlobalTauri=true では invoke_handler
  登録済み command へ到達可能な点も変わらない（ADR-0332 の可視化を維持）。

## 未決定

- WDIO spec への neg 経路追加と E2E 期待件数の更新（現行 273 件を維持するため今回は対象外）。
- runtime ACL の導入と実 WebView での権限挙動検証（ADR-0332 を追跡先とする）。

## 参照

- ADR-0278: capability 静的スコープゲート
- ADR-0348: wdio capability の静的ゲート
- ADR-0332: capability 範囲の可視化
