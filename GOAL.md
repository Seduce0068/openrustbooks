# Goal

章ドロップダウンにタイトル検索フィルターを追加する。

## Scope

### 背景

ADR-0006 の未決定「目次パネルの折りたたみ・検索」。
章数が多い EPUB ではドロップダウンから目的の章を探しにくい。
タイトルで絞り込む入力欄を追加し、検索性を向上する。

### 成果物

- `web/src/lib.rs`: 変更（検索入力、章リストのフィルタリング）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0078-chapter-filter.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 検索:
   - ツールバーに `<input id="reader-chapter-filter">` を置く（aria-label="Filter chapters"）。
   - 入力値で章タイトルを部分一致（大文字小文字無視）で絞り込む。
   - 現在表示中の章は、フィルターに一致しなくても一覧に残す（選択状態を保持）。
   - 空文字のときは全章を表示する。
2. 既存の挙動を変えない:
   - 章移動・履歴・位置復元は従来どおり。
   - 既存の 125 テスト、E2E 206 項目を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - フィルター入力に "Two" と入れると該当章だけが残る。
  - 入力を消すと全章に戻る。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
