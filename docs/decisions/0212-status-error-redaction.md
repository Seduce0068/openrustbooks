---
type: Decision
title: "ADR-0212: Reader エラー status の redaction"
description: "内部エラーの Debug 表現を status と console から排除し、操作別の固定文言のみを表示する決定を記録する。"
tags:
  - ADR
  - web
  - security
generated:
  by: dsh
  at: "2026-11-19T11:58:00+09:00"
status: accepted
---

## 背景

`web/src/lib.rs` のファイル読込／EPUB オープン／章一覧／章描画／リンク解決の
6 経路が `format!("... {error:?}")` で内部エラーを status に表示していた
（R6 TD-M12）。`CoreError` の文字列にはアーカイブ内パスや実装詳細が
含まれ得ず、ADR-0098 / 0103 / 0154 の redaction 方針と一貫しなかった。

## 決定

- 6 経路の status を操作別の固定文言へ置換する:
  - File read: `File read failed. Please try again.`
  - EPUB open: `EPUB open failed. The file may be unsupported or damaged.`
  - Chapter list: `Chapter list could not be read.`
  - Chapter render / resolve_images: `Chapter could not be displayed.`
  - Link resolution: `Link resolution failed.`
- `report_reader_error(operation)` を追加し、console.error も
  `openrustbooks: <操作名>` の固定文言のみにする。
- 生エラーは `Err(_)` で捨て、Debug / `to_string()` / 型名 / パス / URL /
  本文を status と console のどちらにも出力しない。
- 既存 E2E 契約である `This link cannot be opened.` は変更しない
  （link=None の別分岐）。
- helper と定数は `#[cfg(target_arch = "wasm32")]` とし、
  native の clippy `dead_code` を回避する。
- E2E を 1 件追加し、期待総数 251 → **252**（check 235 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 操作別固定文言（採用） | 機微情報の漏洩を構造的に防げる |
| `{error}` / `to_string()` に置換 | 内部パスが残るため不採用 |
| console へ詳細を出す | ADR-0154 の redaction 方針に反する |

## 影響

- エラー原因の詳細は利用者に見えない（トレードオフ）。
- 開発時の診断は固定文言のみのため、
  詳細調査には wasm デバッガ等が必要（未決定）。
- `docs/e2e-traceability.md` は 251 固定のままで、
  今回の E2E 252 件の反映は未実施（ADR-0206 の未決定として継続）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `grep -n '{error:?}' web/src/lib.rs` → 0 件。
- E2E「不正 EPUB は固定文言のみ通知しサンプル EPUB は通常どおり表示される」が ok。
  - 固定 status / console の固定文言。
  - `Err(` / `JsValue` / 型名 / ファイル名 / 本文の非含有。
  - サンプル EPUB の再表示。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、wasm-web check、
  `./scripts/verify.sh`: すべて成功。
- E2E **252 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `docs/e2e-traceability.md` の 252 件への更新と VC 割当て。
- Editor / AI / config 等、他エラー経路への横展開。
- 通知の寿命と分類粒度（操作別 vs エラー種別別）。
- 開発用詳細ログの別経路（デバッグビルド限定等）の要否。
