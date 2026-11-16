# Goal

wasm で EPUB を開くときのアーカイブバイト列の二重コピーを解消する。

## Scope

### 背景

ADR-0013 の未決定「読み込み時のメモリコピー（`ArrayBuffer`・`Vec`・
`ArchiveReader` の複製）」。調査の結果、`WasmEpubBook::new` が
`Vec<u8>` を受け取った後、`EpubBook::open(&bytes)` →
`ArchiveReader::with_limits(bytes, ...)` が `bytes.to_vec()` で
アーカイブ全体を二重にコピーしている（256 MiB の EPUB なら 256 MiB の
余分なコピー）。

### 成果物

- `core/src/archive.rs`: 変更（所有権を受け取る `from_owned` を追加）
- `core/src/epub.rs`: 変更（`open_owned` を追加）
- `core/src/wasm.rs`: 変更（`WasmEpubBook::new` で `open_owned` を使う）
- `docs/decisions/0051-owned-epub-open.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 最適化:
   - `ArchiveReader::from_owned(bytes: Vec<u8>, limits)` を追加し、
     既存の `with_limits` はこれを呼ぶ形にする（挙動は同一）。
   - `EpubBook::open_owned(bytes: Vec<u8>)` を追加し、
     `WasmEpubBook::new` はこれを使う（1 回のコピーを削減）。
2. 既存の挙動を変えない:
   - `open` / `with_limits` の既存 API と挙動は維持する。
   - 既存の 120 テスト、E2E 166 項目を変えない。
   - 追加した API が既存 API と同じ結果を返すテストを追加する。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
