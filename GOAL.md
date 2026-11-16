# Goal

本文エリアの lang 属性を EPUB の言語に合わせ、スクリーンリーダーの読み上げを最適化する。

## Scope

### 背景

ADR-0021 の未決定「スクリーンリーダーでの本文読み上げの最適化」。
`<html lang="ja">` は固定で、UI の日本語には正しいが、EPUB 本文が
英語など別言語の場合、スクリーンリーダーが日本語の発音規則で読み上げて
しまう。core は `metadata.language` を保持しているため、これを本文要素
に反映する。

### 成果物

- `core/src/wasm.rs`: 変更（`language()` getter を追加）
- `web/src/lib.rs`: 変更（本文要素への `lang` 設定）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0057-content-language.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 言語の反映:
   - `WasmEpubBook::language()` で EPUB の言語（例: `ja`, `en`）を返す。
   - 本文要素（`.reader-html`）に `lang` 属性として設定する。
   - 言語が不明な場合は `lang` を設定しない（UI の `<html lang="ja">` が継承される）。
   - 章を切り替えても現在の本の言語が維持される。
   - 別の本を開くと言語が更新される。
2. 既存の挙動を変えない:
   - UI 全体の `<html lang="ja">` は変更しない（UI は日本語固定）。
   - 既存の 121 テスト、E2E 172 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - サンプル EPUB（言語 `ja`）を開くと本文要素に `lang="ja"` が付く。
- 既存の 121 テストと E2E 172 項目が通る。

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
