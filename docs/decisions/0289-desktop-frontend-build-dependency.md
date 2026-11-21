---
type: Decision
title: "ADR-0289: desktop ビルド前の frontend dist 生成と検証"
description: "tauri.conf.json の frontendDist 依存を明示化し、生成済み web/dist の存在と安全性を fail-closed で検証する決定。"
tags:
  - ADR
  - build
  - tauri
generated:
  by: dsh
  at: "2026-11-22T03:42:00+09:00"
status: accepted
---

## 背景

verify.sh は trunk build を一時 DIST へ出力するため web/dist を更新しない。
一方 desktop/tauri.conf.json の frontendDist は ../web/dist を指し、
cargo build グラフにはその生成依存が無い。
このままだと古い、あるいは存在しない web/dist が desktop バイナリへ埋め込まれ得る。

## 決定

- `scripts/build-desktop.sh` を追加する。
  - `trunk build --offline --locked --dist web/dist` で frontend を生成する。
  - 生成直後に `check-frontend-dist.sh` を呼ぶ。
  - `cargo build -p openrustbooks-desktop --locked` を実行する。
- frontend は verify.sh の E2E と同じ dev プロファイルで統一する。
  `tauri build` の `beforeBuildCommand` は cargo build では発火しないため競合しない。
- `scripts/check-frontend-dist.sh` を追加する。
  - tauri.conf.json の frontendDist を desktop/ 基準で解決し、
    想定する web/dist と一致することを検査する。
  - index.html / `*_bg.wasm` / `*.js` の存在を検査する。
  - wasm / JS に開発ホスト絶対パスが混入していないことを走査する。
- verify.sh の前段 static gates から frontend-dist 検査を外し、
  E2E 一時 DIST 生成後に build-desktop.sh を呼ぶ。
- tauri-build は web/dist を rerun-if-changed で追跡しないため、
  cargo build 前に desktop/build.rs の mtime を更新して再ビルドを強制する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 明示 build script + 生成後検査（採用） | CLI 差分なしで依存を固定できる |
| E2E 一時 DIST を web/dist へコピー | 一時 DIST と配布 DIST を混同するため却下 |
| tauri build に一本化 | 既存 verify.sh / WDIO の cargo build 契約を壊すため見送り |

## 影響

- verify.sh は trunk build を二重実行する（一時 DIST + web/dist）。
  E2E と desktop 埋め込みの条件を同一にするための意図的な選択。
- `web/dist` は Git 管理外のまま。
- wasm-bindgen CLI は Cargo.lock 固定版 0.2.129 を PATH に明示する。
- CI extra_gates への frontend-dist 検査追加は未実施。
  CI が web/dist を生成しないため順序設計が必要。

## 検証（Runtime Proof）

- `bash -n` / `shellcheck`: 新規 2 script と verify.sh は成功。
- check-frontend-dist: positive 1 + negative 9、すべて期待どおり。
- build-desktop.sh: rc=0、trunk build → check → cargo build 成功。
- `./scripts/verify.sh`: 全ゲート成功、build-desktop rc=0、
  E2E **256 passed / 0 failed**、WDIO gate rc=0。

## 未決定

- CI での frontend 生成と check-frontend-dist の順序設計。
- trunk の複数出力先対応による二重 build の解消。
- release プロファイルの frontend dist を配布に使う方針。
