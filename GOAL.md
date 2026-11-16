# Goal

SVG の XML 処理命令（xml-stylesheet 等）を除去し、外部 CSS 参照による情報漏洩を防ぐ。

## Scope

### 背景

ADR-0044 で SVG の外部スキーム参照を塞いだが、XML 処理命令
（`<?xml-stylesheet href="https://..."?>`）は残っていた（ADR-0044 の未決定）。
Blob URL を直接開くと、処理命令が外部 CSS を読み込み、読者の IP や閲覧時刻が
漏れる可能性がある。

### 成果物

- `web/src/lib.rs`: 変更（XML 処理命令の除去）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0045-svg-xml-processing-instructions.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. XML 処理命令の除去:
   - SVG サニタイズ時に、パース済み Document の子ノードを走査し、
     `PROCESSING_INSTRUCTION_NODE` を削除する。
   - DOM API（`child_nodes()` / `remove_child()`）を使い、新規依存・新規
     web-sys feature を追加しない。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 156 項目、スクリプト・外部参照の除去、
     相対参照の維持、SVG の通常表示を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - SVG から `xml-stylesheet` 処理命令が除去される。
- 既存の 119 テストと E2E 156 項目が通る。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存・新規 web-sys feature を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
