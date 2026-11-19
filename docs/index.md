# docs

openrustbooks の設計・要件・判断記録を置くディレクトリ。

- [requirements.md](requirements.md) — ユーザー要件、未決定事項、検証条件（VC-001〜015）。
- [e2e-traceability.md](e2e-traceability.md) — 検証条件（VC）と E2E check 名（254 件）の対応表。
- [architecture.md](architecture.md) — 初期提案の構成案。現状は ADR と README を参照。
- [runbook-verified-build.md](runbook-verified-build.md) — 検証済み dist の保持・SHA256 検証・serve・復旧手順。
- [decisions/](decisions/index.md) — 決定記録（ADR）。全 230 件の索引。
- [ADR-0207: AppState のスコープと設計受容](decisions/0207-appstate-scope.md) — 設定系トランザクション完了の確認と残リスクの個別追跡。
- [ADR-0218: AI カスタムエンドポイント opt-in の設計受容](decisions/0218-custom-ai-endpoint.md) — allowlist の現状契約と将来の最小安全設計。
- [ADR-0219: 検証済み配布物のローカル保持](decisions/0219-verified-artifact-retention.md) — 全ゲート成功時のみ dist を保持し manifest で完全性を記録。
- [ADR-0220: E2E トレーサビリティの 253 件への更新](decisions/0220-traceability-refresh.md)
  — ADR-0206 の追従漏れを解消し、2 件を VC-004 / VC-009 へ割り当て。
- [ADR-0222: updater 未配線の設計受容](decisions/0222-updater-deferred.md)
  — 更新チェック未配線を見送り、設定表示を「未配線」へ正直化。
- [ADR-0224: 古い TODO の ADR 実績による整理](decisions/0224-todo-reconciliation.md)
  — 解消済み TODO を ADR 実績へ照合し、残リスクを ADR の未決定へ集約。
- [ADR-0225: 壊れた画像のセトリング待ち解消](decisions/0225-broken-image-settling.md)
  — 画像の error を記録して失敗画像を待ち対象外にし、章ごとの 2 秒待ちを防ぐ。
- [ADR-0226: status 文言の日本語統一](decisions/0226-status-message-language.md)
  — lang=ja 固定の Web UI で Reader status を日本語へ統一し、aria-live 分離は後段へ残す。
- [ADR-0227: AI クライアント注入点の設計受容](decisions/0227-ai-client-injection.md)
  — 公開注入点を追加せず、将来のクロスクレート要件に備える最小設計を記録。
- [investigation/](investigation/) — 別セッションによる残タスク調査レポート。
