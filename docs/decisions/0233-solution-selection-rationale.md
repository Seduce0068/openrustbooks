---
type: Decision
title: "ADR-0233: 既存ソリューション選定根拠の横断記録"
description: "主要依存のライセンス・鮮度・既知脆弱性・Wasm 対応を横断比較し、選定根拠を一箇所に記録する決定。"
tags:
  - ADR
  - tech-selection
  - dependencies
  - license
  - wasm
generated:
  by: dsh
  at: "2026-11-19T17:59:54+09:00"
status: accepted
---

## 目的

選定根拠を述べた ADR は個別の依存ごとに散在し、ライセンス・鮮度・既知脆弱性を
横断して比較できる表が無い。本 ADR は既存の決定を上書きせず、
現状確認日の実測値を 1 つの表へ集約し、要件から E2E への追跡と併せて
参照できるようにする。

## 背景

- `docs/e2e-traceability.md` は VC → E2E check（254 件）のみを記載し、
  REQ ID と主要 ADR ID の列が無かった。
- `docs/requirements.md` の検証条件は「候補であり承認済みの受け入れ基準ではない」
  と明記されており、初版の測定可能な受け入れ基準への参照先が無かった。
- 選定関連の ADR（ライセンス監査、脆弱性ゲート、feature 削減、WASM 配線など）は
  ADR-0001 〜 ADR-0232 に分散し、横断比較表が無かった。

## 決定

### 選定根拠の横断比較

| 依存 / 技術 | 採用 | ライセンス | 鮮度（現状確認日） | 既知脆弱性 | Wasm 対応 | 根拠 ADR |
| --- | --- | --- | --- | --- | --- | --- |
| Rust 2024 / MSRV 1.95 | edition 2024 / rust-version 1.95 | —（言語・ツールチェーンは自動生成表の対象外。公式は MIT OR Apache-2.0） | 2026-11-19 確認（Cargo.toml、cargo 1.100.0） | 対象外（cargo audit は Cargo.lock を走査） | wasm32-unknown-unknown ビルド可 | ADR-0003 / 0214 |
| Tauri | 2.12.5 | Apache-2.0 OR MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | native desktop 専用（wasm 対象外） | ADR-0001 / 0097 / 0129 / 0232 |
| Leptos | 0.8.22 | MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | CSR / wasm32 でビルド可 | ADR-0003 / 0161 |
| async-openai | 0.42.2 | MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | native のみ（wasm は明示エラー） | ADR-0002 / 0104 / 0156 |
| reqwest | 0.13.5 | MIT OR Apache-2.0 | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | native 側で使用（wasm は target 分離で回避） | ADR-0003 / 0101 |
| rumdl | 0.3.2 | MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | featureless で wasm 対応 | ADR-0002 / 0155 / 0160 / 0195 |
| comrak | 0.59.0 | BSD-2-Clause | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | wasm の Markdown Preview で使用可 | ADR-0162 / 0163 |
| ammonia | 4.2.5 | MIT OR Apache-2.0 | 2026-11-19 確認（Cargo.lock） | 既知脆弱性は修正済み（ADR-0086）、baseline 0 vulnerabilities | native / wasm 共通サニタイズ | ADR-0012 / 0086 / 0181 |
| pulldown-cmark | 0.13.6 | MIT | 2026-11-19 確認（Cargo.lock） | 既知脆弱性は修正済み（ADR-0086）、baseline 0 vulnerabilities | 推移依存、rumdl 経由で wasm 対応 | ADR-0086 / 0163 |
| zip | 9.1.3 | MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | wasm32 向け API を公開 | ADR-0004 / 0223 |
| quick-xml | 0.42.0 | MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | native / wasm の EPUB 解析で使用 | ADR-0005 |
| serde / figment | serde 1.0.229 / figment 0.10.19 | MIT OR Apache-2.0 | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | serde は両対応、figment は native 設定のみ | ADR-0204 / 0209 / 0210 |
| tokio | 1.54.1 | MIT | 2026-11-19 確認（Cargo.lock） | 直接該当なし（baseline 10 warnings / 0 vulnerabilities） | native のみ（wasm では非使用） | ADR-0003 |

### 出典と測定義務

- ライセンスは [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) の
  自動生成表を出典とし、ここでは代表値を要約する。個別の通知・著作権表示は
  同ファイルを正とする。Rust 自体は依存ではないため同表に記載がなく、
  ライセンス欄を「—」とした。
