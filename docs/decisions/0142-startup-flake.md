---
type: Decision
title: "ADR-0142: E2E 起動待ちフレークの診断と再試行"
description: "E2E 起動時に CDP 診断イベントを収集し、マウント失敗時のみ 1 回再試行する決定を記録する。"
tags:
  - ADR
  - testing
  - e2e
generated:
  by: dsh
  at: "2026-11-18T14:35:00+09:00"
status: accepted
---

## 背景

ADR-0140 の 10 回連続実行で `Reader ルートが表示される` が 20 秒待っても false となり、
`expected 229 checks, but executed 1` で早期終了した（第 10 回）。
原因は再現せず特定に至らなかったが、
`Cdp.send()` が console / pageerror / requestfailed を全て破棄しており、
起動失敗を診断できない状態だった（事実）。

## 決定

- `tools/verify-web-reader.py`:
  - `Cdp.send()` が診断対象イベント（console / exception / Log /
    loadingFailed / dom / load）を退避する。
  - navigate 前に `Runtime` / `Log` / `Network` を有効化。
  - `Page.addScriptToEvaluateOnNewDocument` で window error、
    unhandledrejection、`TrunkApplicationStarted`、DOMContentLoaded を記録。
  - 20 秒でマウントしない場合のみ診断を出力し、
    `Page.reload` を **1 回だけ**再試行する。
  - `checker.check` は最後に 1 度だけ呼び、期待総数 **229** を維持。
- `web/src/lib.rs`:
  - wasm-bindgen start と mount 完了時に
    `window.__orbRustStartedAt` / `window.__orbRustMountedAt` を付与し、
    起動段階（fetch / init / mount）を判別可能にする。
- アプリの機能は変更しない（診断と待機の安定化のみ）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 診断収集＋1 回再試行（採用） | 理由が分かり、期待 229 も維持できる |
| 再試行なし | 原因不明のままフレークに晒される |
| アプリに ready シグナルを追加 | ADR-0141 の完了シグナルと同種で範囲が広がる（未決定） |
| 固定待ち延長 | 根本解決にならない |

## 影響

- 起動失敗時にコンソール／ネットワーク／マーカーを含む診断が出る。
- `web/src/lib.rs` に 20 行追加（マーカー設定のみ）。
- 新規依存なし、unsafe なし。

## 検証

- 強制失敗ダミー dist で診断経路を検証:
  console.error / pageerror / Log / リロード前後診断 /
  `expected 229 checks, but executed 1` を正しく出力。
- 5 回連続（14:12〜14:23）すべて 229/0、フレーク未再現。
- `./scripts/verify.sh` rc=0（全ゲート・audit・trunk build・E2E 229/0）。
- 全単体テスト core 188 + desktop 56 + web 2、wasm-test 3、
  audit 脆弱性 0 件。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（startup-flake-auditor の指摘）

- 真の原因（24MB Wasm のコールドスタート／fetch 失敗／execution context
  競合／固定ポート残存等）は未再現のため未特定。
- ~~CDP 接続先を起動 PID と照合する~~ ADR-0143 で実装済み（browser-level WS）。
- リトライ回数・バックオフ、リトライ前に `localStorage.clear()` するか。
- エラー収集の置き先とリスナー名（`__orbSettingsErrors` は settings 固有）。
- ~~`Cdp.send()` の recv タイムアウト~~ ADR-0144 で 10 秒＋fail-closed を実装済み。
- アプリ側 ready シグナルの追加（ADR-0141 の `data-downscaled` と同種）。
- 20 回連続実行、CLS 計測、5 ブラウザ実機検証（ADR-0140 から継続）。
