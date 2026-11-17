# Goal

IME 変換中はキーボードショートカットを発火させない（todo.md B04 / C01 の一部）。

## Scope

### 背景

`key_handler` は `isComposing` を確認していないため、日本語入力の変換確定時に
`d` や `[` / `]` などのキーがショートカットとして扱われ、意図せず章移動や
表示切り替えが起こる可能性がある。

### 成果物

- `web/src/lib.rs`: 変更（`event.is_composing()` のガード追加）
- `tools/verify-web-reader.py`: 変更（E2E 検証を追加）
- `docs/decisions/0092-ime-composing-guard.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `key_handler` の先頭で `event.is_composing()` が true なら return する。
2. E2E で、composition 中を模したキーイベントではショートカットが
   発火しないことを確認する。
3. 既存の 221 項目を変えない（期待値を 222 に更新）。

## Acceptance criteria

- 6 ゲート・E2E が成功する。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- `master` へ push する。force push はしない。
