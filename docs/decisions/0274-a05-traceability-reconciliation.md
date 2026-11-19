---
type: Decision
title: "ADR-0274: A05 前提別状態のトレーサビリティ整合"
description: "R6 A05 監査で判明した前提 1〜6 の達成度を再分類し、JS / IPC 契約レベルと実 WebView 未達を切り分ける。"
tags:
  - ADR
  - traceability
  - a05
  - desktop
generated:
  by: dsh (a05-trace-writer)
  at: "2026-11-20T06:26:00+09:00"
status: accepted
---

## 背景

R6 A05 監査で、前提別状態・runbook・README・ADR-0254 への追記漏れが判明した。
[GOAL.md](../../GOAL.md) は A05 の達成／未達を文書へ正確に反映し、
全体を完了としないことを契約としている。

既存文書には次の齟齬があった。

- `docs/runbook-release.md` が「desktop は未接続」と古い現状を記載していた
  （ADR-0272 で mock 経路は接続済み）。
- README の主な機能に desktop 設定保存が未反映だった。
- ADR-0254 に ADR-0272 / 0273 の事後経緯が無かった。
- ADR-0253 の実装前提 1〜6 に現在の達成度が無かった。

本次は文書のみの変更であり、コード・E2E・Cargo.* は変更しない。

## 前提別状態

| 前提 | 状態 | 根拠 | 詳細 |
| --- | --- | --- | --- |
| 1 `withGlobalTauri` / CSP | 一部完了 | ADR-0264 / 0268 / 0270 | 経路（`__TAURI__.core.invoke`）は確定。nonce/hash と meta/config CSP の一本化は未決定。 |
| 2 capability 最小権限 | 静的ゲート完了 | ADR-0278 | web 側 IPC 表面を 2 command に固定。実 WebView の権限挙動は未検証。 |
| 3 UI 起点 CAS + 構造化エラー | 完了 | ADR-0269 | `ConfigPatch.expected` と `CommandError` で stale 更新を判別。 |
| 4 a11y | 未検証 | ADR-0272 / 0273 | 実装は完了（`for` / `id` / `aria-invalid` / `aria-describedby` / `role="status"`）。実 WebView 検証は未実施。 |
| 5 Tauri mock E2E +1 | 完了 | ADR-0272 | 256/0（check 239 + equal 17）。JS / IPC 契約レベルの検証。 |
| 6 実 WebView | 環境ブロッカー | ADR-0273 | Tauri CLI 未導入（`cargo tauri --version` 終了コード 101）で起動不可。 |

状態の分類は「完了 / 一部完了 / 未検証 / 環境ブロッカー」に限定した。
前提 2 の静的ゲートは完了（ADR-0278）だが、実ビルドの完了証跡が出るまで実 WebView の権限挙動は「未検証」として扱う。

## 決定

- A05 全体は**完了としない**。JS / IPC 契約レベルの達成と実 WebView 未達を分けて記録する。
- Tauri mock E2E 256/0 は JS / IPC 契約の検証であり、実 WebView の代替ではない
  （ADR-0273）。CDP 接続の E2E も同様に代替としない。
- 前提 3 / 5 は完了、前提 1 は一部完了、前提 4 は実装完了・未検証、
  前提 2 は静的ゲート完了（実 WebView の権限挙動は未検証）、前提 6 は環境ブロッカーとして扱う。
- 実 WebView 検証（往路／復路／表示／a11y）と capability の実ビルド完了証跡は「残」として維持する。
- E2E の check 名・件数・コードは変更しない（256 維持）。
- 既存の verify.sh 全ゲート成功 / E2E 256 passed / 0 failed（ADR-0272）を現行契約として維持する。

## 根拠

- ADR-0253: 段階設計と実装前提 1〜6。browser readonly / fail-closed の維持。
- ADR-0264: IPC 経路の比較と、nonce/hash・meta/config CSP 一本化の未決定。
- ADR-0268: meta / desktop config CSP の整合を検査する静的ゲート（実 WebView 挙動は未検証）。
- ADR-0269: UI 起点 CAS と構造化エラーの実装（前提 3 完了）。
- ADR-0270: `withGlobalTauri` と `__TAURI__.core.invoke` の確定（前提 1 の経路）。
- ADR-0271: DTO / a11y / E2E の未実装を切り分けた設計受容。
- ADR-0272: 完全な `ConfigView` DTO 往復と Tauri mock E2E 256/0（前提 5 完了）。
- ADR-0273: 実 WebView 検証の環境ブロッカー記録（前提 6）と mock 非代替の明記。
- Runtime Proof: `cargo tauri --version` は `no such command: tauri`（終了コード 101）、
  `DISPLAY=:1` / `xdotool` / `wmctrl` は存在する（ADR-0273 の実測を継承）。
  Tauri mock E2E 256 passed / 0 failed は ADR-0272 の verify.sh 実測を継承する。

## 未決定

- capability の実ビルド挙動（最小権限の記述が実際の ACL へどう反映されるか、ADR-0253）。
- `withGlobalTauri` に伴う inline script の nonce/hash 具体策（ADR-0091 / 0264 / 0268）。
- meta CSP と desktop config CSP の一本化（ADR-0264 / 0268）。
- Tauri CLI の導入責任者と時期、GTK / WebKitGTK 依存の有無（ADR-0273）。
- CI billing の解消と実 CI 検証（ADR-0219 / ADR-0237 / ADR-0254）。

## 影響

- 本 ADR を正として、runbook / README / ADR-0253 / ADR-0254 / todo / 各索引・log を追従する。
- browser は読み取り専用 fail-closed、desktop は mock 経路まで接続済み・実 WebView 未検証。
- A05 は残（未着手）のままである。

## 検証（Runtime Proof）

- 文書のみの変更。コード・E2E・check 名・Cargo.* は未変更（256 維持）。
- markdownlint: 0 issues（変更対象を含む `docs/**/*.md` / README.md / todo.md）。
- 実 WebView 検証・外部サービスへのアクセス・Tauri CLI 導入は行わない。

## 参照

- [GOAL.md](../../GOAL.md) — 本件の契約
- [Runbook: リリース運用](../runbook-release.md) — 既知問題の現状開示
- [ADR-0253: Tauri 設定保存 UI の段階設計](0253-tauri-settings-save-design.md)
- [ADR-0272: Tauri 設定保存 UI の接続](0272-settings-save-ui.md)
- [ADR-0273: 実 WebView 検証の環境ブロッカーとしての記録](0273-real-webview-verification-deferred.md)
