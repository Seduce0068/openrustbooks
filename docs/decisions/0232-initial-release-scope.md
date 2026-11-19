---
type: Decision
title: "ADR-0232: 初版リリーススコープ"
description: "要件と実装の境界を実証付きで明示し、初版で提供する範囲と未決定を切り分ける決定を記録する。"
tags:
  - ADR
  - release
  - scope
  - requirements
generated:
  by: dsh
  at: "2026-11-19T17:37:15+09:00"
status: accepted
---

## 目的

`docs/requirements.md` は利用者の要望を網羅するが、記載がそのまま初版の
約束ではない。初版で実際に提供する範囲を実装・検証の証拠付きで明示し、
未決定事項を誤約束しない境界を設ける。要件自体は削除せず、
実装判断としてスコープを記録する。

## 背景

- 要件には PDF・ZIP UI・Google Drive・直接フォルダ参照・OS ファイル
  マネージャー連携が記載されているが、いずれも仕様は未決定
  （`docs/requirements.md:20-28,141-178`）。
- 実装と ADR は ADR-0231 まで 231 件に達し、E2E 254 件・core 270 件の
  検証実績が揃った（ADR-0231）。
- README の「主な機能」は実装済みと設計段階が混在しており、
  初版の境界を利用者へ誤って伝えるおそれがある（README:81-100）。

## 決定

### 初版スコープ（現状の実装・検証済み範囲）

#### native desktop

- 開発と CI は Linux を対象とし、ワークスペース全体のゲートを Linux 上で
  検証する（README:150-166）。
- デスクトップは OS WebView 方式（Tauri / wry）を採用し、Rust の `core`
  と `desktop` クレートで構成する（ADR-0001 / ADR-0003）。
- MSRV は 1.95（ADR-0214）。設定は `config.toml` を figment で読み書きし、
  tmp + rename の原子保存と revision CAS を適用する
  （ADR-0209 / ADR-0210 / ADR-0211）。
- 設定フォームは読み取り表示とし、保存接続は見送る（ADR-0128）。

#### browser

- Leptos CSR / WASM。Chrome / Chromium の CDP による E2E 254 件
  （`checker.check` 237 + `checker.equal` 17 = 254）を検証する
  （ADR-0090 / ADR-0143 / ADR-0225）。
- EPUB 閲覧を実装済み: 章移動・目次（nav / NCX / landmarks / page-list）・
  表紙・読書位置（章 / フラグメント / スクロール）・許可リスト
  サニタイズ・ARIA とキーボード操作・ダーク表示・
  `prefers-reduced-motion`（ADR-0011 / ADR-0012 / ADR-0021 / ADR-0213 /
  ADR-0230）。
- 表示設定（文字サイズ・行間・ダーク表示等）は localStorage へ保存し、
  開き直したときに復元する（ADR-0018 / ADR-0038）。
- ZIP は fail-closed な WASM API を公開済みだが、UI は未接続
  （ADR-0223）。

#### text

- Markdown は閲覧・下書き・デバウンス自動保存・lint（`rumdl`、
  auto-fix を含む）を実装済み（ADR-0110 / ADR-0160 / ADR-0161 /
  ADR-0164 / ADR-0179）。
- ブラウザの editor は Markdown のみを対象とする。
  CodeMirror は採用せず textarea + Preview の最小構成（ADR-0161）。
  format 選択と Lint ボタンは disabled で、TXT 編集は初版外
  （web/src/lib.rs:368-384）。E2E の editor 検証は Markdown のみ。

#### AI

- AI は native のみで提供する。ブラウザの AI API は明示的なエラーを
  返す（ADR-0156）。
- provider は OpenAI 互換 API と Ollama（ローカル）（README:75、
  ADR-0002）。
- 既定無効・送信先 allowlist・送信前のネイティブ同意ダイアログ・
  fail-closed の service ガードを適用する
  （ADR-0099 / ADR-0100 / ADR-0101 / ADR-0104 / ADR-0156）。
- `rumdl` による lint retry と auto-fix を品質ゲートとする
  （ADR-0002 / ADR-0110 / ADR-0116）。

#### 保存 / 検証

