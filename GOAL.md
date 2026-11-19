# Goal

E2E の swipe スタブ二重発火と固定 sleep を条件待ちへ置換する（R6 R4-e2e#3,#4 / C08）。

## Status

**完了**（2026-11-19）。契約の充足と根拠は
[ADR-0235](docs/decisions/0235-e2e-conditional-wait.md) を参照。

## Scope

### 背景

`tools/verify-web-reader.py` の `swipe()` は最初の `cdp.evaluate()` で
引数未埋め込みの TouchEvent を dispatch した後、同じイベントを
実引数付きで再 dispatch していた（二重発火）。
また `asyncio.sleep()` が多数残り、状態変化を条件待ちではなく
固定時間で決めつけていた。

### 成果物

- `tools/verify-web-reader.py`: 変更済み（swipe を 1 回 dispatch、fixed sleep を条件待ちへ）
- `docs/decisions/0235-e2e-conditional-wait.md`: 追加済み
- `GOAL.md` / `todo.md`: 完了契約・完了反映済み

### 必須事項（完了契約）

1. [x] `swipe()` から最初の evaluate（`arguments[0..2]` 参照、二重 dispatch）を削除し、
   f-string 版 1 回のみ dispatch する。`dx` / `dy` / `touches` の意味は維持。
2. [x] 到達条件が明確な箇所（章移動・縦 / マルチタッチ・ダーク切替・ファイル選択後の
   DOM / 保存・リセット後）を `wait_until()` / `wait_swipe_done()` /
   `wait_swipe_ignored()` ベースへ置換する。
3. [x] 負向チェックは baseline カウンタ＋第 1 章本文＋0.3 秒維持確認で判定する。
4. [x] 到達条件のない残り 106 件の sleep は据え置く（無理な置換は flaky 化するため）。
5. [x] `wait_until()` の `RuntimeError` / 一般例外は fail-closed（再送出）を維持
   （ADR-0183）。
6. [x] E2E check 名・期待総数 254 を維持し、全ゲート成功。

## Acceptance criteria

- [x] E2E 254 項目、全ゲート成功（実装エージェント検証: `py_compile` OK、
  AST 237 + 17 = 254 / `Checker(expected=254)`、`verify.sh` 全ゲート 0、E2E 254/0）。
- [x] 文書検証（文書エージェント検証）: AST 254/254、markdownlint 0 issues。

## Constraints

- CDP 接続先照合（ADR-0143）と check 名一意性（ADR-0182）を維持。
- 新規依存を追加しない。
- `master` へ push する。force push はしない（文書エージェントの書き込み範囲外）。
