---
type: Todo
title: "openrustbooks プロジェクトTODO"
description: "openrustbooks 固有のタスクを管理するプロジェクト内 TODO。レビュー統合と ADR 実績の照合結果を反映。"
tags:
  - todo
  - openrustbooks
  - tasks
  - cross-review
generated:
  by: dsh (lead)
  at: "2026-11-17T21:20:00+09:00"
status: current
---

## 進行中

（項目なし）

## 未着手

### 優先度: 高

<!-- R6-2026-11-19: swarm-project-review + swarm-todo-discovery 統合 -->
<!-- 2026-11-19: ADR 実績との照合で解消済みを完了へ移動（ADR-0224）。 -->
- 【A05 残】Tauri コマンドの UI 接続（段階 2）と設定保存 UI のみ残存。
  ADR-0127 / ADR-0223 の段階 2、ADR-0128 / ADR-0193 の「未決定」を追跡先とする。
  CSP（B01 / B02）は ADR-0091 / ADR-0130 / ADR-0191 で解消済みのため重複計上しない。

### 優先度: 中

<!-- R6-2026-11-19 中優先度 / 2026-11-19: 解消済みを完了へ移動（ADR-0224 / TD-M10 は ADR-0225 で完了） -->

- 【B04-B06 / C01 残】UX と a11y の残課題。IME・モーダル・reduced-motion・dark ボタンコントラスト、
  `prefers-contrast: more` は完了（ADR-0236）。残るは実機 5 ブラウザ検証のみ
  （ADR-0230 / 0224 / 0236 の未決定を追跡先とする）。

### 優先度: 低

<!-- R6-2026-11-19 低優先度 / 2026-11-19: 外部要因と ADR 未決定を集約 -->
- 【CI-H01】billing 解消後の実 CI 検証。trunk build・配布パス検査・
  E2E 254 件を共通ワークフローのマージゲートへ組み込む。
  根拠 `.github/workflows/ci.yml:6`（no-external-font-refs / audit /
  trunk build + E2E は verify.sh 側のゲート）。billing は外部要因のため低優先度。
- 【D05 残 / OS キーチェーン】API キー / client_secret / refresh_token の
  平文保存を OS キーチェーンへ移す判断は ADR-0098 の「未決定」を追跡先とする。
  IPC マスク／マスク済み DTO／DTO 境界は完了済み（ADR-0098 / 0102 / 0128 / 0186）。

- 【TD-L08】serde_yaml 0.9.34+deprecated / tikv-jemalloc 残存は上流 optional 化待ち。
  根拠 `Cargo.lock:5894` / `docs/decisions/0155:45,69`。受容リスクを ADR に明記済み。

<!-- ADR-0207 の残リスク（低優先度で個別追跡） -->

- 【R1-tauri#8 / R3-release#1,#2】updater の公開鍵・署名設定と
  リリース専用ワークフロー（`contents: write` + 署名 secrets）。
- 【R3-release#4,#5】導入・更新・ロールバック手順、既知問題の開示、
  署名秘密鍵の `.gitignore` 登録。
- 【R3-config#5】容量超過時の位置削除を通知・退避方式へ変更。
- 【R2-epub#1-#4】EPUB 層の API・エラー・所有権境界を整理。
  `open_owned` の limits 指定、1316 行ファイルの分割、欠落エラーの統一、
  Document 側の二重定義解消。
- 【R2-crates#2-#5】ADR-0003 の rumdl feature 記述更新、
  未実装 Wasm API の非公開化、core の Web バインディング分離、
  workspace 依存の一元化。
- 【R1-archive#2,#3,#5 残】実展開サイズの累計上限、入力パーサ側の深さ上限、表紙 media-type 検証。
  正規化後同名エントリの拒否は完了（ADR-0240）。
- 【R5-adr 全体評価】ADR-0002 と ADR-0003 の日付逆転（推測）を確認。

## 完了

