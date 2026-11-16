# Goal

本文領域のアクセシブル名を章タイトルに結びつける。

## Scope

### 背景

ADR-0057 の未決定「ページ見出しへの `aria-labelledby` 設定など、さらに詳細な本文構造の提供」。
`.reader-content` は `aria-label="Chapter content"` のみで、スクリーンリーダーでは
どの章を読んでいるか分かりにくい。ツールバーの章タイトルを有効なラベルにする。

### 成果物

- `web/src/lib.rs`: 変更（章タイトルへの id 付与、`aria-labelledby` 追加）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0058-region-aria-labelledby.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. ラベルの結びつけ:
   - 章タイトルの `<span>` に `id="reader-chapter-title"` を付ける。
   - `.reader-content` に `aria-labelledby="reader-chapter-title"` を追加する。
   - 既存の `aria-label="Chapter content"` は後方互換のため維持する
     （ARIA 仕様で `aria-labelledby` が `aria-label` より優先される）。
2. 既存の挙動を変えない:
   - 既存の 121 テスト、E2E 174 項目を変えない（aria-label の検証を含む）。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - `.reader-content` の `aria-labelledby` が `reader-chapter-title` を指す。
  - 参照先の要素に章タイトルが入っている。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
