# Goal

E2E の swipe スタブ二重発火と固定 sleep を条件待ちへ置換する（R6 R4-e2e#3,#4 / C08）。

## Scope

### 背景

`tools/verify-web-reader.py` の `swipe()` は最初の `cdp.evaluate()` で
引数未埋め込みの TouchEvent を dispatch した後、同じイベントを
実引数付きで再 dispatch していた（二重発火）。
また `asyncio.sleep()` が多数残り、状態変化を条件待ちではなく
固定時間で決めつけていた。

### 成果物

- `tools/verify-web-reader.py`: 変更（swipe を 1 回 dispatch、fixed sleep を条件待ちへ）
- `docs/decisions/0235-e2e-conditional-wait.md`: 追加
- `GOAL.md` / `todo.md`: 契約・完了反映

### 必須事項

1. `swipe()` から最初の evaluate（二重 dispatch）を削除し、1 回だけ dispatch する。
2. 引数は f-string で整数を埋め込み、既存の swipe 意味（dx / dy / touches）を維持する。
3. 状態変化を待つ固定 sleep を `wait_until()` ベースへ置換する（全 27 件を無理に置換せず、swipe 周辺と明確に状態依存な箇所を対象とする）。
4. `wait_until()` の RuntimeError / その他の例外は fail-closed を維持する（R6 TD-M02）。
5. E2E check 名・期待総数 254 を維持し、全ゲート成功。

## Acceptance criteria

- E2E 254 項目、全ゲート成功。

## Constraints

- CDP 接続先照合（ADR-0143）と check 名一意性（ADR-0182）を維持。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
