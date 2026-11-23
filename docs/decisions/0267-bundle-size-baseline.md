---
type: Decision
title: "ADR-0267: WASM バンドルサイズの比較可能ベースライン"
description: "verify.sh と同一のビルド条件で現行 WASM 配布物のサイズを実測し、今後の比較可能 before とする決定を記録する。"
tags:
  - ADR
  - wasm
  - performance
generated:
  by: dsh
  at: "2026-11-20T04:12:00+09:00"
status: accepted
---

## 背景

ADR-0243 / 0246 / 0256 / 0259 / 0260 は
バンドルサイズの before / after 実測を未決定として残していた。
`docs/investigation/round-02.md:65` に 22.99 MB / gzip 2.53 MB の参考値があるが、
profile・commit・RUSTFLAGS・gzip 条件が不明で比較できない。
初回計測は verify.sh と異なる RUSTFLAGS だったため、再計測した。

## 決定

- 現行ソースを verify.sh と同一条件でビルドし、これを今後の比較可能 before とする。
- ビルド条件:
  - `RUSTFLAGS="--remap-path-prefix=$HOME=~"`
  - `trunk build --offline --locked --dist /tmp/orb-bundle-verified`
  - dev profile（trunk 既定、`--release` なし）
  - `gzip -n -c`（filename / timestamp を埋め込まない）
- 過去の 22.99 MB / gzip 2.53 MB は「条件不明の参考値・非比較」として扱う。
- 今回の値は git commit `ba5a889a761de5c17e3143b5a4f7c90f11a9c735` と
  Cargo.lock SHA-256 `782215028b8258c1778e8482c5abb7d343062110090db709aef46dffd5aa399b`
  に紐付ける。

## 実測値

| ファイル | raw bytes | gzip bytes (-n) | SHA-256 |
| --- | ---: | ---: | --- |
| `favicon.svg` | 389 | 255 | `ebce3124d04edef1930acb3edaa4825647a6d653e86f1fdd9aca3bd6ac88b18e` |
| `index.html` | 20,889 | 4,728 | `aa141faf38bd816ec13fb6d0d68446a3d1266375f08bdc2e0c21d8635ccdf733` |
| `openrustbooks-web-1713a623073ed8e_bg.wasm` | 69,012,923 | 12,183,616 | `31e2ccbd0ccbfb006c816646254a65650edad9884a6b03d3ebfee43ad7741693` |
| `openrustbooks-web-b7c824cf6fd7e902.js` | 93,786 | 15,322 | `873506313f151f4d7e112f9e3643c30d4ccfebe45548658b5bd793e4cc97d544` |
| `.trunk-lock` | 0 | 20 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| **合計** | **69,127,987** | **12,203,941** | — |

- WASM raw: **69,012,923** bytes（約 65.81 MiB）、gzip: **12,183,616** bytes。
- JS raw: **93,786** bytes、gzip: **15,322** bytes。
- `/home/user` 文字列の WASM 内出現回数: **0**（remap 適用を確認）。

## ツールバージョン

- trunk 0.22.2
- wasm-bindgen 0.2.129
- rustc 1.100.0 (`0ef1bc386 2026-11-09`)
- cargo 1.100.0 (`3acc79c0c 2026-11-05`)
- gzip 1.14

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| verify.sh 同一条件で再計測（採用） | 配布物と比較可能な値が得られる |
| 初回の無 RUSTFLAGS 値を使用 | ADR-0088 の path stripping 契約と異なる |
| 過去の 22.99 MB 値を使用 | 条件不明で比較不能 |

## 影響

- 今後の API 削除 / 分割 / 依存変更は、本値と同じ条件で比較する。
- E2E check 名・件数（255）・コードは変更しない。
- `web/dist` は git 対象外の古い生成物で、本次は一時 dist のみを使用した。

## 検証（Runtime Proof）

- ビルド: 2026-11-20 04:06 JST、rc=0。
- 全ファイル SHA-256 と raw / gzip サイズを `/tmp/orb-bundle-verified-sha.txt` に保存。
- 再計算: raw 合計 69,127,987 / gzip (-n) 合計 12,203,941。
- `./scripts/verify.sh`: 全ゲート成功、E2E **255 passed / 0 failed**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- release profile (`trunk build --release`) での値。現状は dev profile を採用。
- サイズ上限ゲートの指標（raw / gzip）、閾値、導入先（verify.sh か extra_gates）、
  ベースライン更新ポリシ。
- toolchain / trunk / wasm-bindgen 更新時の再計測タイミング。
- gzip レベルを `-n` 既定（6）で固定するか、より高圧縮を採用するか。
- 他ブラウザの転送サイズ（ADR-0265 の環境ブロッカーを継続）。