- [x] 【R1-archive#4】正規化後同名の ZIP エントリを fail-closed で拒否。archive / epub / wasm に test 5 件追加。
      core 274・wasm 15・E2E 254 件・全ゲート成功 / ADR-0240。完全一致 raw 名は未決定。
- [x] 【R3-release#3】tauri.conf.json の version を削除し Cargo workspace を単一情報源へ。
      check-version-consistency.sh を verify.sh / ci.yml へ配線。負向 test 4 ケース失敗、E2E 254 件・全ゲート成功 / ADR-0239。
- [x] 【R4-unit#4】deflate 圧縮率依存の assertion を方式／メタデータの直接検証へ置換（archive.rs / wasm.rs）。
      core 270・wasm 14・E2E 254 件・全ゲート成功 / ADR-0238。
- [x] 【R3-ci#3-#5】wasm ゲートの二重指定の有無、CI とローカルのビルドプロファイル差、
      `trunk build` への `--locked` 付与を確認・実施。
      `wasm_targets` は `rustup target add` のみ、`extra_gates` が実検査で二重実行ではないと判定。
      `CARGO_PROFILE_DEV_DEBUG` / `TEST_DEBUG` / `INCREMENTAL` は runner の disk 対策で
      ローカルへ全局適用しない。`verify.sh:268` に `--offline --locked` を付与し、
      `ci.yml` / `ci-wasm-test.sh` は不変。bash -n OK・verify.sh 全ゲート rc=0・
      E2E 254/0・Cargo.lock 差分なし・markdownlint 0 issues / ADR-0237。
      CI-H01（billing 解消後の実 CI 検証）は低優先度で継続。
- [x] 【B06 / C01】`prefers-contrast: more` で枠線・補助文字のコントラストを強化。
       light border 4.83:1 / muted 10.31:1、dark border 6.96:1 / muted 12.02:1 を
       light / dark と `prefers-color-scheme: dark` の両経路へ適用。
       forced-colors ブロックは非変更・後方配置を契約化（ADR-0039 維持）。
       E2E 254/0・verify.sh rc=0・静的計算再現・markdownlint 0 issues / ADR-0236。
       残る実機 5 ブラウザ検証は未着手に継続。
- [x] 【R4-e2e#3,#4 / C08】swipe スタブの二重発火と固定 sleep を条件待ちへ統一。
      先頭 evaluate（`arguments[0..2]` 未埋め込み）を削除し、f-string 版 1 回のみ dispatch。
      到達条件が明確な章移動・縦 / マルチタッチ・ダーク切替・ファイル選択後の DOM / 保存・
      リセット後を `wait_until` / `wait_swipe_done` / `wait_swipe_ignored` ベースへ置換。
      負向チェックは baseline カウンタ＋第 1 章本文＋0.3 秒維持で判定。到達条件のない
      106 件の sleep は据え置き、`wait_until` の fail-closed は維持。
      E2E 254 件・全ゲート成功、AST 254/254、markdownlint 0 issues / ADR-0235。
- [x] 【C04 / R5-ops#1】OKF・索引・log・lint の運用を `docs/documentation-operations.md`
      へ明文化し、`.markdownlint-cli2.jsonc` を追加して一時ファイル依存を解消。
      ADR-0234 を追加し、AGENTS.md のグローバル参照を `~/.agents/AGENTS.md`（正本）へ更新。
      全索引（234 件）・README・log を追従し、markdownlint 0 issues。
- [x] 【D02 / R4-trace#1,#4 / R5】REQ→ADR→E2E 対応表へ REQ / 主要 ADR 列を追加し、
      既存ソリューションの選定根拠（ライセンス / 鮮度 / 既知脆弱性 / Wasm 対応）
      を横断記録。VC は ADR-0233 により初版の測定可能な受け入れ基準として参照。
      E2E check 名 254 件不変、AST 254/254（missing / duplicate / extra 0）、
      markdownlint 0 issues / ADR-0233。
