# Goal

Reader の status に内部エラーの Debug 表現が流出する問題を修正する（R6 TD-M12）。

## Scope

### 背景

`web/src/lib.rs` のファイル読込／EPUB オープン／章一覧／リンク解決のエラー経路が
`format!("... {error:?}")` で status に表示している。
内部エラー文字列にはパスや実装詳細が含まれ得ず、
ADR-0098 / 0103 / 0154 の redaction 方針と一貫しない。

### 成果物

- `web/src/lib.rs`: 変更（`{error:?}` の固定文言化、详细は console.error へ）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、251 → 252）
- `docs/decisions/0212-status-error-redaction.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. 5 箇所の `{error:?}` status を操作別の固定文言へ置換する。
2. 内部エラー詳細は `console.error` にのみ出力する（本文・パスはログにも出さない方針なら固定種別のみ）。
3. 既存の E2E「解決できないリンクは移動せず状態を表示する」等を壊さない。
4. E2E を 1 件追加（不正 EPUB 読込時に固定文言が出され、Debug 表現やパスが出ない）。
   期待総数 251 → 252。
5. 全ゲート成功。

## Acceptance criteria

- E2E 252 項目、全ゲート成功。

## Constraints

- エラー文言にファイル名・パス・本文・URL を出さない。
- 既存の Reader operation は変更しない。
- `master` へ push する。force push はしない。
