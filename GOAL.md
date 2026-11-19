# Goal

ZIP パスの深さ上限を設計受容として記録する（R6 R1-archive#3）。

## Scope

### 背景

`normalize_path()` はコンポーネント数を制限していないが、
再帰せず O(入力長) のヒープ確保のみで、
ZIP エントリ名は u16 長（最大 65535 バイト）に制約される。
入力サイズ・エントリ数・宣言サイズ・実展開予算の既存防御に加え、
サニタイザの深い入れ子テスト（ADR-0050 / 0065）の前例があり、
即時のセキュリティ必須対応ではないと判定された。

### 成果物

- `docs/decisions/0252-path-depth-limit.md`: 追加
- `todo.md`: R1-archive#3 を「設計受容」として完了へ移動
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 深さ上限を追加せず、既存防御との責任分界を ADR に記録する。
2. 実装時に選ぶなら `ArchiveLimits` フィールド案が望ましい旨と、
   128 は出典なしの例示値であることを明記する。
3. 未決定（wasm DoS 実測、JS DTO の信頼境界、XML/TOC 深さとの一本化）を残す。
4. markdownlint 0 issues、verify.sh（E2E 255）成功。

## Acceptance criteria

- 規約・索引・todo が整合し、全ゲート成功。

## Constraints

- コード・E2E は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
