# Goal

エディタ下書きをデバウンス自動保存し、未保存状態を可視化する（R6 TD-H05）。

## Scope

### 背景

現在は `Save draft` クリック時のみ localStorage へ保存するため、
画面遷移・リロード前に保存を忘れると本文を失う。
ADR-0164 の「明示保存のみ」方針を、利用者を守る範囲で拡張する。

### 成果物

- `web/src/lib.rs`: 変更（デバウンス自動保存、未保存表示、beforeunload ガード）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、247 → 248）
- `docs/decisions/0179-editor-autosave.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `on:input` 後に 800ms デバウンスで `save_editor_draft()` を呼ぶ。
2. 保存成功時は `.editor-status` に「下書きを自動保存しました」を表示。
3. 入力直後は「未保存の変更があります」を表示し、自動保存成功で上書きする。
4. 上限超の入力は自動保存対象外とし、固定文言「下書きが大きすぎて保存できません」を表示（手動 Save と同じ）。
5. 保存失敗時は ADR-0165 の共通通知に任せ、成功文言を出さない。
6. `beforeunload` で未保存変更がある場合のみ離脱警告を出す（E2E の Page.reload を壊さないよう、保存済みなら出さない）。
7. E2E に 1 件追加（入力後、Save クリックなしで待機→リロード→本文復元）。期待総数 247 → 248。
8. 全ゲート成功。

## Acceptance criteria

- E2E 248 項目、全ゲート成功。

## Constraints

- 新規依存を追加しない（gloo timers / web_sys timeout を利用）。
- status は `aria-live="polite"` の既存領域を使い、デバウンスで読み上げ過多を避ける（modern-web-guidance: accessibility）。
- 本文・サイズ値は status / console へ出さない。
- `master` へ push する。force push はしない。
