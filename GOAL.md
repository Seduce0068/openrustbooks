# Goal

`prefers-contrast: more` で枠線と補助文字のコントラストを補強する（R6 B06 / C01 残）。

**状態: 完了（2026-11-19）。** 下記の契約はすべて充足済みで、新規の実装作業は不要。
詳細は [ADR-0236](docs/decisions/0236-prefers-contrast.md) を正とする。

## Status

- 判定: 完了。追跡先は ADR-0236 の「未決定」と `todo.md` の実機 5 ブラウザ検証へ移行済み。
- 根拠: 静的コントラスト再計算（4.83 / 10.31 / 6.96 / 12.02）、E2E 254/0、
  `./scripts/verify.sh` rc=0、markdownlint 0 issues。
- 作成日時の確認: `date --iso-8601=seconds` → `2026-11-19T18:50:35+09:00`。

## Scope

### 背景

`forced-colors` は ADR-0039 で対応済みだが、`prefers-contrast: more` は
「必要性なし」として未対応だった。しかし static 計算で light の枠線
`#e5e7eb` / 白 = 1.24:1、dark の枠線 `#334155` / `#0f172a` = 1.72:1 と
WCAG 1.4.11（非テキスト 3:1）に不足していたため、本次で補強した。

### 成果物（すべて存在）

- `web/index.html`: 変更済み（`prefers-contrast: more` のトークン上書き 2 ブロック）。
- `docs/decisions/0236-prefers-contrast.md`: 追加済み。
- `GOAL.md` / `todo.md`: 完了契約・残課題の更新済み。
- decisions / docs 各 index・README・`docs/log.md`: 236 件へ追従済み。

### 必須事項（充足結果）

1. `@media (prefers-contrast: more)` で light / dark の `--color-border` を 3:1 以上へ強化
   — 充足（light 4.83:1 / dark 6.96:1）。
2. `--color-muted` も補強し、通常テキストの 4.5:1 を維持
   — 充足（light 10.31:1 / dark 12.02:1）。
3. forced-colors の既存契約（ADR-0039）を壊さない
   — 充足（ブロック非変更・後方配置を契約化）。
4. E2E 254 を維持する（色値への直接依存なし）— 充足（check 237 + equal 17、254/0）。
5. 全ゲート成功 — 充足（`verify.sh` rc=0）。

## Acceptance criteria

- [x] 静的コントラスト比が枠線 3:1 以上、muted 4.5:1 以上（4.83 / 10.31 / 6.96 / 12.02）。
- [x] E2E 254 維持（色値依存なし、254/0）。
- [x] forced-colors ブロック非変更・後方配置。
- [x] `verify.sh` 全ゲート rc=0、markdownlint 0 issues。

## Constraints

- 一般テーマの配色は変更しない（media query 内のみ）— 遵守。
- 新規依存を追加しない — 遵守。
- `master` へ push する。force push はしない — 本次の文書更新は git 操作を行わないため未実施。

## 未決定（本契約の完了後も残る）

- 一般テーマの border の WCAG 1.4.11 未達（別タスク）。
- dark の `th` / `td` / `blockquote` に残るハードコード `#444` の枠線。
- hover / focus / disabled の包括的なコントラスト監査。
- 実ブラウザ 5 種（Chrome / Edge / Brave / Firefox / Safari）の実機検証。
- forced-colors の優先順がソース順依存である点（構造的保障は未決定）。
