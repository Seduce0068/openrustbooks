# Goal

OS の prefers-reduced-motion を尊重する（R6 TD-M13、ADR-0213）。

## Scope

### 成果物

- `web/index.html`: 色 transition 維持、空間移動 no-preference opt-in
- `web/src/lib.rs`: prefers_reduced_motion、再試行 skip、後始末維持
- `tools/verify-web-reader.py`: E2E 1 件、252 → 253
- `docs/decisions/0213-reduced-motion.md`: 追加
- `todo.md`: TD-M13 を完了へ移動

### 決定

- transition:none / 0.01ms は禁止、色 transition は維持。
- 画像セトリング待ちは残し、スクロール再試行のみ reduce で skip。

## Acceptance criteria

- E2E 253 項目、全ゲート成功（検証済み）。
