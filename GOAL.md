# Goal

Web 版 EPUB リーダーで、章内のスクロール位置も読書位置として保存し、同じ本を
開き直したときに復元する。

## Scope

### 背景

ADR-0015 は章とフラグメントを保存・復元するようにしたが、章内のスクロール
位置は保存しないため、長い章では毎回先頭から読み直すことになる。
ADR-0015 の未決定「章内のスクロール位置（ピクセル）の保存と復元」を実装する。

### 成果物

- `web/src/lib.rs`: 変更（スクロール位置の保存・復元）
- `tools/verify-web-reader.py`: 変更（スクロール位置の E2E 検証を追加）
- `docs/decisions/0016-scroll-position.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 保存:
   - `SavedPosition` に `scrollTop` を加える（既存の保存値に無くても 0 として読む）。
   - `.reader-content` のスクロールで、200ms のデバウンス後に
     `{chapterIndex, fragment: null, scrollTop}` を保存する。
   - 保存する値は章の index とスクロール位置だけ（本文・内容は保存しない）。
   - 章が切り替わった後に古い章のスクロール保存が遅れて書き込まれないようにする。
2. 復元:
   - 保存位置にフラグメントがあればその位置へスクロールする（従来どおり）。
   - フラグメントが無く `scrollTop > 0` なら、描画後に `.reader-content` を
     その位置へスクロールする。
   - 復元後のスクロール位置を保存値へ書き戻す。
3. 章の切り替え:
   - 目次・前後移動・相対リンクで章を切り替えたときは先頭（またはフラグメント）へ
     スクロールし、保存値も新しい章の位置にする。
4. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 28 項目、サニタイズ、画像、リンク、サイズ上限を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 長い章でスクロールすると localStorage の `scrollTop` が更新される。
  - リロードして同じ本を開くとスクロール位置が復元される。
  - フラグメント付きの位置は従来どおりフラグメントへスクロールする。
  - 章を切り替えると先頭へ戻り、保存値も先頭になる。
  - `scrollTop` の無い旧形式の保存値でも章が復元される。
- 既存の 119 テストと E2E 28 項目が通る。

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

- 新規依存を追加しない。保存先は localStorage のみ。外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