- [x] 【D01】初版スコープ ADR を追加し、native desktop / browser / text / AI /
      保存・検証の実装範囲と初版に含まない項目・未決定を実証付きで明示。
      requirements.md へ導線を追加、E2E 254 件・core 270 件 / ADR-0232。
- [x] 【R4-unit#3】未検出 backend / provider の unit test を追加し core を 267→270 件へ拡充。
      E2E 254 件・全ゲート成功 / ADR-0231。
- [x] 【C05 / R3-error#3,#5】二重 prefix と JsValue Debug を削除し、desktop run() を Result 化（終了コード 101 維持）。
      status 分離は ADR 未決定。E2E 254 件・全ゲート成功 / ADR-0229。
- [x] 【R1-ai#2,#5 / R2-ai#3,#4】AI 死設定を設計受容。月間上限を「未配線」、timeout を「Ollama のみ」と表示し、E2E 254 件・全ゲート成功 / ADR-0228。
- [x] 【R2-ai#2】AI クライアント注入点を設計受容（公開注入点は追加せず）。
      `#[cfg(test)]` 内の `CountingAiClient` ＋ `pipeline_with_mock()` による非公開フィールド
      直接構築を維持。クロスクレート要件が出た場合のみ同期 `with_client` を `pub(crate)` から検討し
      `new()` を委譲。信頼境界（allowlist / 予約ヘッダ拒否 / IPC DTO）を公開しない。
      コード・E2E 不変（core 267）、markdownlint 0 issues / ADR-0227。
- [x] 【TD-M11 第一段】Reader status の英語文言 22 パターンを日本語へ統一し、E2E 期待を同期。
      aria-live 分離は第二段の未決定。E2E 254 件・全ゲート成功 / ADR-0226。
<!-- 2026-11-19: 監査で解消済みと判定した項目を ADR 実績へ照合して移動（ADR-0224）。 -->
- [x] 【TD-M10】壊れた画像のセトリング待ちを解消。画像の error を記録し失敗画像を
      待ち対象外にして、章切替ごとの 2 秒待ちを防止（ADR-0225）。
      正常画像の自然幅待ち・CLS 契約・generation ガードは維持。
      E2E 1 件追加で 253 → **254**（check 237 + equal 17）。
      AST 抽出で expected / found / distinct 254、missing / extra 0、
      `docs/e2e-traceability.md` の新規 check を VC-004 の読書位置行へ割り当て、
      markdownlint 0 issues。E2E 254 項目すべて成功。
- [x] 【B03】既知脆弱性 2 件（ammonia / pulldown-cmark）を修正し、audit ゲートを整備。
      ADR-0086 / 0125 / 0126。
- [x] 【E01】配布 WASM の開発ホスト絶対パスを除去し、パス検査を fail-closed 化。
      ADR-0088 / 0152。
- [x] 【独自・最優先 Tauri パス検証】`resolve_path` の封じ込めと document command の
      パス検証を実装（ADR-0097 / 0129）。
- [x] 【D06】AI の既定無効化・service ガード・送信先 allowlist・同意ダイアログを実装。
      全文プレビューは ADR-0104 の未決定として継続（ADR-0099 / 0100 / 0101 / 0104）。
- [x] 【D05 完了分】秘密情報の IPC 露出をマスクし、マスク済み DTO と DTO 境界を整備。
      OS キーチェーン移行は ADR-0098 の未決定として低優先度で継続
      （ADR-0098 / 0102 / 0128 / 0186）。
- [x] 【B09】履歴が 1 件のときも localStorage を更新（ADR-0085）。
- [x] 【独自 R2-arch-state#1】Markdown 保存時に front matter を保全（ADR-0087）。
- [x] 【B01 / B02 / E03】Web と Desktop の CSP を強化し、外部フォントを削除。
      SVG の参照制御と XML PI 除去も完了（ADR-0044 / 0045 / 0091 / 0130 / 0191）。
