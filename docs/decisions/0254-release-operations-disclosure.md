---
type: Decision
title: "ADR-0254: リリース運用の文書先行開示"
description: "導入・更新・ロールバックと既知問題を先に開示し、公開・署名・endpoint は未決定として残す決定を記録する。"
tags:
  - ADR
  - release
  - operations
  - disclosure
generated:
  by: dsh (release-runbook-writer)
  at: "2026-11-20T00:26:18+09:00"
status: accepted
---

## 背景

R6 R3-release#4,#5。ADR-0222 は updater 未配線を設計受容とし、
公開鍵・署名 secrets・配布 endpoint・Release workflow・ロールバック手順を
未決定として残していた。[GOAL.md](../../GOAL.md) は、外部サービスや成果物公開を
行わずに利用者向けの現状開示と署名 material の誤コミット防止を先に整備することを
契約としている。現状は公開 installer、updater、署名済み成果物、更新用／配布用の
公開 endpoint のいずれも存在せず、設定表示は「更新確認: 未配線」である。
`desktop/tauri.conf.json` の `createUpdaterArtifacts: true` は成果物生成フラグの残置で、
署名・公開は行われていない。AI 用 endpoint は既存設定に残る。
本 ADR は、運用文書を先に開示し未決定を正直に残す判断を記録する。

## 決定

- 導入・更新・ロールバック手順と既知問題を
  [docs/runbook-release.md](../runbook-release.md) に先に開示する。
- 現状を「公開 installer / updater / 署名済み成果物 / 公開 endpoint は無い」と
  正直に記載し、`check_updates` は死設定であることを明記する。
- 公開 installer、署名済み成果物、公開 endpoint、自動更新、自動 rollback は
  今回作成・変更・公開しない。
- 検証済み配布物の導入・更新・ロールバックは
  [runbook-verified-build.md](../runbook-verified-build.md) と整合する
  手動手順として定義する。
- 署名秘密鍵・release secrets はリポジトリ外または CI secrets に置き、
  `.gitignore` で拒否する。R6 で親が署名 material の除外パターンを追加済み。
- 公開鍵の設定、署名 secrets の登録、endpoint、billing、Release workflow、
  `require_signed_version`、実機検証は未決定として残す。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 即 Release workflow を整備して公開 | 却下。署名 secrets / endpoint / ホスティング / billing の責任分界が未定義で、GOAL.md の「外部サービス・成果物公開は行わない」に反する |
| 文書先行で現状と既知問題を開示（採用） | 外部影響なしで契約を満たす。未決定を明示するため過剰な約束をしない |
| 完全非開示 | 却下。updater 未配線や設定 readonly を利用者へ伝えず、誤認と信任低下を招く |

## 根拠

- ADR-0219: 検証済み dist は全ゲート成功時のみローカル保持し、SHA256 は
  完全性のみ。署名／真正性と Release workflow との境界は未決定であり、
  配布を約束できない。
- ADR-0222: updater は初期化のみで更新チェックは未配線。
  公開鍵・endpoint・署名 secrets が無い状態では接続しても fail-closed となり、
  現状開示が先決である。
- ADR-0232: 初版スコープは自動更新、アップデータの署名と配布、実機 5 ブラウザ検証を
  含まない。開示内容はこの境界と一致させる。
- ADR-0193: ブラウザ版の設定フォームは契約どおり読み取り専用で、
  変更可能と誤解させない方針を既に確立している。
- ADR-0253: Tauri 設定保存 UI は browser readonly を維持し、desktop のみ
  段階設計で接続する。保存 UI が未接続であることを既知問題として開示する。
- ADR-0239: Cargo workspace をバージョンの単一情報源とし drift を防いでいる。
  公開成果物のバージョン決定の前提は整っているが、署名・配布は未実装である。
- ADR-0246: feature 差分の無い workspace 依存を統一した背景記録。
  署名済み配布物の生成には至っていない（直接根拠ではなく背景）。

## 影響

- 新規 ADR 1 件と Runbook 1 件を追加し、各索引・README・todo・log を追従する。
- コード・CSP・tauri.conf・公開鍵・秘密鍵・Release workflow は変更しない。
- 外部サービス、billing、成果物公開は行わない。
- 未決定を減らすものではなく、現状の境界を利用者へ可視化するのみである。

## 未決定

- 設定保存 IPC の UI 起点 CAS / 構造化エラーは [ADR-0269](0269-ipc-cas-contract.md) で実装済み。

- 公開鍵の生成・配布・ローテーションと失効（kill switch を含む）。
- 署名 secrets の登録先と管理責任者、失効時の手順。
- 配布 endpoint の URL / ホスティング先と CSP `connect-src` の追加範囲。
- CI billing の解消と実 CI での検証、artifact upload の責任者と期限。
- Release workflow の方式（共通 `rust-gates.yml@v1` のタグ付け替え vs v2）。
- `require_signed_version` を有効化する時期。
- 実機 5 ブラウザ（Chrome / Edge / Brave / Firefox / Safari）での検証。
- 自動更新・自動 rollback の要否と、実装する場合の同意・検証の粒度。
- 保持済み dist の保存先固定、保持期間、ローテーションと破棄の主体。

## 検証（Runtime Proof）

- 現状は ADR-0222 / 0219 / 0232 の Runtime Proof と
  [runbook-verified-build.md](../runbook-verified-build.md) の実測記録を根拠とする。
- 本件は文書と `.gitignore` のみの変更で、コード・E2E は不変。
  R6 で `./scripts/verify.sh` を実行し、全ゲート成功 /
  E2E **255 passed, 0 failed** を確認した。
- `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md`: 0 issues。

## 参照

- [GOAL.md](../../GOAL.md) — 本件の契約
- [Runbook: リリース運用](../runbook-release.md) — 導入・更新・ロールバック・既知問題
- [Runbook: 検証済み配布物の取得・保持・復旧](../runbook-verified-build.md)
- [ADR-0219: 検証済み配布物のローカル保持](0219-verified-artifact-retention.md)
- [ADR-0222: updater 未配線の設計受容](0222-updater-deferred.md)
- [ADR-0232: 初版リリーススコープ](0232-initial-release-scope.md)
- [ADR-0193: 設定フォームの読み取り専用化](0193-settings-form-readonly.md)
- [ADR-0253: Tauri 設定保存 UI の段階設計](0253-tauri-settings-save-design.md)
- [ADR-0239: Cargo をバージョンの単一情報源とする](0239-version-single-source.md)
- [ADR-0246: workspace 依存の第一段統一](0246-workspace-dependency-unification.md)
