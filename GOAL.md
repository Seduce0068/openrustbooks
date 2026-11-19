# Goal

容量超過時の読書位置削除を利用者へ通知する（R6 R3-config#5）。

## Scope

### 背景

`save_position()` は localStorage の容量超過時に古い位置を
`POSITION_MIN_KEEP = 10` / `1` まで削除して再試行するが、
成功しても削除件数・残件数を通知していなかった。
全失敗時も「保存に失敗しました」とのみ表示し、
不可逆な削除が起きた事実が隠れていた。

### 成果物

- `web/src/lib.rs`: 変更（容量超過時の削除件数・残件数通知）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、254 → 255）
- `docs/decisions/0245-position-prune-notification.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. 容量超過で古い位置を削除して再保存に成功した場合、
   削除件数と残件数を status で通知する。
2. 削除後も保存失敗の場合も、削除件数と残件数を通知する。
3. キー名・ファイル名・生エラーは status / console へ出さない（数値のみ）。
4. 既存の通常 prune（90 日 / 40 件）は変更しない。
5. E2E 1 件追加（削除件数・残件数の表示と機微情報非含有）、期待総数 254 → 255。
6. 全ゲート成功。

## Acceptance criteria

- E2E 255 項目、全ゲート成功。

## Constraints

- 退避先（IndexedDB / 自動ダウンロード）は今回実装しない。
- aria-live 構造は変更しない（ADR-0021 / 0226 を維持）。
- `master` へ push する。force push はしない。
