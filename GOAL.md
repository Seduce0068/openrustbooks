# Goal

フラグメントスクロールの retry を強化し、残るフレークを解消する。

## Scope

### 背景

ADR-0046 で `scrollIntoView()` から明示的な `scrollTop` 設定に変更し、
10 回連続成功を確認した。しかしその後も約 1/10 の頻度で
「フラグメント付きリンクで移動先へスクロールする」が `scrollTop=0` で
失敗する。現状の retry は 3 回・`setTimeout(0)` のみで、画像のレイアウトが
確定する前に retry が使い切られている。

### 成果物

- `web/src/lib.rs`: 変更（retry 回数と間隔）
- `docs/decisions/0052-fragment-retry-hardening.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. retry の強化:
   - フラグメント移動で `scrollTop` が 0 のままの場合、16ms（1 フレーム
     相当）間隔で最大 12 回まで再試行する。
   - 位置保存（フラグメント無し）の挙動は変更しない。
2. 既存の挙動を変えない:
   - 既存の 121 テスト、E2E 166 項目を変えない。

### 必須テスト

- E2E を 15 回連続実行し、フラグメントスクロールの失敗が 0 件であること。

## Acceptance criteria

- 6 ゲートすべて成功する。
- CI は billing issue で実行不可（ローカル検証で代替）。

## Constraints

- 新規依存を追加しない。
- `desktop/` を変更しない。
- コメントと doc コメントは日本語で簡潔に書く。
- `master` へ push する。force push はしない。
