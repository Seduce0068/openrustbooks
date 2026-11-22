# Goal

a11y 文言とリンク表示を改善する（R6 レビュー / R2-UX01）。

## Scope

### 成果物

- `web/src/lib.rs`: 変更
- `web/index.html`: 変更
- `docs/decisions/0306-a11y-label-cleanup.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. lang=ja に対して主要ナビ / ボタンの英語ラベルを日本語化する（E2E 依存文言は維持）。
2. 巨大画像警告に「縮小は実行されない」旨を可視化する。
3. 破壊操作の window.confirm を非推奨文言へ置き換えるか、既存 dialog パターンへ寄せる。
4. Clear data を「読書データを削除」等へ明確化する。
5. 非表示 file input の focus 導線、リンクの下線、ライセンス導線を追加する。
6. E2E 256 件・check 名・#settings-language / updater 固定文言 / 保存成功文言を維持。
7. verify.sh 全ゲート成功。

## Constraints

- 新機能実装（本格的な file picker / ライセンス画面）は行わない。
- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- 主要 nav / Settings ラベルを日本語化し、リンク下線と file input focus を改善。
- Clear data の危険告知と AGPL 全文リンクを追加。
- E2E 必須文言を維持、R4 medium 2 件を反映。
- verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- Label in Name / dialog 統一は ADR-0306 の未決定。