- [x] 【D03 / R2-crates#1】wasm `init()` の二重定義を解消し初期化を一本化（ADR-0089）。
- [x] 【R3-config#2,#3,#4】設定保存の原子化・revision CAS・インポート時の保護を実装
      （ADR-0157 / 0158 / 0209 / 0210 / 0211）。
- [x] 【R4-e2e#1,#2 / C06】E2E 期待総数と CDP 接続先の決定的照合を実装
      （ADR-0090 / 0143 / 0182）。
- [x] 【C07 / R5-adr#2】ADR の未決定を解決結果へ更新（mailto / XML PI、ADR-0045 / 0082）。
- [x] 【A09 / B10 / C08】README 表記、古い契約値、未使用依存とデバッグ関数を整理
      （ADR-0083）。
- [x] 【R1-ai lint auto-fix】lint 自動修正を実装し、検証分岐を一元化
      （ADR-0110 / 0194）。
- [x] 【R4-unit#5】エラー検証ヘルパーを集約（ADR-0096 等のテスト整理）。
- [x] 【TD-M22 段階 1】WasmArchive を公開（entries/read/is_epub）、固定文言エラーと wasm test 7 件追加（wasm 14）。
      E2E 253 件・全ゲート成功 / ADR-0223。UI 接続は段階 2 の未決定。
- [x] 【TD-M21】updater の未配線を設計受容（見送り）。設定表示を「更新確認: 未配線」へ正直化。
      本番接続・自動チェック・自動インストールは実装せず、check_updates は死設定のまま。
      公開鍵 / endpoint / 署名 secrets / Release workflow は未決定として ADR に集約。
      コード変更は表示のみ、E2E 253 維持 / ADR-0222。
- [x] 【TD-M20】extra_headers / Ollama options をクライアントへ配線。予約ヘッダ拒否と options マージで unit 7 件追加（core 267）。
      E2E 253 件・全ゲート成功 / ADR-0221。
- [x] 【TD-L12】E2E トレーサビリティを 251 → 253 件へ更新（ADR-0206 の追従漏れ解消）。
      Python AST で checker.check 236 + checker.equal 17 = 253 件を抽出し、
      不正 EPUB の固定文言通知を VC-009、reduce の即時フラグメント移動を VC-004 の
      既存行へ割り当て。対応表は missing / duplicate / extra 0（253/253 一致）、
      markdownlint 0 issues / ADR-0220。E2E コード・check 名・期待総数は不変。
- [x] 【TD-M16】検証済み配布物をローカルへ保持し復旧の糸口を確保（即時対応完了）。
       `verify.sh --dist <dir>` で全ゲート成功時のみ保持、manifest で完全性を記録。
       引数なしの既定動作は不変。引数なし／--dist とも rc=0、E2E 253/0、SHA256 5 行すべて OK。
       恒久策（保存先・保持期間、本番プロファイル、--locked、共通ワークフロー upload、
       署名／真正性、Release Workflow との境界）は未決定 / ADR-0219 / Runbook 追加。
- [x] 【TD-M17】AI カスタムエンドポイント opt-in を設計受容（見送り）。allowlist の現状契約と
      将来の最小安全設計を記録。フラグは未実装のまま、ADR-0101 の未決定を継続。
      コード・設定・E2E 不変（253 維持）、markdownlint 0 issues / ADR-0218。
- [x] 【TD-M19】Editor / Reader の死設定に未接続注記を追加。DTO/config 互換を維持し E2E 253 件・全ゲート成功 / ADR-0217。
- [x] 【TD-M18】ストリーミング未接続を設計受容。UI 表示を「未配線」とし、stream_generate を疑似ストリームと明記。
      E2E 253 件・全ゲート成功 / ADR-0216。
- [x] 【TD-M14】Dependabot を導入し Cargo / GitHub Actions の patch / minor を週次自動検出。
      major は手動判断、セキュリティ更新は既定継続。YAML 構文 OK / markdownlint 0 issues / ADR-0215。
