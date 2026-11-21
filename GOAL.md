# Goal

AI provider の空レスポンスを失敗扱いにする（R6 レビュー / R2-ER01）。

## Scope

### 背景

OpenAI 経路は choices が空 / content が None の場合に `unwrap_or_default()` で空文字を返す。
Ollama 経路も `result["response"].as_str().unwrap_or("")` で空文字を返す。
これが lint_passed=true の「生成成功」として扱われ、空文書保存の危険がある。

### 成果物

- `core/src/ai_markdown/client.rs`: 変更
- `docs/decisions/0292-ai-empty-response.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. choices 欠落 / content None / response キー欠落を CoreError::AiProvider の固定文言で拒否する。
2. 空文字が正当な応答であり得る場合は root cause を区別して失敗扱いにする。
3. エラー文言に host / body / secret を出さない（ADR-0287 に整合）。
4. 単体 test を追加する。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO 維持。

## Constraints

- 外部 AI API への送信は行わない（mock / ロジック test のみ）。
- prompt / pipeline の成功契約を壊さない。
- `master` へ push する。force push はしない。
