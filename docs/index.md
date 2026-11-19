# docs

openrustbooks の設計・要件・判断記録を置くディレクトリ。

- [requirements.md](requirements.md) — ユーザー要件、未決定事項、検証条件（VC-001〜015）。
- [e2e-traceability.md](e2e-traceability.md) — 検証条件（VC）と E2E check 名（253 件）の対応表。
- [architecture.md](architecture.md) — 初期提案の構成案。現状は ADR と README を参照。
- [runbook-verified-build.md](runbook-verified-build.md) — 検証済み dist の保持・SHA256 検証・serve・復旧手順。
- [decisions/](decisions/index.md) — 決定記録（ADR）。全 221 件の索引。
- [ADR-0207: AppState のスコープと設計受容](decisions/0207-appstate-scope.md) — 設定系トランザクション完了の確認と残リスクの個別追跡。
- [ADR-0218: AI カスタムエンドポイント opt-in の設計受容](decisions/0218-custom-ai-endpoint.md) — allowlist の現状契約と将来の最小安全設計。
- [ADR-0219: 検証済み配布物のローカル保持](decisions/0219-verified-artifact-retention.md) — 全ゲート成功時のみ dist を保持し manifest で完全性を記録。
- [ADR-0220: E2E トレーサビリティの 253 件への更新](decisions/0220-traceability-refresh.md)
  — ADR-0206 の追従漏れを解消し、2 件を VC-004 / VC-009 へ割り当て。
- [investigation/](investigation/) — 別セッションによる残タスク調査レポート。
