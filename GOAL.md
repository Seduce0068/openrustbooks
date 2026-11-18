# Goal

Leptos の非公開 re-export ではなく `send_wrapper` を直接依存として使う（R6 TD-M07）。

## Scope

### 背景

`web/src/lib.rs:6` は `leptos::__reexports::send_wrapper::SendWrapper` を使っている。
これは `#[doc(hidden)]` の re-export で、Leptos 更新時にパスが変わると
コンパイルエラーになる。ADR-0166 / 0186 は解決策を未決定としている。

### 成果物

- `web/Cargo.toml`: 変更（`send_wrapper` 直接依存を追加）
- `Cargo.lock`: 変更（web クレートの依存一覧へ追加、新規クレートなし）
- `web/src/lib.rs`: 変更（import を `send_wrapper::SendWrapper` へ）
- `docs/decisions/0200-send-wrapper-direct-dep.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `send_wrapper` を web クレートの直接依存として追加する（バージョンは Leptos が使う 0.6系に合わせる）。
2. `web/src/lib.rs` の import を公開クレートパスへ変更する。
3. Cargo.lock に新しいクレートが増えないこと（既存 transitive の直接化のみ）。
4. 全ゲート成功（E2E 251 維持）。

## Acceptance criteria

- 全ゲート成功、E2E 251 項目成功。

## Constraints

- unsafe コードを追加しない（`SendWrapper` クレートに委譲）。
- 既存の cleanup 挙動を変更しない。
- `master` へ push する。force push はしない。
