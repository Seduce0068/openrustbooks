---
type: Decision
title: "ADR-0251: アーカイブ実展開サイズのセッション予算"
description: "ZIP エントリの実展開バイトを AtomicU64 で累計し、宣言サイズ合計では防げない反復読み出しを制限する決定を記録する。"
tags:
  - ADR
  - archive
  - security
generated:
  by: dsh
  at: "2026-11-19T23:13:00+09:00"
status: accepted
---

## 背景

`ArchiveLimits::max_total_size` はセントラルディレクトリの宣言サイズ合計のみを
検査していた（ADR-0004 / 0240）。`read_entry()` は 1 回あたり
`max_entry_size` で打ち切るが、`&self` で何度でも呼べるため、
同一 64 MiB エントリの反復読み出しで実展開の累計が上限を超え得た。
`WasmArchive` が `read_entry` を JS へ公開しているため（ADR-0223）、
宣言値だけでは不十分だった（R6 R1-archive#2）。

## 決定

- `ArchiveReader` に非公開の `expanded: AtomicU64` を追加する。
- 公開 API（シグネチャ、`ArchiveLimits` のフィールド、`Send + Sync`）は変更しない。
- `read_entry_with_limit()` の前に、
  予約額 `min(declared_size + 1, limit + 1)` を
  `try_update()`（旧 `fetch_update`、MSRV 1.95 で安定）＋ `checked_add()`
  の CAS で予約する。
- 上限超過／オーバーフロー時は固定文言
  `"アーカイブの累計展開サイズが上限を超えています"`
  の `CoreError::Validation` を返す（path を含めない）。
- 読み出し本体を `read_entry_body()` へ分離し、
  暗号化・非対応圧縮・read エラー・実バイト超過の全経路で予約を返金する。
- 成功時は実 `data.len()` で精算し、差額を返金する。
  実バイトが予約を超えた場合は再チェックし、
  上限超過なら全額返金＋Validation で fail-closed にする。
- 同一 path の再読は予算が残る限り許可し（再課金）、
  枯渇後のみ拒否する。ADR-0247 / 0250 の EPUB 経路で
  `chapter_text()` → `chapter_html()` → `toc()` の連続呼び出しが壊れないことを
  test で固定する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| `AtomicU64` セッション予算（採用） | `&self` / `Send + Sync` / wasm / 並行呼び出しを全部維持できる |
| `&mut self` 化 | EPUB / wasm の多数の `&self` 経路を破壊 |
| `Mutex` | 追加状態管理とロック競合が不要 |
| 宣言合計のみ（現状） | 反復読み出しで累計を制限できない |

rustc 1.100 では `fetch_update` が `try_update` へリネーム／非推奨化された。
実装は同一 CAS セマンティクスの `try_update` を用い、
`clippy -D warnings` を通過させた。

## 影響

- `core/src/archive.rs` のみ変更（20 unit test、うち 5 件新規）。
- `is_epub()` の 64 バイト probe も予算に計上される
  （既定 256 MiB では無視できる）。
- 正常な EPUB の再読は予算内で従来どおり成功。
- E2E 255 維持。

## 検証（Runtime Proof）

- `cargo test --all-targets --locked`: core **284** / desktop 79 / web 2。
- new tests: 再読枯渇拒否、CRC 破壊時の返金、上限内複数エントリ、
  EPUB 連続再読、8 スレッド並行で成功 1 件（CAS 検証）。
- `wasm-pack test --node --locked`: 15 passed。
- `cargo clippy --all-targets --locked -- -D warnings`、
  wasm32 core / web check、`./scripts/verify.sh`: すべて成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 256 MiB の実測根拠（ADR-0190 の暫定値を継続）。
- 予約の `+1` ヘッドルームにより、
  宣言合計が `max_total_size` とちょうど等しい archive の
  最後の読み出しが拒否され得る点（既定値・fixture では非該当）。
- `is_epub()` probe を予算へ計上する契約の明文化。
- 入れ子アーカイブを開いた場合の予算継承
  （現状は `ArchiveReader` インスタンス単位）。
