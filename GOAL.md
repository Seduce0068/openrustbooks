# Goal

SVG 内の外部リソース参照を除去し、Blob URL 直接オープン時のプライバシー漏洩を防ぐ。

## Scope

### 背景

ADR-0043 で SVG のスクリプト実行経路を塞いだが、
`<use href="http://...">` や `<image href="https://...">` などの外部参照は残って
いた（ADR-0043 の未決定「`<use href>` / `<image href>` による外部リソース
参照の制御」）。Blob URL を直接開くと、これらの参照がネットワーク要求を
発生させ、読者の IP や閲覧時刻が外部サーバに漏れる可能性がある。

### 成果物

- `web/src/lib.rs`: 変更（外部スキーム参照の除去、コメント重複の修正）
- `tools/verify-web-reader.py`: 変更（外部参照除去の E2E 検証を追加）
- `docs/decisions/0044-svg-external-references.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 外部参照の除去:
   - SVG サニタイズ時に、`href` / `xlink:href` の値が外部スキーム
     （`http:` / `https:` / `data:` / `blob:` / `file:`）で始まる場合、
     その属性を削除する。
   - `javascript:` は既存の判定で除去済み（維持する）。
   - 相対参照（EPUB 内の画像など）は維持する。
   - 判定は空白・制御文字を除去した正規化済み文字列に対して行う
     （ADR-0043 と同じ回避策）。
2. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 154 項目、スクリプト除去、
     SVG の通常表示、読書位置、履歴、表示設定、カラースキーム追従、
     強制カラーモード対応を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - SVG から http/https/data/blob/file の外部参照が除去される。
  - 相対参照は維持される。
- 既存の 119 テストと E2E 154 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存・新規 web-sys feature を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
