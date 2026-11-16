# Goal

非整形式 XHTML の扱いを調査し、ADR-0005 の未決定を解消する。

## Scope

### 背景

ADR-0005 の未決定「非整形式の XHTML を許容するかどうかの判断（現状は fail-closed）」。
調査の結果、`EpubBook::chapter_html` は内容をそのまま返し XML 検証をせず、
Web 側の `sanitize_epub_html`（ammonia）が HTML としてサニタイズしている。
つまり OPF / nav は XML として fail-closed だが、章本文は HTML パーサーとして
寛容に扱われている。この実際の挙動をテストで確認し ADR に記録する。

### 成果物

- `core/src/epub.rs`: 変更（調査用テストを追加）
- `docs/decisions/0059-malformed-xhtml-policy.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 調査:
   - 非整形式 XHTML（閉じタグ欠落）の章を持つ EPUB を開き、
     `chapter_html` が内容を返すことを確認する。
   - OPF が非整形式 XML の場合は `EpubBook::open` がエラーを返す
     （fail-closed）ことを確認する。
2. 既存の挙動を変えない:
   - プロダクションコードは変更しない（テスト追加のみ）。
   - 既存の 121 テスト、E2E 176 項目を変えない。

## Acceptance criteria

- 実際の挙動がテストで確認され、ADR に記録される。
- 6 ゲートすべて成功する。

## Constraints

- 新規依存を追加しない。
- プロダクションコードを変更しない。
- `master` へ push する。force push はしない。
