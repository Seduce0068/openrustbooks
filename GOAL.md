# Goal

状態: **完了（検証済み）**。R6 R1-archive#3 / ADR-0252 の契約は満たされた。

ZIP パスの深さ上限を設計受容として記録する（R6 R1-archive#3）。

## Scope

### 背景

`normalize_path()` はコンポーネント数を制限していないが、
再帰せず O(入力長) のヒープ確保のみで動作する。ZIP エントリ名（u16 長、
最大 65535 バイト）は `normalize_path()` 側の制約として扱い、
`resolve_reference()` が受け取る href は XML 内容のため u16 長ではなく
`max_entry_size = 64 MiB`（1 エントリの非圧縮サイズ上限）が実質的な上限になる。
入力サイズ・エントリ数・宣言サイズ・実展開予算の既存防御に加え、
サニタイザには深さ制限を避けた前例（ADR-0050 / 0065、ただし回帰テスト併用）があり、
即時のセキュリティ必須対応ではないと判定された。

### 成果物（完了）

- `docs/decisions/0252-path-depth-limit.md`: 追加・監査指摘反映済み。
- `todo.md`: R1-archive#3 を「設計受容」として完了へ移動（既存）。
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新。
  本セッションの書き込み範囲は `GOAL.md` と `docs/index.md`（254 → 255）。

### 必須事項（すべて完了）

1. 深さ上限を追加せず、既存防御との責任分界を ADR に記録した。
2. 実装時に選ぶなら `ArchiveLimits` フィールド案が望ましい旨と、
   128 は出典なしの例示値（ADR-0188 / 0209 の retry 回数とは無関係）であることを明記した。
3. 未決定（wasm DoS 実測、JS DTO の信頼境界、XML/TOC 深さとの一本化）を残した。
4. markdownlint 0 issues。E2E 255 件はコード・E2E 未変更のため不変
   （本セッションでは verify.sh を再実行せず、前回の成功実績を継承）。

## Acceptance criteria（検証済み）

- 規約・索引・todo が整合し、全ゲート成功（前回 verify.sh の実績）。
- 監査指摘を ADR-0252 / `docs/index.md` / 本契約へ反映済み。

## Constraints

- コード・E2E は変更しない。
- 新規依存を追加しない。
- 本セッションでは git 操作を行わない（push は範囲外）。
