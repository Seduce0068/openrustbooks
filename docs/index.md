# docs

openrustbooks の設計・要件・判断記録を置くディレクトリ。

- [requirements.md](requirements.md) — ユーザー要件、未決定事項、検証条件（VC-001〜015）。
- [e2e-traceability.md](e2e-traceability.md) — 検証条件（VC）・REQ / 主要 ADR・E2E check 名（255 件）の対応表。
- [architecture.md](architecture.md) — 初期提案の構成案。現状は ADR と README を参照。
- [runbook-verified-build.md](runbook-verified-build.md) — 検証済み dist の保持・SHA256 検証・serve・復旧手順。
- [runbook-release.md](runbook-release.md) — 導入・更新・ロールバック・既知問題と公開作業の未決定。
- [documentation-operations.md](documentation-operations.md) — OKF・索引・log・ADR・lint の運用規約。
- [decisions/](decisions/index.md) — 決定記録（ADR）。全 262 件の索引。
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
- [ADR-0232: 初版リリーススコープ](decisions/0232-initial-release-scope.md)
  — 実装・検証済み範囲と、初版に含まない項目・未決定を実証付きで切り分ける。
- [ADR-0233: 既存ソリューション選定根拠の横断記録](decisions/0233-solution-selection-rationale.md)
  — ライセンス / 鮮度 / 既知脆弱性 / Wasm 対応を横断比較し、選定根拠を一箇所に記録。
- [ADR-0234: ドキュメント運用の一本化](decisions/0234-documentation-operations.md)
  — OKF / 索引 / log / lint の規約と設定をリポジトリ内へ集約する決定。
- [ADR-0235: E2E 待機の条件待ち化と swipe 二重 dispatch の解消](decisions/0235-e2e-conditional-wait.md)
  — 未埋め込みイベントの二重発火を 1 回 dispatch へ、到達条件が明確な待機を条件待ちへ統一。
- [ADR-0236: prefers-contrast: more の枠線・補助文字コントラスト強化](decisions/0236-prefers-contrast.md)
  — media query 内で枠線・補助文字を強化し、forced-colors 契約を後方で維持。
- [ADR-0237: CI / ローカルゲートの整合と trunk build への --locked 付与](decisions/0237-ci-gate-consistency.md)
  — wasm_targets は target 導入のみ、extra_gates が実検査。verify.sh の trunk build に --locked を付与。
- [ADR-0241: generated.at の意味論と日付逆転の扱い](decisions/0241-generated-timestamp-semantics.md)
  — 生成／正規化時点の記録である `generated.at` を作成時刻と誤読しない規約を追加し、既存値を保持。
- [ADR-0242: rumdl feature 選択の現状と ADR-0003 の履歴化](decisions/0242-rumdl-feature-status.md)
  — native / wasm とも featureless である現状を ADR-0242 を正とし、target 別 feature 選択を履歴化。
- [ADR-0252: ZIP パスの深さ上限を追加しない設計受容](decisions/0252-path-depth-limit.md)
  — 非再帰正規化と既存の多重防御を理由に深さ上限を追加せず、責任分界を記録。
- [ADR-0253: Tauri 設定保存 UI の段階設計](decisions/0253-tauri-settings-save-design.md)
  — browser は読み取り専用 fail-closed を維持し、desktop のみ最小 IPC と実装前提を順に満たして保存 UI を接続。
- [ADR-0254: リリース運用の文書先行開示](decisions/0254-release-operations-disclosure.md)
  — 導入・更新・ロールバックと既知問題を先に開示し、公開 / 署名 / endpoint は未決定として残す。
- [ADR-0256: Wasm AI cluster の削除](decisions/0256-wasm-ai-cluster-removal.md)
  — 未使用の AI 3 型と自己言及 test を削除し、ADR-0156 の AI 部分のみ supersede する。
- [ADR-0258: feature が一致しない依存の workspace 化を見送り現状を設計受容とする](decisions/0258-workspace-feature-additive-deps.md)
  — chrono / uuid の member 加算と wasm dev tokio の member 限定を記録し、workspace 化せず残置する。
- [ADR-0261: OS キーチェーン移行の設計受容](decisions/0261-os-keychain-deferred.md)
  — 平文保存 + redaction + Unix 0600 を受容し、移行の未決定を ADR-0098 へ集約して実装を見送る。
- [ADR-0262: E2E 件数 current 表記のコード出典追従](decisions/0262-e2e-count-documentation-refresh.md)
  — current 表記をコード出典の 255 件へ追従し、254 件は履歴として保持する決定。
- [investigation/](investigation/) — 別セッションによる残タスク調査レポート。
