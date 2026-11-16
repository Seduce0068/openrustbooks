# Goal

EPUB 内の SVG 画像を Blob URL 化する前にスクリプト実行要素を除去し、直接オープン時のリスクを減らす。

## Scope

### 背景

ADR-0013 で SVG 画像は Blob URL で表示する判断をしたが、
「blob URL を直接開いた場合のスクリプト実行余地」が未決定として残っていた。
`<img>` 文脈ではスクリプトは実行されないが、Blob URL を新しいタブで開くと
アプリのオリジンで SVG が評価され、`<script>` や `on*` 属性が実行される
可能性がある。

### 成果物

- `web/src/lib.rs`: 変更（SVG バイト列のサニタイズ）
- `tools/verify-web-reader.py`: 変更（SVG サニタイズの E2E 検証を追加）
- `docs/decisions/0043-svg-blob-sanitization.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. SVG サニタイズ:
   - MIME が `image/svg+xml` のリソースについて、Blob URL を作る前に
     SVG を `image/svg+xml` としてパースし、以下を除去する。
     - `<script>` 要素
     - `on*` イベントハンドラ属性
     - `<foreignObject>` 要素（HTML スクリプトの混入経路）
     - `href` / `xlink:href` の `javascript:` スキーム
   - サニタイズに失敗した場合は Blob URL を作らず、`src` を削除する
     （fail-closed）。
   - パース・シリアライズは DOM API（`DomParser` / `outerHTML`）を使い、
     新規依存・新規 web-sys feature を追加しない。
2. 既存の挙動を変えない:
   - PNG / JPEG / GIF / WebP などの非 SVG 画像は一切変更しない。
   - 既存の 119 テスト、E2E 147 項目、読書位置、履歴、表示設定、日時表示、
     確認ダイアログ、カラースキーム追従、画像の非同期デコード、
     スクロール順序テスト、強制カラーモード対応を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - スクリプトを含む SVG がスクリプトなしで表示されること。
  - 通常の SVG が従来どおり表示されること。
- 既存の 119 テストと E2E 147 項目が通る。

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