- [x] 【TD-M15】MSRV を 1.95 と宣言し CI ツールチェーンも固定。1.95 で fmt/check/test/clippy 成功、stable verify 253/0 / ADR-0214。
- [x] 【TD-M13】prefers-reduced-motion 対応。色 transition を維持しつつ空間移動を opt-in 化、再試行のみ reduce で skip。
      E2E 253 件・全ゲート成功 / ADR-0213。
- [x] 【TD-M12】Reader エラー status / console を操作別固定文言へ redaction。6 経路の `{error:?}` を削除し E2E 252 件・全ゲート成功 / ADR-0212。
- [x] 【PR-M08 / TD-M08】AppConfig / ReaderSettings / SettingsView の 3 重を
      権威・寿命が異なるレイヤー分離として設計受容。WasmConfig は ADR-0205 で削除済み。
      統合は wasm 単体の永続化（ADR-0018 / 0038）、ADR-0157 の現在値保護・
      `exportedAt` 拒否、E2E 251、DTO 境界の秘密マスキング（ADR-0128 / 0186）の観点から却下。
      localStorage `openrustbooks:settings` と camelCase 8 項目を互換契約として固定。
      残課題（ReaderConfig 死設定、ConfigPatch 保存配線、auto_save_delay 供給元、
      UI 設定保存接続）は ADR-0211 の未決定へ集約。コード変更なし・E2E 251 維持 / ADR-0211。
- [x] 【TD-L10】設定更新へ revision CAS と pending 予約を導入。stale expected は Conflict、config/service/disk 不変。
      unit 6 件追加（desktop 79）、E2E 251 件・全ゲート成功 / ADR-0210。
- [x] 【TD-L09】AppConfig::save() を tmp + rename の原子書き込みへ変更。0600 を維持し、unit 7 件追加（core 261）。
      E2E 251 件・全ゲート成功 / ADR-0209。lost update（TD-L10）は未解消。
- [x] 【TD-L11】remove_document で unsaved_changes キーも削除。孤立フラグも掃除し、unit 2 件追加（desktop 74）。
      E2E 251 件・全ゲート成功 / ADR-0208。
- [x] 【PR-M10】AppState の 4 RwLock 集約を Tauri の composition root として設計受容。
      設定系トランザクションは ADR-0177 で完了（6 コマンドが commit_validated_config を共有、
      生成失敗時 3 状態不変、ロック同時保持なし）。Repository/Store 分離は見送り。
      残リスク 3 件を低優先度 todo（TD-L09〜L11）へ切り出し / ADR-0177 / ADR-0207。
      コード変更なし・E2E 251 維持。
- [x] 【TD-L05 / TD-L07】検証条件へ VC-001〜015 を付与し、E2E 251 件との対応表を追加。
      AST 再抽出で missing/duplicate/extra 0、未カバー要件も明記。E2E 251 件・全ゲート成功 / ADR-0206。
- [x] 【TD-M08a】WasmConfig と Wasm test 3 件を削除（wasm 10→7）。
      Rust ソースの参照 0、他 wasm API 維持。E2E 251 件・全ゲート成功 / ADR-0205。
- [x] 【TD-M08b】figment env を明示マップ＋生値検証 filter へ変更。
      不正 max_retries は toml/default を維持、unit 7 件追加（core 255）。
      E2E 251 件・全ゲート成功 / ADR-0204。
- [x] 【TD-L04】CoverImage を href / bytes のみへ縮小し allow(dead_code) を削除。
      serde 未知フィールド無視で core 契約を維持。E2E 251 件・全ゲート成功 / ADR-0203。
- [x] 【TD-L06】AI コマンドの確認／format／alignment を純粋関数へ抽出し異常系 10 件を追加。
      desktop 72 件・E2E 251 件・全ゲート成功 / ADR-0202。
- [x] 【PR-L03】build_app の共通 8 プラグインを with_common_plugins へ集約。
      global_shortcut の挿入位置と登録順を維持。E2E 251 件・全ゲート成功 / ADR-0201。
