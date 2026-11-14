# Goal

Web 版 EPUB リーダーで、本ごとの読書位置（章とフラグメント）を localStorage に
保存し、同じ本を開き直したときに復元する。

## Scope

### 背景

ADR-0014 は本文内の相対リンクで章を移動できるようにしたが、リーダーを開き
直すたびに先頭の章へ戻る（ADR-0012・ADR-0014 の未決定）。読書位置の保存は
「対応サンプルを開き、目次・ページ・章を移動できる」体験を継続するために要る。
保存先はブラウザの localStorage とし、外部送信は行わない。

### 成果物

- `web/src/lib.rs`: 変更（読書位置の保存と復元）
- `tools/verify-web-reader.py`: 変更（復元の E2E 検証を追加）
- `docs/decisions/0015-reading-position.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 保存:
   - 本の識別子は `File.name()`・`File.size()`・`File.lastModified()` から作る。
   - localStorage のキーは
     `openrustbooks:position:<name>:<size>:<lastModified>`。
   - 値は JSON `{"chapterIndex": N, "fragment": "..." | null}`。
   - 章を表示するたびに保存する（目次・前後移動・相対リンク・復元を含む）。
   - 同じ章内のフラグメント移動でもフラグメントを保存する。
   - localStorage が使えない・値が壊れている場合は黙って諦め、表示を妨げない。
2. 復元:
   - 本を開いたとき、保存された位置があり `chapterIndex` が章数の範囲内なら
     その章を表示する。範囲外なら先頭の章にする。
   - 復元時もフラグメントがあればスクロールする。
   - 保存が無い本は従来どおり先頭の章。
3. プライバシー:
   - 保存するのは章の index とフラグメントのみ。ファイル内容・本文は保存しない。
   - 外部送信を追加しない。新規依存を追加しない。
4. 既存の挙動を変えない:
   - 既存の 119 テストが通る。core の公開 API を変えない。
   - サニタイズ、画像解決、相対リンク移動、サイズ上限を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に復元の検証を追加する。
  - 保存が無い本を開くと先頭の章になる。
  - 章を移動すると localStorage に位置が保存される。
  - リロードして同じ本を開くと、移動した章が復元される。
  - フラグメント付きで移動した位置が復元され、スクロールする。
  - 範囲外の保存値では先頭の章になる。
- 既存の 119 テストと既存の E2E 項目が通る。

## Acceptance criteria

- 上記の必須事項がコードと E2E で確認できる。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --locked`
  - `cargo test --all-targets --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  - `trunk build --offline --dist <一時出力先>`（web）
  - `python3 tools/verify-web-reader.py --dist <出力先>`（E2E）
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## Constraints

- 新規依存を追加しない。保存先は localStorage のみ。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
