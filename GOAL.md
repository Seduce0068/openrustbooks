# Goal

AI クライアント注入点を設計受容として記録する（R6 R2-ai#2）。

## Scope

### 背景

`AiMarkdownPipeline` は `client::create_client()` を直接呼ぶが、
core 内の unit test は `#[cfg(test)]` 子モジュールから非公開フィールドを
直接構築してモック `Arc<dyn AiClient>` を注入している（ADR-0133）。
ネットワーク／FS なしで決定的にテストできており、
公開注入点を追加する利得は限定的。
一方で公開注入点は `validate_base_url` / 予約ヘッダ拒否の
fail-closed 境界を迂回し得る。

### 成果物

- `docs/decisions/0227-ai-client-injection.md`: 追加
- `todo.md`: R2-ai#2 を設計受容として完了へ移動
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 既存モック注入の事実と ADR-0133 の契約を記録する。
2. 公開注入点を追加しない判断と、クロスクレート要件が出た場合の最小設計を記載する。
3. 信頼境界（allowlist / 予約ヘッダ拒否 / IPC DTO 境界）を明記する。
4. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- markdownlint 0 issues、verify.sh 成功。

## Constraints

- コード・E2E は変更しない。
- 新規依存を追加しない。
- `master` へ push する。force push はしない。
