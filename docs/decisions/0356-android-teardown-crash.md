---
type: Decision
title: "ADR-0356: Android instrumented test の teardown クラッシュ原因"
description: "Espresso Web の assertion 成功後に Process crashed が起き、FORTIFY が破棄済み mutex へのロックを検出した事実を記録し、@Ignore 継続を決定する。"
tags:
  - ADR
  - android
  - test
  - wry
generated:
  by: dsh (lead)
  at: "2026-11-28T03:12:14+09:00"
status: accepted
---

## 背景

AND-UI-01/02 の Espresso Web テストは AVD API 36 / x86_64 で assertion 成功後に
instrumentation が `Process crashed` で終了するため `@Ignore` で無効化されていた。
原因は推測のままで、生成物の再現ログも最新実行は skipped に上書きされていた。

## 決定

- `@Ignore` を継続する。assertion 自体は成功しており、クラッシュは teardown 工程中に
  発生する。ログからは FORTIFY の破棄済み mutex ロックまでが事実で、mutex の所有者は未確認。
  ネイティブ lifecycle の可能性はあるが、WRY / Tauri 固有と断定しない。
- 生成ソース（desktop/gen/android）と JNI/native 側の修正は行わない。unsafe を追加する
  選択も global 指示で禁止されている。
- 上流/生成側の lifecycle 修正が入るまで、原因と再現手順を ADR と runbook に残す。

## 検証（Runtime Proof）

- AVD orb_api36（API 36 / x86_64）を起動し、`web/` で `trunk serve --port 1420`、
  `adb forward tcp:1420 tcp:1420`、Tauri CLI の options server を用意。
- `@Ignore` を一時的に外して `./scripts/run-android-instrumented-tests.sh` を実行:
  assertion は失敗せず、`Finished 1 tests` の直後に `Process crashed`。
- `adb logcat -d -b crash`:
  `F libc: FORTIFY: pthread_mutex_lock called on a destroyed mutex (0x7e4c82c02c38)`
  が 2 スレッドから記録された（2026-11-28T03:11:01+09:00）。
- クラッシュ buffer に tombstone は無く、シグナル種別は未確認。

## 結果

- 良い影響: 「assertion 成功・teardown クラッシュ」とクラッシュ時のログ事実を記録し、
  再現手順を確定した。テスト側 / ネイティブ側の切り分けは未完。
- 悪い影響: Android instrumented test は引き続き無効。実機 WebView の自動検証は
  WebdriverIO（desktop）と手動確認に依存する。

## 未決定

- mutex 所有者の特定（tombstone / スタック取得）と、ネイティブ側またはテスト側の切り分け。
- `forceJavascriptEnabled()` の重複呼び出し（3 箇所）はテスト側要因の候補として未決定。
  tombstone が無いためログだけでは否定できない。

## 参照

- android-test/androidTest/WebViewSmokeTest.kt
- docs/runbook-mobile.md（Android instrumented test 手順）
