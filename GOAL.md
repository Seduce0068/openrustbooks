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
   `consent: bool` を追加し、false なら送信前に拒否する（fail-closed）。
2. JS の bool だけでは同意を担保できないため、可能なら
   `tauri-plugin-dialog` で Rust 側からネイティブ確認を表示し、結果を待つ。
3. 送信内容をローカルで合成する preview コマンドを追加する
   （AI 呼び出し・ネットワークなし）。
4. Tauri の command 登録（`main.rs`）は変更しない。
5. 新規依存を追加しない（`tauri-plugin-dialog` は既存依存）。
6. 6 ゲート・E2E が成功する。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- Web UI の配線は変更しない（未接続のまま）。
- 送信先制限・service ガード・秘密マージは変更しない。
- `master` へ push する。force push はしない。
