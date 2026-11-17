# Goal

ADR-0002 の Wasm lint に関する記述を実装状況に合わせて明確化する。

## Scope

### 背景

調査レポートで「ADR-0002 と ADR-0003 が Wasm 対応方針で矛盾」と指摘された。
精査の結果、ADR-0003 は lint を両対応、client/pipeline をネイティブ専用として
おり矛盾ではないが、ADR-0002 の「Wasm 対応によりデスクトップ/ブラウザ双方で
同一品質の lint を提供可能」という記述は、現状 `WasmMarkdownLinter` が空配列を
返すプレースホルダである点と食い違う。

### 成果物

- `docs/decisions/0002-ai-markdown-pipeline.md`: 変更（現状を明記）
- `GOAL.md`: 本契約

### 必須事項

1. ADR-0002 の「結論」と「影響」に、次を明記する。
   - rumdl を共通コアに採用する方針は維持する。
   - ブラウザ版の lint は現在プレースホルダ（空配列を返す）であり、
     ADR-0003 の方針どおり client/pipeline はネイティブ専用。
   - ブラウザ版での rumdl 配線は未実装（将来タスク）。
2. 判断そのもの（rumdl 一択）は変更しない。
3. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。
- 6 ゲートすべて成功する。

## Constraints

- コードを変更しない。
- 既存の決定内容を書き換えず、現状の注記を追加する。
- `master` へ push する。force push はしない。