- `scripts/verify.sh` が fmt / check / test / clippy / wasm / audit /
  `trunk build` / パス検査 / E2E を決定的に実行するゲートである
  （ADR-0084）。
- `--dist <dir>` 指定時のみ、全ゲート成功した検証済み配布物を manifest
  付きで保持する（ADR-0219）。

### 初版に含まない項目

- PDF ビューア。要件には残すが実装範囲外（REQ-002、README:94）。
- ZIP の UI。WASM API のみで UI 未接続（ADR-0223）。
- TXT 編集。ブラウザでは format 選択が disabled で未実装。
- Google Drive / クラウド同期（REQ-006 / REQ-007）。
- 既存フォルダの直接アクセス（REQ-005）。
- OS ファイルマネージャー連携（REQ-008）。
- モバイル配布。
- 自動更新（ADR-0222）。
- 実機 5 ブラウザ（Chrome / Edge / Brave / Firefox / Safari）での検証。
- カスタム AI endpoint（ADR-0218）。
- 月額使用量の enforcement（ADR-0228 の未配線）。
- ストリーミング UI（ADR-0216）。
- アップデータの署名と配布（ADR-0222）。

## 根拠（Runtime Proof）

- `docs/requirements.md:20-28`: 記録の読み方と「初版ですべて実装する
  ことは未決定」の明示。
- `docs/requirements.md:141-178`: 検証条件の提案・追加検討する品質要件・
  優先して具体化する事項。
- `README.md:81-100`: 主な機能（実装済みと設計段階の区分）。
- ADR-0001 / ADR-0003: OS WebView 方式と native / wasm のターゲット分離。
- ADR-0011: Web 版でのローカル EPUB 閲覧。
- ADR-0104: AI 送信前のネイティブ同意ダイアログ。
- ADR-0128: 設定フォームの読み取り表示と保存見送り。
- ADR-0216: ストリーミング未接続の設計受容。
- ADR-0222: updater 未配線の設計受容。
- ADR-0223: ZIP リーダーの WASM API 公開（UI 後段）。
- ADR-0224: 古い TODO の ADR 実績による整理。
- ADR-0229 / ADR-0230: エラー文言と dark ボタンコントラストの仕上げ。
- ADR-0231: 未検出 backend / provider の test 追加（core 270 passed）。
- E2E 254 件（期待総数 254 と実測が一致、重複・欠落なし）。
- core test 270 件（ADR-0231 の Runtime Proof）。
- `markdownlint-cli2 --config /tmp/mdlint.jsonc`（対象 7 ファイル、
  一時設定の MD013 line_length 120 を含む）: **0 issues / exit 0**。
  再現には同設定が必要（リポジトリへの設定ファイル追加は本次範囲外）。
- 文書のみの変更のため compile / test / `verify.sh` は未実行。
  E2E 254 件と core 270 件は ADR-0231 と `docs/e2e-traceability.md` の
  間接証拠であり、本 ADR の範囲確認に用いる。
- 補助根拠: ADR-0090 / 0099 / 0101 / 0110 / 0116 / 0156 / 0160 /
  0161 / 0164 / 0179 / 0213 / 0214 / 0219。

## 未決定

- 対象 OS の Windows / macOS。現状は Linux 開発・CI のみ検証済み。
- モバイルの Android / iOS 配布。
- ブラウザ 5 種（Chrome / Edge / Brave / Firefox / Safari）の実機対応。
- 初版のリリース日と配布先。
- PDF / ZIP UI の次期スコープ。
- Google Drive / クラウド同期と直接フォルダアクセスの仕様。
- `docs/requirements.md` の他の未決定事項は本 ADR で置き換えず継続有効。
  例: EPUB の対応バージョン / DRM（REQ-002）、Undo・検索置換（REQ-003）、
  MPE の必須機能（REQ-009）、トークン上限・ローカル優先モード（REQ-010）。

## 影響

- 要件は削除せず、初版で提供しない機能も将来の候補として
  `docs/requirements.md` に残す。
- 初版の提供範囲は本 ADR を正とし、README の「主な機能」と併せて
  参照する。要件記載をもって実装を約束したものではない。
- スコープを変更する場合は新規 ADR で決定し、本 ADR と各索引を
  同時に更新する。
