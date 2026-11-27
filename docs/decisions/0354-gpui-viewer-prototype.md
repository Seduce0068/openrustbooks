---
type: Decision
title: "ADR-0354: GPUI 版ビューアのプロトタイプ追加"
description: "既存 Tauri/Leptos 版と Android/PWA を維持したまま、openrustbooks-core を再利用する独立 GPUI クレートを追加する決定。"
tags:
  - ADR
  - gpui
  - desktop
  - viewer
generated:
  by: dsh (lead)
  at: "2026-11-28T00:42:53+09:00"
status: accepted
---

## 背景

既存 desktop は Tauri + Leptos + WebView 構成で、Android 版も同じ WebView UI を
共有する。GPUI（Zed 由来の Rust UI フレームワーク）版は過去セッションで
条件付き検討・PoC とされたが、リポジトリには実装も ADR も存在しなかった。
先行事例として JARVIS の adr/0005-gpui-desktop-adoption.md と ui/ クレートが
同じ Kubuntu / Wayland 環境でビルド・起動済みであり、再利用できる知見がある。

## 決定

- GPUI 版は gpui/ に独立 workspace として追加する。ルート Cargo.toml の members
  には入れず、workspace.dependencies も変更しない。これによりルートの MSRV 1.95・
  wasm ゲート・Android クロスビルド CI を維持する。
- 依存は gpui-pre / gpui-pre-platform 0.3.12 を gpui / gpui_platform 名で alias し、
  GPUI 固有クレートの MSRV は 1.100 とする。公式 gpui 0.2.2 は Linux の xattr
  問題でビルドできない（JARVIS ADR-0005 の実測）。
- EPUB の解析・サニタイズ・設定型は openrustbooks-core を path 依存でそのまま
  再利用する。GPUI 側は EPUB バイト列の読み込み、章選択、プレーンテキスト表示、
  固定日本語エラー表示のみを実装する（PoC 範囲）。
- 画像表示・スクロール位置保持・設定 UI・AI 送信は PoC の対象外とし、既存 Tauri/Leptos 版を唯一の機能完全版として維持する。
- Rust の unsafe は使用しない。clippy -D warnings -D clippy::undocumented_unsafe_blocks をゲートとする。

## 代替案

| 案 | 判断 | 根拠 |
| --- | --- | --- |
| ルート workspace に GPUI crate を追加 | 却下 | Android クロスビルドと MSRV 1.95 の契約を破る |
| 公式 gpui 0.2.2 を利用 | 却下 | Linux の xattr ビルド失敗が先行事例で記録済み |
| desktop crate を GPUI へ置換 | 却下 | Tauri IPC / Android / PWA の検証契約を失う |
| 独立 workspace の gpui/ を追加 | 採用 | 既存ゲートを維持しつつ PoC を検証できる |

## 結果

- 良い影響: core の fail-closed な EPUB/ZIP 解析をそのまま再利用でき、GUI 側の実装は表示と操作だけに集中できる。JARVIS の実機知見により Wayland 固有コードを自作せずに済む。
- 悪い影響: 独立 lockfile と GPUI 依存（ネイティブ图形スタック）を追加するため保守対象が増える。GPUI 版はプレーンテキスト表示のみで既存版と機能差がある。

## 検証（Runtime Proof）

- cargo fmt --all: rc=0。
- cargo check --offline: rc=0。
- cargo test --offline: 3 passed / 0 failed（純粋ロジックと fail-closed 入力）。
- cargo clippy --offline --all-targets -- -D warnings -D clippy::undocumented_unsafe_blocks: rc=0。
- cargo run --offline -- /tmp/gpui-sample.epub: プロセスが起動後 40 秒以上生存し、异常終了しなかった。画面上の描画内容はスクリーンショットで確定できず未確認。

## 未決定

- 実機 Wayland ウィンドウの目視確認とキーボード操作（前の章 / 次の章）の検証。
- 画像・表紙・スクロール位置保持・設定 UI の移植範囲。
- gpui-pre の最新版 0.3.14 への更新タイミング（現状は JARVIS 実績のある 0.3.12 固定）。

## 参照

- JARVIS ADR-0005: ../jarvis/jarvis/adr/0005-gpui-desktop-adoption.md（ローカル先行事例）
- GPUI 公式: <https://gpui.rs>
