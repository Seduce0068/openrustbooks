# Goal

Web 版 EPUB リーダーに、章と位置の移動履歴（戻る・進む）を追加する。

## Scope

### 背景

ADR-0016 までで章・フラグメント・スクロール位置の保存と復元はできるが、
読書中の移動を戻る手段が無い。目次や章内リンクで移動した後、前の位置へ
戻るには目次から探し直す必要がある。ADR-0015・ADR-0016 の未決定
「戻る・進むの履歴」を実装する。

### 成果物

- `web/src/lib.rs`: 変更（履歴スタック、戻る・進むボタン）
- `tools/verify-web-reader.py`: 変更（履歴の E2E 検証を追加）
- `docs/decisions/0017-navigation-history.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 履歴:
   - 章を移動する操作（目次選択・前後章・相対リンクの章移動）で、移動先を
     履歴に積む。移動前の位置（章・スクロール位置）は履歴の現在エントリへ
     記録する。
   - 履歴は最大 50 件とし、古いものから捨てる。
   - 戻る・進むは履歴上の位置へ移動し、スクロール位置も復元する。
   - 戻った後に新しい移動をすると、進む方向の履歴は捨てる。
   - 本を開き直したら履歴をリセットし、初期位置を 1 件だけ積む。
   - 履歴はメモリ上だけで保持し、localStorage へは保存しない。
2. UI:
   - ツールバーに `Back` / `Forward` ボタンを置き、移動できないときは
     `disabled` にする。
   - ボタンには `id="reader-back"` / `id="reader-forward"` を付ける（E2E 用）。
3. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 34 項目、サニタイズ、相対リンク、読書位置の
     保存・復元を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 開いた直後は Back / Forward が disabled。
  - 章を移動すると Back が有効になる。
  - Back で前の章へ戻り、Forward で元の章へ進む。
  - Back で戻った章のスクロール位置が復元される。
  - 戻った後に新しい移動をすると Forward が disabled。
  - 本を開き直すと履歴がリセットされる。
- 既存の 119 テストと E2E 34 項目が通る。

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

- 新規依存を追加しない。履歴はメモリ上のみで、localStorage へ保存しない。
- 外部送信を追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
