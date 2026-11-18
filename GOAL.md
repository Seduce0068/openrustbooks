# Goal

SPA 再マウント時のイベントリスナー蓄積を解消する（ADR-0165 の未決定）。

## Scope

### 背景

`web/src/lib.rs` の save-error（Editor / Reader）、画像 load、
prefers-color-scheme change の各 Closure が `forget()` でページ寿命保持。
SPA 遷移でコンポーネントが再マウントされるたびに window / document / mq へ
リスナーが蓄積し、古い signal への set が残る（ADR-0165 の未決定）。

### 成果物

- `web/src/lib.rs`: 変更（on_cleanup による remove_event_listener、または Singleton 化）
- `docs/decisions/0166-listener-cleanup.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. save-error / image load / prefers-color-scheme の各リスナーについて、
   再マウント時に古いリスナーが残らないようにする。
2. Leptos 0.8 の `on_cleanup()` が利用可能なら登録解除に使う
   （利用不可なら Singleton 化など代替を選択し根拠を記録）。
3. 既存 E2E 241 項目と全ゲートを壊さない。

## Acceptance criteria

- 全ゲート成功、E2E 241 項目成功。

## Constraints

- 新規依存を追加しない。
- イベント名・通知文言は変更しない。
- `master` へ push する。force push はしない。