- [x] 【TD-M07】send_wrapper を web の直接依存 (0.6) へ変更。
      Cargo.lock は 1 行追加のみ、re-export 参照 0 件。E2E 251 件・全ゲート成功 / ADR-0200。
- [x] 【PR-M12】第三者通知へ外部依存 781 件（dev 9 件除外）を自動生成。
      旧 MIT / imagesize は履歴保持、markdownlint 0 issues / ADR-0199。
- [x] 【PR-L01 / L02】Reader の ARIA 属性を修正。aria-labelledby へ一本化し、
      aria-pressed/可視ラベルを effective_dark へ。E2E 251 件・全ゲート成功 / ADR-0198。
- [x] 【TD-L02 / L03】未使用 trait メソッド is_done / available_space を削除。
      ADR-0189 の回帰は next_token シーケンスで担保。残存参照 0、E2E 250 件・全ゲート成功 / ADR-0197。
- [x] 【TD-M05】wasm-pack test を CI extra_gates へ追加。
      scripts/ci-wasm-test.sh を単一情報源とし verify.sh と同順で呼ぶ。
      E2E 250 件・全ゲート成功 / ADR-0196。
- [x] 【TD-M06】rumdl featureless 検査を独立スクリプト化し verify.sh と CI の両方へ接続。
      4 ケース negative test 成功、E2E 250 件・全ゲート成功 / ADR-0195。
- [x] 【PR-M09】auto_fix 分岐を lint.rs の validate_with_auto_fix へ集約。
      native / wasm とも委譲し、直接テスト 2 件追加（core 248）。
      E2E 250 件・全ゲート成功 / ADR-0194。
- [x] 【PR-M11】設定フォームを readonly/disabled 化し常時説明を追加。
      setter 更新を削除し表示用 signal は維持。E2E 250 件・全ゲート成功 / ADR-0193。
- [x] 【TD-M09】Reader の on_cleanup で残余 object URL / cover URL を revoke。
      SendWrapper パターンを再利用、既存 revoke 維持。E2E 249 件・全ゲート成功 / ADR-0192。
- [x] 【PR-M07】Google Fonts の preconnect / stylesheet を削除し CSP をローカル完結化。
      verify.sh に no-external-font-refs ゲートを追加。E2E 249 件・全ゲート成功 / ADR-0191。
- [x] 【PR-M04】read の上限を 256 MiB（または capabilities 指定）へ fail-closed 適用。
      clamp せず validation エラー、1 byte probe で全読も検出。unit 5 件（core 246）。
      E2E 249 件・全ゲート成功 / ADR-0190。
- [x] 【PR-M01 / TD-M04】Ollama ストリームをバイト累積＋行キューへ変更。
      error / 不正 JSON / 不正 UTF-8 / 1 MiB 超過を fail-closed、AiConfig.timeout_secs を配線。
      unit 14 件、core 241・E2E 249・全ゲート成功 / ADR-0189。
- [x] 【PR-M06】atomic write の tmp 名を `<file>.<pid>.<counter>.tmp` へ一意化。
      create_new + 128 回リトライ、失敗時削除。unit 4 件（core 228）。
      E2E 249 件・全ゲート成功 / ADR-0188。
- [x] 【PR-M02】table 数値属性（colspan/rowspan/start/value/span）を fail-closed 検証。
      不正値は属性のみ削除、unit 4 件追加（core 224）。E2E 249 件・全ゲート成功 / ADR-0187。
- [x] 【PR-M03 / PR-M05】設定 IPC 境界を DTO 正規化へ統一。
      extra_headers / 秘密 / パスの注入を遮断し、ヘッダはキー単位マージ。
      update_ai_config の api_key は ADR-0098 契約で受容。
      core 220・desktop 62・E2E 249・全ゲート成功 / ADR-0186。
