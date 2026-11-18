# Goal

E2E `wait_until()` の Python 例外を握り潰さず再送出する（R6 TD-M02、ADR-0183）。

## Scope

### 成果物

- `tools/verify-web-reader.py`: `except Exception: pass` → `raise`
- `docs/decisions/0183-wait-until-failclosed.md`: 追加
- `todo.md`: TD-M02 を完了へ移動

### 決定

- RuntimeError 明示再送出（ADR-0144）は維持。
- Cdp.evaluate の JS 例外検査は本次対象外（一時 TypeError で 249 件が壊れる）。
- 根本課題は ADR の未決定として記録。

## Acceptance criteria

- E2E 249 項目、全ゲート成功（検証済み）。
