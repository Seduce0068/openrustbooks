# Goal

OS の `prefers-reduced-motion` 設定を尊重する（R6 TD-M13）。

## Scope

### 背景

`web/index.html` は color / all の transition を無条件に適用し、
Reader のフラグメント移動は `scrollIntoView` と段階的リトライでスクロールする。
ADR-0145 の縦書き検討時から reduced-motion 未対応が残っている（TD-M13）。

modern-web-guidance (`motion`) では、
空間移動／scale／rotation は `prefers-reduced-motion: no-preference` で opt-in し、
色や背景色の非空間 transition は維持する方針を推奨する。

### 成果物

- `web/index.html`: 変更（空間 transition を no-preference へ、color transition は維持）
- `web/src/lib.rs`: 変更（reduce 時はスクロールを即時移動へ）
- `tools/verify-web-reader.py`: 変更（E2E 1 件、252 → 253）
- `docs/decisions/0213-reduced-motion.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. CSS の `transition: all 0.2s` を空間プロパティのみ
   `@media (prefers-reduced-motion: no-preference)` 下へ移す。
2. color / background-color の transition は reduce 時も維持する。
3. Reader のフラグメント／位置復元で reduce を検出した場合、
   リトライループや smooth scroll を使わず即時に目的地へ移動する。
4. E2E に 1 件追加（Emulation.setEmulatedMedia reduce で CSS / 即時移動を検証）、
   期待総数 252 → 253。
5. 全ゲート成功。

## Acceptance criteria

- E2E 253 項目、全ゲート成功。

## Constraints

- 新規依存を追加しない。
- 既存のスクロール位置保存・復元契約を壊さない。
- `master` へ push する。force push はしない。