- [x] cargo-audit 未インストール時に audit ゲートを fail-closed 化。
      PATH 隠し検証で HIDDEN_RC=1、E2E 未実行を確認。E2E 249 件・全ゲート成功 / ADR-0185。
- [x] 【TD-M01】image_dimensions に JPEG / VP8X / VP8 と境界値テストを追加。
      imagesize 0.15 のヘッダー配置と突合、core 220 件・E2E 249 件・全ゲート成功 / ADR-0184。
- [x] 【TD-M02】wait_until の Python 例外を再送出するよう変更。
      一時スクリプト 4 ケース成功、E2E 249 件・全ゲート成功 / ADR-0183。
      JS 例外の Python 例外化は根本課題として未決定に記録。
- [x] 【TD-M03】E2E Checker に check 名の重複検知を追加。
      AST 解析で 249 件すべて一意、重複 0。E2E 249 件・全ゲート成功 / ADR-0182。
- [x] 【PR-H06】SVG サニタイズを core の ammonia 共通実装へ一本化。
      DomParser 手動実装を削除し、unit 4 件追加（core 216）。
      E2E 249 件・全ゲート成功 / ADR-0181。
- [x] 【TD-H06】下書きをバックアップの optional `drafts` field へ追加。
      version:1 互換を維持し、1 MiB 超過除外・現在値保護を適用。
      E2E 249 件・全ゲート成功 / ADR-0180。
- [x] 【PR-H07】README の ADR 一覧を `docs/decisions/index.md` と整合。
      ADR-0098〜0131 の説明 34 件を正しい行へ移動し、0132 には自身の説明のみ。
      markdownlint 0 issues / コード変更なし・E2E 248 維持。
- [x] 【TD-H05】エディタ下書きを 800ms デバウンス自動保存し、未保存表示を追加。
      タイマー cleanup と世代ガードで古い発火を破棄。beforeunload は E2E 阻塞のため見送り。
      E2E 248 件・全ゲート成功 / ADR-0179。
- [x] 【TD-H04】can_watch を false 化し、未登録の watch_directory 関数も削除。
      capability / IPC / 実装を一致。core 212 件・E2E 247 件・全ゲート成功
      / ADR-0178。
- [x] 【PR-H04】設定保存と AI service 初期化を prepare/commit に分離。
      生成失敗時に config / disk / service を不変とし、保存成功後のみ service を置換。
      desktop 59 件・E2E 247 件・全ゲート成功 / ADR-0177。
- [x] 【PR-H03 / TD-H03】EPUB ロードへ独立した load_generation を導入し、
      古い完了の UI 上書きを遮断（ガード 10 箇所）。E2E 247 件・全ゲート成功
      / ADR-0176。E2E 追加は再現フックが無く 247 維持（理由は ADR 記載）。
- [x] 【PR-H05】設定リセットの保存失敗上書きを修正。save_settings を bool 化し、
      成功時のみ成功文言を表示。E2E 247 件・全ゲート成功 / ADR-0175。
      旧 :3883-3924 の setItem オーバーライド try/finally 欠如は未決定として継続。
- [x] 【PR-H02 / TD-H02】`read_text` の不正 UTF-8 を U+FFFD 置換で返すよう修正。
      有効 UTF-8 はゼロコピー、エラー文言への内容埋め込みを廃止。
      unit 2 件追加（core 210 件）、`./scripts/verify.sh` rc=0 / E2E 246 件成功
      / ADR-0174。
- [x] クロスレビュー 5 ラウンドを実行（セキュリティ／アーキテクチャ／
      運用保守／テスト品質／ドキュメント整合、全 19 観点）。
      すべて読み取り専用エージェントで、プロダクションコードは未変更。
- [x] 第 6 回で 5 回分と並行セッションの `final-report.md` を統合。
      重複を初出 ID へ集約し、妥当性を評価した。
- [x] 2026-11-19: `swarm-project-review` 5 観点と `swarm-todo-discovery` 5 観点を
      並列実行し、R6 で重複排除・根拠確認・重大度再判定のうえ未着手へ追記。
      エージェントは 10 体、深さ 1、書き込みは `todo.md` のみ。
