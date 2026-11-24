---
type: Decision
title: "ADR-0352: Android アプリ版を Tauri 2 の Android target として追加"
description: "既存 desktop crate に cdylib ターゲットを追加し、ユーザー管理下のローカルツールチェーンで Android ビルド可能にする決定。"
tags:
  - ADR
  - android
  - tauri
  - mobile
generated:
  by: dsh (mob-android-implementer)
  at: "2026-11-23T15:26:44+09:00"
status: accepted
---

## 背景

対応方針（2026-11-23）で Android はアプリ版と PWA の両方に対応するとされた。
`desktop` crate は Tauri 2 / tauri-build 2、CLI は tauri-cli 2.12.6 を使用している。
初期状態では Java / Android SDK / Android 向け rustup target が無く、
`cargo tauri android init` は
`Java not found in PATH ... JAVA_HOME environment variable not set` で失敗した。
`sudo` とグローバル設定変更は禁止されている。

## 決定

- Android アプリ版は既存 `desktop` crate を Tauri 2 の Android target として使う。
  別 crate は新設しない。
- `desktop/Cargo.toml` に `[lib]`（`cdylib` + `rlib`）を追加する。
  既存 `desktop/src/main.rs` が `tauri::mobile_entry_point` 付き `main` と
  `pub fn run()` を既に持つため、同一ソースを lib として公開する。
  これに伴い lib 側で `dead_code` になる `#[cfg(not(mobile))] fn main` を
  package スコープの `[lints.rust] dead_code = "allow"` で許容する。
- ビルドツールはユーザー管理下のローカル領域へ導入する。
  - JDK: `$HOME/.local/share/dsh-android-tools`
  - Android SDK: `$HOME/.local/share/android-sdk`
- JDK は Temurin 21 LTS を Adoptium API 経由で取得し、公式 checksum と照合する。
  cmdline-tools は Google 公式 ZIP を Android Developers の SHA-256 表で照合する。
- SDK コンポーネントは `sdkmanager` で導入し、ライセンス同意はコマンドで行う。
- rustup target は Tauri 公式 prerequisites に従い 4 つ追加する。
- `cargo tauri android init` で `desktop/gen/android` をローカル再生成する方針とする。
  `desktop/gen` は `.gitignore` 対象のままで、生成物は追跡しない。
  R4 レビューで生成物がレビュー不能だった旨は親エージェントへ通知済み。
- `scripts/build-android.sh` が JDK / SDK / NDK の環境変数を fail-closed で補い、
  `cargo tauri android build` を呼ぶ。引数は `debug` / `release` のみ受け付ける。
- `desktop/tauri.conf.json` は init による差分が無かったため変更しない。

## 結果

### 良い影響

- `sudo` やグローバル設定変更なしで Android ビルド環境を再現できる。
- Tauri CLI の対話的自動インストールに依存せず、配置先とバージョンを固定できる。
- デスクトップ用コードと Android 用コードが重複しない。

### 悪い影響

- JDK・SDK・NDK で約 2.7 GB のローカルディスクを消費する。
- `dead_code` の package スコープ許容により、同 crate の他の未使用検出が緩む。
  現在の検出対象は `#[cfg(not(mobile))] fn main` の 1 件のみ。
- Tauri CLI 2.12.6 が要求する `platforms;android-37` は現行 sdkmanager に存在せず、
  `platforms;android-37.0` を導入する回避策が必要。

## 代替案

- **Android Studio を導入する**: Tauri 公式ガイドの標準手順だが、重量が大きく
  既定パスや環境変数のグローバル設定を伴うため不採用。
- **Android 用 crate を新設する**: `desktop` と plugin 登録・コマンド実装が重複し、
  修正の二重管理が発生するため不採用。
- **Tauri CLI の自動 SDK 導入に任せる**: `$HOME/Android/Sdk` という固定パスと
  対話プロンプトに依存し、今回の「ユーザー管理下の明示パス」要件を満たさないため不採用。

## 検証（Runtime Proof）

- Temurin JDK: `bb8a93f9ae6e7a693b2b721e85dbeff26ae1a77447985a8d7afcdd9c7cf6222e` 一致、
  `openjdk version "21.0.13.1" 2026-11-17 LTS`。
- cmdline-tools: `4e4c464f145a7512b57d088ac6c278c03c9eea610886b35a5e0804e74eedf583` 一致、
  `sdkmanager --version` は `22.0`。
- `rustup target add` で aarch64 / armv7 / i686 / x86_64 の linux-android を追加。
- `cargo tauri android init --ci --skip-targets-install` 成功、
  `desktop/gen/android` に 40 ファイルを生成（Gradle 9.6.1 / compileSdk 37 /
  minSdk 24 / targetSdk 37）。
- `cargo check -p openrustbooks-desktop --locked` と
  `cargo clippy -p openrustbooks-desktop --all-targets --locked -- -D warnings` が成功。
- `scripts/build-android.sh` は `bash -n` / `shellcheck` / `shfmt -d` すべて成功。
- `docs/runbook-mobile.md` は markdownlint-cli2 警告ゼロ。
- `cargo tauri android build --ci --target aarch64` が成功。署名なし APK
  （40,831,720 bytes、SHA-256 `f1c50e6afe40bdb7e01596e581542bf7eeed2729678184ff05f99f9485a95004`）
  と AAB（22,707,533 bytes、SHA-256
  `984420aa2be020c21cd286fc5987a297926f74e020608a0faeffc59fee7eb310`）を生成。
- `aapt2 dump badging` で package `com.openrustbooks.app` /
  minSdk 24 / targetSdk 37 / `INTERNET` permission を確認。
- 成果物名は `universal` だが、検証したのは `--target aarch64`（arm64-v8a）のみ。
  全 ABI（armv7 / i686 / x86_64）のビルドと検証は未実施。

## 未解決

- 全 ABI（armv7 / i686 / x86_64）のビルドと検証は未実施。`universal` という
  成果物名は ABI 混在を意味せず、実測は arm64-v8a のみ。
- 実機（Galaxy S26 Ultra / Android 16）へのインストールと動作確認は未実施。
- 署名鍵と Google Play 登録は未導入。生成 APK は `*-unsigned.apk` である。
- 初回の `cargo tauri android build` は Web 側 `wasm-opt` の bulk memory 検証で失敗したが、
  Android ツールチェーン起因ではない。検証では既存の検証済み `web/dist` を
  設定マージで利用して APK / AAB 生成を確認した。

## 参照

- Tauri prerequisites: `https://v2.tauri.app/start/prerequisites/`
- Android Studio command line tools: `https://developer.android.com/studio#command-tools`
- Adoptium available releases: `https://api.adoptium.net/v3/info/available_releases`
- tauri-cli 2.12.6 の `NDK_VERSION` / `SDK_VERSION`:
  `crates/tauri-cli/src/mobile/android/mod.rs`