- 鮮度と脆弱性は `cargo metadata --locked --format-version 1`、`cargo audit`、
  `Cargo.lock` の **2026-11-19 実測**に基づく。現状確認日は 2026-11-19 である。
- `cargo audit` は 793 crate を走査し、10 allowed warnings /
  0 vulnerabilities（終了コード 0）だった。この 10 warnings は歴史 ADR（ADR-0125 / 0126）の
  baseline と同数で、直接の脆弱性は 0 件。
- 依存バージョンは `Cargo.lock` の実測値を採用した。
  rumdl の featureless 宣言は [scripts/check-rumdl-featureless.sh](../../scripts/check-rumdl-featureless.sh)
  が担保する（ADR-0155 / 0195）。

## 未決定

- 新規依存選定時に `tech-selection` スキルを必須化するか、
  評価マトリクスの保存先は未決定。本 ADR は既存依存の記録にとどめる。
- ライセンス変更の監視頻度と、自動生成表の再生成・検知の自動化は未決定。
- Dependabot の major 更新は手動判断を維持し、自動判断への移行は未決定
  （patch / minor の週次検出は ADR-0215 で決定済み）。
- Wasm 対応は 2026-11-19 時点の確認であり、次回の target 更新時・
  新規依存追加時の再評価タイミングは未決定。

## 影響

- 既存 ADR の決定は上書きしない。本 ADR は参照と横断整理のみを行い、
  各依存の詳細は根拠 ADR を正とする。
- VC は本 ADR により初版の測定可能な受け入れ基準として参照する。
  要件側の「候補」表記は変更せず、E2E check 名 254 件は不変。
- 新規依存なし、unsafe なし、コード変更なし。

## 参照

- [ADR-0001: デスクトップ版にOS WebViewを採用する](0001-use-system-webview.md)
- [ADR-0003: 共通コアをターゲット別依存でNative/Wasm両対応にする](0003-native-wasm-target-split.md)
- [ADR-0004: ZIPアーカイブ閲覧をcoreの読み取り専用モジュールとして実装する](0004-zip-archive-reader.md)
- [ADR-0005: EPUB本文の読み出しをcoreの読み取り専用モジュールとして実装する](0005-epub-body-reading.md)
- [ADR-0012: EPUB の HTML をサニタイズして表示する](0012-epub-html-display.md)
- [ADR-0086: 既知脆弱性依存の修正](0086-dependency-vulnerability-fix.md)
- [ADR-0101: 送信先ベース URL の許可リスト](0101-base-url-allowlist.md)
- [ADR-0104: 送信前のネイティブ同意ダイアログ](0104-send-consent-dialog.md)
- [ADR-0125: 脆弱性監査のローカルゲート](0125-audit-gate.md)
- [ADR-0126: CI での脆弱性監査有効化](0126-ci-audit-gate.md)
- [ADR-0155: rumdl の feature 削減](0155-rumdl-feature-reduction.md)
- [ADR-0156: Wasm AI/lint スタブの fail-closed 化](0156-wasm-stub-failclosed.md)
- [ADR-0160: Wasm Markdown linter の実配線](0160-real-wasm-lint.md)
- [ADR-0161: 最小限の textarea エディタ UI](0161-minimal-editor-ui.md)
- [ADR-0162: Markdown Preview のレンダリング](0162-markdown-preview-rendering.md)
- [ADR-0163: 未使用 Markdown 依存の削除](0163-remove-unused-markdown-dep.md)
- [ADR-0181: SVG サニタイズの共通化](0181-svg-sanitizer-unification.md)
- [ADR-0195: rumdl featureless 検査の独立化](0195-rumdl-feature-gate.md)
- [ADR-0204: 環境変数マッピングの明示化](0204-env-mapping.md)
- [ADR-0209: 設定保存の原子化](0209-atomic-config-save.md)
- [ADR-0210: 設定更新の revision CAS](0210-config-revision-cas.md)
- [ADR-0214: MSRV 1.95 の宣言](0214-msrv.md)
- [ADR-0215: Dependabot の導入](0215-dependabot.md)
- [ADR-0223: ZIP リーダーの WASM API 公開](0223-zip-wasm-api.md)
- [ADR-0232: 初版リリーススコープ](0232-initial-release-scope.md)
- [docs/e2e-traceability.md](../e2e-traceability.md)
- [docs/requirements.md](../requirements.md)