- [x] 【PR-H01 / TD-H01】`WriteOptions.overwrite` を実効化。
      `core/src/io.rs` で既存ファイルを拒否し、root 相対パスのみをエラーへ記載。
      unit 6 件追加（core 208 件）、`./scripts/verify.sh` rc=0 / E2E 246 件成功
      / ADR-0173。TOCTOU は未決定として継続。

## メモ

- 2026-11-19: PR-M08 / TD-M08 は 2 段階で完了（TD-M08a: WasmConfig 削除 / ADR-0205、
  PR-M08: 3 レイヤー分離の設計受容 / ADR-0211）。設定モデル統合の残課題は
  `todo.md` の未着手ではなく ADR-0211 の「未決定」へ集約した（追跡先は同 ADR）。
- 2026-11-19 R6: レビュー 5（architecture / error-handling / input-validation /
  cicd / documentation+ergonomics）と探索 5（static / feature / test / deps /
  value）を同時起動。子は読み取り専用で、秘密情報・子間通信・shell は禁止。
- 2026-11-19 却下: `@v1` の SHA ピン留め提案は共通指示 5 がタグ `v1` を規定
  しているため却下（既存の却下理由を維持）。advisory DB 鮮度と billing は
  外部要因のため既知として扱い、新規 TODO に重複計上しない。
- 2026-11-19 補足: 既存の高優先度項目のうち path validation / secret DTO /
  AI 既定無効 / audit gate / CSP / config patch / secret clear / audit log 等は
  既に ADR とコミットで解消済み。着手前に現 HEAD の ADR を確認すること。
- 2026-11-19: 監査で解消済みと判定した TODO を ADR 実績へ照合し、未着手から完了へ
  移動した。CI-H01 は billing 依存の外部要因として低優先度へ移し、E2E は 253 で統一。
  残リスクは個別 TODO を増やさず ADR の「未決定」へ集約（ADR-0224）。

- エージェント動員: Round 1 = 4 観点（初回 3 件失敗 → 再委譲で回収）、
  Round 2 = 4、Round 3 = 4、Round 4 = 4、Round 5 = 3。
  合計 19 観点・22 回起動。全エージェントは `read` / `glob` / `grep` のみ。
- 却下した誤検知:
  1. ADR-0083 リンク切れ → 実在し git 追跡済み。
  2. `@v1` をコミット SHA へピン留め → 共通指示 5 はタグ `v1` を規定。
  3. capability 不在による IPC 全拒否 → 並行レポートが既に却下。
     本 TODO のパス検証指摘は ACL ではなく Rust 側の問題で別項目。
  4. SVG 内 CSS 未除去 → `svg` / `math` は許可リスト外で style も除去。
  5. `architecture.md` の status 更新推奨 → 注記が「提案時点の記録として
     残す」と明記しており既知。
- 未確認: 呼び出し先 `rust-gates.yml@v1` の入力実体、wasm ゲートの
  二重指定の有無、CI の実際の実行状態、外部脆弱性情報の照合、
  Tauri v2 capability の実挙動、EPUB 2 / 固定レイアウトの実データ。
- ゲート表記: README は数字を記載せず、`.github/workflows/ci.yml:6` のコメントが
  verify.sh 側のゲート（no-external-font-refs / audit / trunk build + E2E）を定義する。
- `GOAL.md` はレビュー中に 4 回差し替わった。契約値は着手時に
  現 HEAD で再確認すること。
- 並行セッションが `docs/investigation/` と `scripts/` を追加中。
  本 TODO はその `final-report.md` の優先順位をベースに、
  本セッションの独立レビューで得た独自知見を統合したもの。
- レビュー開始時 HEAD は `5fbd114`、統合時 HEAD は `9e9dbd6`。
  その間に並行セッションの rebase と契約差し替えが発生している。
