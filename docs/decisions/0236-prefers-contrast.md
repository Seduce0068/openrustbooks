---
type: Decision
title: "ADR-0236: prefers-contrast: more の枠線・補助文字コントラスト強化"
description: "prefers-contrast: more で枠線と補助文字を強化し、forced-colors の既存契約を後方で維持する決定。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-19T18:50:35+09:00"
status: accepted
---

## 背景

ADR-0039 は `prefers-contrast: more` について「既存の配色は最小要件を満たしており必要性なし」
として未対応を判断していた。しかし ADR-0230 の未決定でも指摘されたとおり、
通常テーマの枠線は light `#e5e7eb` / `#ffffff` = **1.24:1**、
dark `#334155` / `#0f172a` = **1.72:1** であり、
WCAG 1.4.11（非テキストのコントラスト 3:1）を満たしていない。
R6 B06 / C01 の残課題として、通常テーマを変えずに利用者設定時だけ補強する方針を採用する。

## 決定

### トークンの上書き

`@media (prefers-contrast: more)` の中で `--color-border` と `--color-muted` のみを上書きする。

- `:root`（light）: `--color-border: #6b7280`（**4.83:1**）、
  `--color-muted: #374151`（**10.31:1**）。
- `html.dark`（dark）: `--color-border: #94a3b8`（**6.96:1**）、
  `--color-muted: #cbd5e1`（**12.02:1**）。

### dark の両経路

`@media (prefers-contrast: more) and (prefers-color-scheme: dark)` にも
`:root` の dark 値（border `#94a3b8` / muted `#cbd5e1`）を定義する。
これにより `html.dark` がまだ付与されていない場合や JS が無効な場合でも、
OS が dark なら暗色側の強化値が適用される。

### forced-colors 契約

- `@media (forced-colors: active)` のブロックは変更しない（ADR-0039 を維持）。
- 当該ブロックは新しい `prefers-contrast` ブロックの**後方**に置く。
  CSS のカスケードでは同じ詳細度の宣言は後にあるものが勝つため、
  強制カラーモード有効時は常にシステムカラーが上書きされる。
- この「非変更・後方配置」を本次の契約として固定する（後述の未決定も参照）。

### 範囲

- 上書きは media query 内のみ。一般テーマ（通常時の配色）は変更しない。
- 新規依存を追加しない。E2E は色値へ依存せず、期待総数 254 を維持する。

## 選択肢と根拠

| 選択肢 | 判断 | 根拠 |
| --- | --- | --- |
| 一般テーマの border を直接 3:1 以上へ変更 | 却下 | 全利用者のデザインを変え、ADR-0039 / ADR-0230 の「平時の表示は変えない」契約に反する。 |
| `prefers-contrast: more` で border / muted を強化（採用） | 採用 | 必要な利用者だけに適用でき、light / dark の両経路を media query で把握できる。最小差分で ADR-0039 も維持できる。 |
| `forced-colors` だけで代替 | 却下 | 強制カラーモードは OS の配色へ完全に委譲する機能で、通常のハイコントラスト設定を持つ利用者を救えない。 |
| `forced-color-adjust: none` で作者色を固定 | 却下 | ADR-0039 と同じく、利用者が選んだ配色を奪うため採用しない。 |

## 影響

- 変更: `web/index.html`（2 つの media query を追加、forced-colors ブロックは後方へ維持）。
- 文書: 本 ADR を追加し、decisions / docs の各 index（235 → 236 件）、README、
  `docs/log.md`、`GOAL.md`、`todo.md` を追従する。
- 通常テーマの外観、E2E の check 名・期待総数、Rust コード・依存・CI 設定は変更なし。

## 検証（Runtime Proof）

- 静的コントラスト比（WCAG 相対輝度で再計算、実装値と一致）:

| 対象 | 通常時 | `prefers-contrast: more` |
| --- | --- | --- |
| light border（`--color-border` / 背景） | 1.24:1 | **4.83:1** |
| light muted（`--color-muted` / 背景） | 4.83:1 | **10.31:1** |
| dark border（`--color-border` / 背景） | 1.72:1 | **6.96:1** |
| dark muted（`--color-muted` / 背景） | 6.96:1 | **12.02:1** |

- E2E の色非依存: `tools/verify-web-reader.py` に 16 進色リテラル、
  `--color-border` / `--color-muted` への参照は存在しない。
  強制カラーモードの既存チェック（2777 / 2781 行）は
  `matchMedia` と境界の有無（`borderTopWidth` / `visibility`）を確認するだけで、
  色値そのものは検証しない。check 名 254 件（check 237 + equal 17）は不変。
- 実装エージェントの検証: `./scripts/verify.sh` は全ゲート成功で終了コード 0、
  E2E **254/0**。
- 文書エージェントの再検証（2026-11-19）: Python で上記 8 通りのコントラスト比を
  再計算し表示値と一致、`grep` で E2E の色依存なしを確認、
  `npx markdownlint-cli2` は 0 issues。

再現コマンド（Kubuntu / リポジトリルート）:

```bash
python3 - <<'PY'
def lum(h):
    c = [int(h[i:i+2], 16) / 255 for i in (1, 3, 5)]
    f = [x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4 for x in c]
    return 0.2126 * f[0] + 0.7152 * f[1] + 0.0722 * f[2]
def ratio(a, b):
    x, y = sorted((lum(a), lum(b)), reverse=True)
    return (x + 0.05) / (y + 0.05)
for fg, bg in [('#6b7280', '#ffffff'), ('#374151', '#ffffff'),
               ('#94a3b8', '#0f172a'), ('#cbd5e1', '#0f172a')]:
    print(f'{fg}/{bg} = {ratio(fg, bg):.2f}:1')
PY
grep -nE '#[0-9a-fA-F]{6}|color-border|color-muted' tools/verify-web-reader.py
npx markdownlint-cli2 'docs/**/*.md' README.md todo.md
```

## 未決定

### 一般テーマの border の 1.4.11 未達

通常時（media query 非該当）の border は 1.24:1 / 1.72:1 のままで WCAG 1.4.11 未達が残る。
デザイン全体を変えるため別タスクとして扱い、本次は行わない。

### ハードコードされた #444 の枠線

dark の `.reader-html` の `th` / `td` と `blockquote` はトークンではなく
`#444` を直接指定しており、今回の上書き対象外。共通トークン化は未決定。

### hover / focus / disabled

hover・focus リング・disabled 状態の包括的なコントラスト監査は未実施。
ADR-0230 の未決定を継続し、個別に判断する。

### 実ブラウザ 5 種の未測

静的計算と単一エンジンの E2E のみで、Chrome / Edge / Brave / Firefox / Safari の
実機における `prefers-contrast: more` の解釈と描画は未検証（ADR-0232 の未決定を継続）。

### forced-colors の優先順がソース順依存

強制カラーモードの優先は「forced-colors ブロックを後方に置く」というソース順に依存する。
宣言順が変わると優先関係も変わるため、構造的な保障（詳細度や `@layer` の導入）は未決定。

## 参照

- [ADR-0039: 強制カラーモードへの対応](0039-forced-colors.md)
- [ADR-0230: dark テーマのボタン文字コントラスト](0230-dark-button-contrast.md)
- [ADR-0232: 初版リリーススコープ](0232-initial-release-scope.md)
- [ADR-0234: ドキュメント運用の一本化](0234-documentation-operations.md)
- [GOAL.md](../../GOAL.md)
