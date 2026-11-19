---
type: Decision
title: "ADR-0223: ZIP リーダーの WASM API 公開"
description: "fail-closed な ZIP core を wasm32 向け API として公開し、UI は後段へ分離する決定を記録する。"
tags:
  - ADR
  - wasm
  - archive
generated:
  by: dsh
  at: "2026-11-19T15:14:00+09:00"
status: accepted
---

## 背景

`core/src/archive.rs` の `ArchiveReader` は
fail-closed のパス検証、エントリ数／サイズ制限、暗号化・非対応圧縮の拒否を
備えて native で完成していた（ADR-0004 / 0051）。
一方 wasm 側は `detect_format()` が `"zip"` を返すだけで、
エントリ一覧／読み出し API が無かった（R6 TD-M22）。
UI を一括実装すると XSS / Blob URL 寿命 / CSP の未決定が
EPUB 以上に大きく、段階を分ける必要があった。

## 決定

- `cfg(target_arch = "wasm32")` で `WasmArchive` を公開する。
- `WasmArchive::new(bytes: Vec<u8>)` は
  `ArchiveReader::from_owned(bytes, ArchiveLimits::default())` へ所有権ごと渡す
  （EPUB の `from_owned` と同じく二重コピーを避ける）。
- API は `entries()` / `read_entry(path)` / `read_entry_text(path)` /
  `is_epub()` に限定する。
- `entries()` は camelCase の
  `{path, isDir, size, compressedSize}` 配列を
  `serde_wasm_bindgen` で返す。
- エラーは専用の固定文言を使う:
  - `Invalid ZIP archive`
  - `ZIP entry read failed`
  - `ZIP entry is not valid UTF-8`
  - `ZIP entries unavailable`
- 既存 `js_error()` は流用しない。
  `CoreError::to_string()` は内部型名／パス／ZipError 本文を含むため、
  `map_err(|_| ...)` で破棄する。
- UI / File input / CSP は変更しない（段階 2 の別 ADR）。
- 新規依存なし（`zip` は既に core の依存）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| WASM API のみ（採用） | core の実績を再利用でき、XSS 面を増やさない |
| UI まで一括実装 | EPUB で 4 ADR を要した領域を一度に扱うため却下 |
| 見送り | REQ-002 の複数形式対応が進まない |

## 影響

- wasm-bindgen-test が **7 → 14 件**（新規 7 件）。
- 正常／traversal／破損／UTF-8／権限リーク／制限の fail-closed を検証。
- JS 側から ZIP エントリを列挙・読み出し可能になるが、
  描画はまだ行わない（UI 側の責任は未決定）。
- E2E 253 維持。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo test --all-targets --locked`（core 267 + desktop 79 + web 2）、
  `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core check: すべて成功。
- `wasm-pack test --node --locked`: **14 passed**。
- `./scripts/verify.sh`: 全ゲート成功、E2E **253 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 入力サイズ上限の責任分界（core は呼び出し側責任、Web の事前チェックは未実装）。
- エラー種別の構造化（現状は固定文言で種別を落としている）。
- `is_epub()` による EPUB / ZIP の振り分けと UI 接続（段階 2）。
- 重複パス、入れ子アーカイブ、暗号化エントリの UI 方針。
- MIME 許可リスト、SVG サニタイズ、Blob URL 寿命、ZIP 内 CSS、CSP 緩和範囲。
- `u64` の size が JS の `Number` で精度落ちする可能性（大型エントリは未検証）。
- ブラウザ対応マトリクス（実機 5 ブラウザ未検証）。
