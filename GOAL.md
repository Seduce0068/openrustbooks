# Goal

feature 加算を伴う workspace 依存の現状を設計受容として記録する（R6 R2-crates#5）。

## Scope

### 背景

ADR-0246 で feature 差分の無い 18 依存を workspace 参照へ統一した。
残る `chrono` / `uuid` / wasm dev `tokio` は workspace 定義より
多くの feature を member 側で必要とし、
Cargo の feature は加算で解決されるため単純な workspace 化は
意図しない依存・挙動を招く。
よって現状维持を設計受容とし、理由と再評価条件を記録する。

### 成果物

- `docs/decisions/0258-workspace-feature-additive-deps.md`: 追加
- `todo.md`: R2-crates#5 を「設計受容・残置」へ更新
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 除外する 3 依存（chrono / uuid / wasm dev tokio）と feature 差分を列挙する。
2. Cargo の feature 加算性と、workspace 側へ上げた場合の影響を記録する。
3. ADR-0242 / 0246 の判断を引き継ぎ、rumdl は対象外と明記する。
4. 再評価条件（MSRV 上の default-features override、Cargo 仕様変更、
   依存更新）を未決定として残す。
5. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 規約・索引・todo が整合し、全ゲート成功。

## Constraints

- Cargo.toml / Cargo.lock / コード / E2E は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
