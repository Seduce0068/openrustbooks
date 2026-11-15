# Goal

読書位置の保存データに上限と寿命を設け、古いキーを自動で掃除する。

## Scope

### 背景

ADR-0018 までで読書位置・表示設定を localStorage に保存するようにしたが、
保存件数の上限が無く、本を読み続けると `openrustbooks:position:` のキーが
増え続ける。localStorage の容量は有限で、上限に達すると保存が失敗する。
ADR-0016・ADR-0018 の未決定「保存値の削除 UI、保存件数の上限、古いキーの
掃除」のうち、自動掃除を実装する。

### 成果物

- `web/src/lib.rs`: 変更（保存時刻、件数上限、寿命、容量超過時の再試行）
- `tools/verify-web-reader.py`: 変更（掃除の E2E 検証を追加）
- `docs/decisions/0019-position-retention.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 必須事項

1. 保存時刻:
   - `SavedPosition` に `savedAt`（Unix ミリ秒、`f64`）を加える。旧形式の
     保存値に無い場合は 0 として読む（`serde(default)`）。
   - 保存時に現在時刻を書き込む。
2. 掃除（保存のたびに実行する）:
   - 寿命: `savedAt` が 90 日より古いエントリを削除する。
   - 件数: 40 件を超える場合、`savedAt` の新しい順に 40 件を残す。
   - `savedAt` が 0 の旧形式エントリは最も古いものとして扱う。
   - 現在開いている本のエントリは必ず残す。
   - 読書位置以外のキー（`openrustbooks:settings`）は触らない。
3. 容量超過:
   - `set_item` が失敗した場合、10 件まで減らして 1 回だけ再試行する。
   - 再試行も失敗した場合は黙って諦め、読書を妨げない。
4. 既存の挙動を変えない:
   - 既存の 119 テスト、E2E 58 項目、復元、履歴、表示設定を変えない。

### 必須テスト

- `tools/verify-web-reader.py` に追加する。
  - 保存値に `savedAt` が入り、現在時刻に近い。
  - 45 件の偽エントリを入れてから保存すると 40 件以下になり、現在の本が残る。
  - 90 日より古いエントリが保存時に削除される。
  - 90 日以内の新しいエントリは残る。
  - `openrustbooks:settings` は掃除の対象にならない。
  - `savedAt` の無い旧形式の保存値でも章が復元される（既存項目）。
- 既存の 119 テストと E2E 58 項目が通る。

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
