---
type: Decision
title: "ADR-0119: wasm32 テストの実行ゲート"
description: "wasm-bindgen-test と wasm-pack により wasm32 単体テストを実際に実行するゲートを追加する決定を記録する。"
tags:
  - ADR
  - wasm
  - testing
generated:
  by: dsh
  at: "2026-11-18T07:15:00+09:00"
status: accepted
---

## 背景

`core/src/wasm.rs` に wasm32 用テスト 3 件を追加したが、
`scripts/verify.sh` の wasm32 ゲートは `cargo check` のみで、
テストは実行されていなかった（ADR-0117 の未決定、test-scanner #14）。
`#[test]` は wasm32 では動作せず、`#[wasm_bindgen_test]` が必要。

## 決定

- `core/Cargo.toml` の dev-dependencies に `wasm-bindgen-test = "0.3.79"` を追加。
- wasm32 用 dev-dep に `tokio = { version = "1.40", features = ["macros", "rt"] }`
  を追加（`core/src/ai_markdown/lint.rs` の `#[tokio::test]` のマクロ解決用）。
- `core/src/wasm.rs` の 3 テストを `#[wasm_bindgen_test]` に変更。
- `Cargo.lock` の `openrustbooks-core` deps に `"wasm-bindgen-test"` を追加
  （`--locked` 維持に必須。パッケージ自体は web 側で既にロック済み）。
- `scripts/verify.sh` に次を追加（既存の wasm32 check ゲートは維持）。

```bash
run wasm-test bash -c 'cd core && wasm-pack test --node --locked'
```

- Node 方式を採用する（headless Chrome は chromedriver が必要で、
  環境に未導入のため）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `wasm-pack test --node`（採用） | runner 自動導入、 chromedriver 不要、再現性が高い |
| `wasm-pack test --headless --chrome` | chromedriver 不在で実行不能 |
| 直接 runner ＋ Node | 手動設定が必要で再現性が劣る |
| `cargo check` のみの現状維持 | テストが実行されず、追加テストが死んでいる |

## 影響

- `scripts/verify.sh` が wasm32 テスト 3 件を実際に実行するようになった。
- 新規クレートは追加していない（Cargo.lock の実質変更は 1 行）。
- `lint.rs` の `#[tokio::test]` 2 件は wasm32 でコンパイルは通るが
  runner に登録されず実行されない（既存実装、今回は変更しない）。

## 検証

- `bash -n` / `shellcheck` / `shfmt -d` すべて通過。
- `cargo metadata --locked` 成功。
- `./scripts/verify.sh` 全体が rc=0:
  - native core 165 + desktop 28 + web 2
  - **wasm-test 3 passed / 0 failed**
  - E2E 228 項目すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- CI（共通ワークフロー）への wasm-pack / runner 導入。
- chromedriver を導入してブラウザ上でも実行するか（E2E の Chrome は
  CDP 直接で、wasm-pack は WebDriver 経由で制御プロトコルが異なる）。
- `lint.rs` の `#[tokio::test]` を wasm32 用テストに変換するか。
