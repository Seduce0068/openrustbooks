# Goal

AI 送信前にプレビューとネイティブ同意ダイアログを要求する（todo.md D06 の本体）。

## Scope

### 背景

既定無効化（ADR-0099）、service ガード（ADR-0100）、送信先制限（ADR-0101）により
リスクは減ったが、利用者が AI を有効にした後は、どの本文がどこへ送られるかを
確認せずに送信できる。設定 IPC が公開済みで UI 未接続のため、送信前に
Rust 側で同意を確認する必要がある。

### 成果物

- `desktop/src/commands/ai.rs`: 変更（同意チェック、プレビューコマンド）
- `docs/decisions/0104-send-consent-dialog.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. 本文を送る 4 コマンド（generate / refine / to_table / front_matter）に
   `AppHandle` を追加し、送信直前に Rust 側でネイティブ確認ダイアログを表示する。
   JS から `consent: bool` を受け取らない（侵害された renderer が true を渡せるため）。
2. `tauri-plugin-dialog` の `app.dialog().message(...).show() + oneshot` を使い、
   async コマンド内で結果を待つ（blocking_show は使わない）。
3. ダイアログ本文に、送信字段ごとの「文字数＋先頭 80 文字」の要約を表示する
   （別の preview コマンドは追加しない）。本文全体・API キーは出さない。
4. Tauri の command 登録（`main.rs`）は変更しない（`AppHandle` は自動注入）。
5. 新規依存を追加しない（`tauri-plugin-dialog` は既存依存）。
6. 6 ゲート・E2E が成功する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- Web UI の配線は変更しない（未接続のまま）。
- 送信先制限・service ガード・秘密マージは変更しない。
- `master` へ push する。force push はしない。
