# Goal

既知脆弱性のある依存 2 件を修正する（todo.md B03）。

## Scope

### 背景

`cargo audit` が次を報告した。
- ammonia 4.2.4: RUSTSEC-2026-0533（SVG/MathML の mutation XSS）→ 4.2.5 以上
- pulldown-cmark 0.13.5: RUSTSEC-2026-0433（入れ子 wikilink の二次時間）→ 0.13.6 以上

いずれも既存のバージョン指定（ammonia "4.2.4"、pulldown-cmark "0.13"）の
セマンティック互換範囲内で修正できる。

### 成果物

- `Cargo.lock`: 変更（2 クレートのバージョン更新）
- `docs/decisions/0086-dependency-vulnerability-fix.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `cargo update -p ammonia --precise 4.2.5` と
   `cargo update -p pulldown-cmark --precise 0.13.6` で Cargo.lock を更新する。
2. `cargo audit` で脆弱性 0 件になる（警告は既存）。
3. 6 ゲートと E2E が通る。
4. 必要なら `core/Cargo.toml` の下限を更新する（API 変更は想定しない）。

## Acceptance criteria

- `cargo audit` の vulnerabilities が 0 件。
- 6 ゲート・E2E が成功する。

## Constraints

- 新規クレートは追加しない（既存依存のバージョン更新のみ）。
- `desktop/` の設定は変更しない。
- `master` へ push する。force push はしない。
