---
type: Todo
title: "openrustbooks プロジェクトTODO"
description: "openrustbooks 固有のタスクを管理するプロジェクト内 TODO。クロスレビュー5回と第6回統合の結果を反映。"
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
- 【PR-H02 / TD-H02】`read_text` が不正 UTF-8 の内容断片をエラー文言へ埋め込む。
  根拠 `core/src/io.rs:432-439`。lossy 変換して返すか、理由のみのエラーにする。
- 【PR-H03 / TD-H03】EPUB 非同期ロードに世代管理が無く、古いファイルが新しい選択を上書きする。
  根拠 `web/src/lib.rs:2628,2660,2788,2805`。
- 【PR-H04】設定保存と AI 再初期化が分かれてコミットされ、失敗時に中途半端な状態になる。
  根拠 `desktop/src/commands/config.rs:524,544,565,655`。
- 【TD-H04】`can_watch=true` を報告しつつ `LocalStorage::watch()` は常に未実装エラー。
  根拠 `core/src/io.rs:291,571-574` / `desktop/src/commands/fs.rs:145-157` /
  `desktop/src/main.rs:111`。実装するか `can_watch=false` とコマンド非公開化。
- 【TD-H05】エディタ下書きが画面遷移・リロードで失われる（自動保存／未保存表示／beforeunload 無い）。
  根拠 `web/src/lib.rs:219-220,248-272,281-292` / `docs/requirements.md:159`。
- 【TD-H06】エディタ下書きがバックアップのエクスポート／インポート対象外。
  根拠 `web/src/lib.rs:1397-1409,1716-1777,1843-1855,1988`。
- 【PR-H05】設定リセットが保存失敗 status を成功メッセージで上書きする。
  根拠 `web/src/lib.rs:3256-3267` / `web/src/lib.rs:2190-2211`。
- 【PR-H06】SVG サニタイズが core（ammonia）と web（DomParser + 手動）で二重実装され drift リスク。
  根拠 `core/src/sanitize.rs:124` / `web/src/lib.rs:940,1040,2298`。
- 【PR-H07】README の ADR 一覧で 0098〜0131 の説明が ADR-0132 に入れ子になっている。
  根拠 `README.md:287-356` / `docs/decisions/index.md:199-268`。
- 【CI-H01】CI が trunk build・配布パス検査・E2E(246件) をマージゲートに入れていない。
  根拠 `.github/workflows/ci.yml:24,42-44` / `scripts/verify.sh:64-121`。
  CI billing 解消後に共通ワークフロー側で対応。

- 【独自・最優先】Tauri IPC の任意パス読書き削除を防ぐ Rust 側パス検証。
  `core/src/io.rs:268-274` の `resolve_path` が絶対パスを無条件採用。
  `desktop/src/commands/fs.rs:60` と `document.rs:25,102` は root を経由せず
  `tokio::fs` を直呼び。UI 未接続(A05)の裏側にあり、UI 接続時に Critical 化。
  A05 と同時に `canonicalize` + root 対照を実装。
- 【D05 / R1#3 / R3#1】API キーとクライアントシークレットの平文保存・
  JS 露出を解消。`core/src/config.rs:103,224-226,341-371` と
  `desktop/src/commands/config.rs:15`。`serde(skip_serializing)`、
  マスク済み DTO、OS キーチェーン移行。
- 【D06】同意なき本文送信を停止。AI 既定無効化、送信プレビューと
  同意ダイアログを追加。
- 【B03】既知脆弱性 2 件（ammonia 4.2.5 / pulldown-cmark 0.13.6）の修正と
  配布 WASM の再ビルド。`cargo audit` を CI ゲートへ組み込む。
- 【E01】配布 WASM に含まれる開発ホスト絶対パス 406 件を除去。
  `--remap-path-prefix` / `-C strip=symbols` と検出ゲートを追加。
- 【B09】履歴 1 件で localStorage が更新されないバグ。
  `save_history` の早期リターンを廃止し E2E を追加。
- 【独自 R2-arch-state#1】Markdown 保存時に front matter が失われる。
  `desktop/src/commands/document.rs:57-60,98-102` と
  `core/src/document.rs:236` の `as_text()` が body のみを返す。
  保存時に front matter を再シリアライズして先頭へ戻す。
- 【A05 + B01 + B02】Tauri コマンドの UI 接続と CSP 強化を同時に実施。
  片面だけの修正は攻撃チェーンを残すため順序に注意。

### 優先度: 中

<!-- R6-2026-11-19 中優先度 -->
- 【PR-M01 / TD-M04】Ollama ストリームがチャンク境界で JSON 行を失い、HTTP タイムアウトも無い。
  根拠 `core/src/ai_markdown/client.rs:332,437-452`。行バッファと timeout を追加。
- 【PR-M02】table の span/start/value 属性に img 同様の fail-closed 検証が無い。
  根拠 `core/src/sanitize.rs:98-106,135,152`。
- 【PR-M03】`update_config` が完全な `AppConfig` を受け、DTO 外の `extra_headers` を JS から注入できる。
  根拠 `desktop/src/commands/config.rs:121,432,516,521,540`。ConfigPatch 経路へ統一。
- 【PR-M04】`read` の offset/limit に上限が無く `max_file_size` も未適用。
  根拠 `core/src/io.rs:34,108`。
- 【PR-M05】`are_headers_placeholder` が any のため一部マスクで全ヘッダ更新が捨てられる。
  根拠 `desktop/src/commands/config.rs:413-432`。
- 【PR-M06】atomic write の tmp パスが固定で並行書き込みが衝突する。
  根拠 `core/src/io.rs:453-456`。
- 【PR-M07】Google Fonts への preconnect が CSP 対象外で外部送信される。
  根拠 `web/index.html:12-14`。
- 【PR-M08 / TD-M08】設定モデルが AppConfig / WasmConfig / ReaderSettings / SettingsView の 4 重。
  根拠 `core/src/config.rs:14` / `core/src/wasm.rs:645` / `web/src/lib.rs:1501`。
- 【PR-M09】auto_fix 分岐の validate_content が native と wasm で重複。
  根拠 `core/src/ai_markdown/mod.rs:190-192` / `core/src/wasm.rs:534-556`。
- 【PR-M10】AppState が設定・文書・AI service・保存を抱え更新トランザクションが無い。
  根拠 `desktop/src/state/mod.rs:12,76,136`。
- 【PR-M11】設定フォームは読み取り専用契約なのに UI は双方向編集可能。
  根拠 `web/src/lib.rs:3656,3765,3876,3896,3930`。
- 【PR-M12】THIRD_PARTY_NOTICES が imagesize のみで配布依存を網羅していない。
  根拠 `THIRD_PARTY_NOTICES.md:1-4,32-59`。
- 【TD-M01】image_dimensions の JPEG/WebP ヘッダーと境界値テストが欠落。
  根拠 `core/src/image_dimensions.rs:9-18,46-72`。
- 【TD-M02】E2E `wait_until` が JS 評価例外を握りつぶし、戻り値も無視される箇所がある。
  根拠 `tools/verify-web-reader.py:463-472,609`。
- 【TD-M03】Checker がチェック名の一意性を検証せず、総数だけでは形骸化する。
  根拠 `tools/verify-web-reader.py:445-451`。
- 【TD-M05】wasm-pack test --node が CI ゲートに含まれていない。
  根拠 `.github/workflows/ci.yml:42-43` / `docs/decisions/0119`。
- 【TD-M06】rumdl の `default-features=false` が CI で静的検査されていない。
  根拠 `core/Cargo.toml:65,71` / `docs/decisions/0168`。
- 【TD-M07】Leptos の非公開 re-export `__reexports::send_wrapper` に依存。
  根拠 `docs/decisions/0166:28` / `web/src/lib.rs:224-242`。
- 【TD-M08b】figment の env マッピングが手動上書きワークアラウンドのまま。
  根拠 `docs/decisions/0122:25,50,65`。
- 【TD-M09】Reader アンマウント時に Blob URL が解放されない。
  根拠 `web/src/lib.rs:2129,2303,3464`。
- 【TD-M10】壊れた画像が章ごとに 2 秒のセトリング待ちを引き起こす。
  根拠 `web/src/lib.rs:392,544,823,1162`。
- 【TD-M11】status 文言が日英混在し、エラーと通常情報が単一 aria-live に混在。
  根拠 `web/src/lib.rs:1342,1391,2664,3284,3591`。
- 【TD-M12】内部エラーの Debug 表現が status へ流出。
  根拠 `web/src/lib.rs:2664,2672,2687,3051,3755`。
- 【TD-M13】prefers-reduced-motion 未対応。
  根拠 `web/index.html:108,134,189,549-587`。
- 【TD-M14】依存更新の自動検出（Dependabot / Renovate）未設定。
  根拠 `.github/workflows/ci.yml:1`。
- 【TD-M15】rust-version（MSRV）未宣言で CI にも MSRV ゲートが無い。
  根拠 `Cargo.toml:9,11`。
- 【TD-M16】検証済み配布物が保持されずデプロイ／復旧の糸口が無い。
  根拠 `scripts/verify.sh:64-65` / `.github/workflows/ci.yml:22-44`。
- 【TD-M17】AI カスタムエンドポイント opt-in が将来課題のまま。
  根拠 `core/src/ai_markdown/client.rs:105-113`。
- 【TD-M18】ストリーミング API が実装済みだがパイプラインから呼ばれない。
  根拠 `core/src/ai_markdown/client.rs:255` / `core/src/ai_markdown/pipeline.rs:108,119`。
- 【TD-M19】EditorConfig / ReaderConfig が死設定。
  根拠 `core/src/config.rs:291,342` / `web/src/lib.rs:3700,3716`。
- 【TD-M20】OpenAI extra_headers / Ollama options がクライアントへ渡っていない。
  根拠 `core/src/ai_markdown/client.rs:203-206,328,352`。
- 【TD-M21】updater プラグインが初期化されるが更新チェック未配線。
  根拠 `desktop/src/main.rs:38,63` / `desktop/tauri.conf.json:32`。
- 【TD-M22】ZIP リーダーが core 実装済みで WASM API / UI 未公開。
  根拠 `core/src/archive.rs:68,158,168` / `core/src/wasm.rs:62`。

- 【D03 / R2-crates#1】wasm `init()` の二重定義を解消。
  `core/src/lib.rs:18-19,31` の明示定義と `core/src/wasm.rs:28` の
  `#[wasm_bindgen]` 版が衝突。Rust 側から wasm.rs 版を呼べない点に限定的。
- 【B01 / E03 / R1-sanitize#1,#2】Web CSP meta と外部 CSS の SRI、
  SVG のプロトコル相対参照(`//`)拒否、複数 XML PI の削除を追加。
- 【R1-ai#2,#5 / R2-ai#3,#4】死設定（timeout_secs / max_tokens_per_month /
  streaming / リトライ回数の誤導出）を配線するか削除。
  `core/src/ai_markdown/pipeline.rs:27` は専用リトライ設定へ変更。
- 【R3-config#2,#3,#4】インポートのアトミック化と部分インポートの
  ロールバック。設定保存のメモリ・ディスク乖離を解消。
- 【R4-unit#3】設定・ストレージ・AI 主要モジュールのテストを追加。
  figment の優先順位、保存ラウンドトリップ、未検出バックエンドを検証。
- 【R4-e2e#1,#2 / C06】E2E に期待総数を宣言し未実行 check を失敗扱いに。
  CDP 接続先を起動 PID と照合し他 Chrome への誤接続を防ぐ。
- 【B04-B07 / C01】UX と a11y。IME の `isComposing` 除外、
  モーダル中のショートカット停止、reduced-motion、コントラスト、
  Chrome 単一検証の多ブラウザ化。
- 【C05 / R3-error#3,#5】エラー文言の二重付与と status 分離、
  `desktop/src/main.rs:80` の `unwrap()` を Result 化。
- 【D01 / D02 / A09 / R4-trace#1,#4 / R5】初版スコープ ADR、
  既存ソリューション選定根拠 ADR、REQ→ADR→テスト対応表、
  測定可能な受け入れ基準を整備。
- 【C07 / R5-adr#3】ADR-0081 の未決定を ADR-0082 の解決結果へ更新。
- 【A09 / R3-ci#1】README のゲート表記「5」を「6」へ統一、
  構成ツリーの ADR 一覧を 0083 まで更新、EPUB 状態表記を一意化。
- 【B10 / C08】未使用依存と `__prune_*` デバッグ関数を削除。
  ADR-0083 の dead code cleanup と調整する。
- 【R2-ai#2】AI クライアントの注入点（`Arc<dyn AiClient>`）を追加し
  単体テスト可能性を確保。
- 【R1-ai / R2-ai#4】lint auto-fix の no-op（`lint.rs:128`）を解消。
- 【R5-adr#2】ADR-0044 の未決定（XML PI）を ADR-0045 の決定で
  解決済みへ更新。
- 【C04 / R5-ops#1】docs 配下に `index.md` / `log.md` を整備し
  OKF・索引運用を明文化。
- 【R4-unit#5】エラー検証ヘルパー（`archive.rs:383-408`）を集約。

### 優先度: 低

<!-- R6-2026-11-19 低優先度 -->
- 【PR-L01 / TD-L01】本文領域に `aria-label` と `aria-labelledby` を併記。
  根拠 `web/src/lib.rs:3628` / ADR-0058。aria-label を削除。
- 【PR-L02】ダーク切替に `aria-pressed` が無く隣接トグルと不一致。
  根拠 `web/src/lib.rs:3509-3511`。
- 【PR-L03】build_app が desktop/mobile で 7 プラグイン重複。
  根拠 `desktop/src/main.rs:16-18,42-44`。共通関数へ抽出。
- 【TD-L02】`TokenStream::is_done` が常に false で呼び出し側ゼロ。
  根拠 `core/src/ai_markdown/client.rs:82-85,315-317,458-460`。削除または実装。
- 【TD-L03】`available_space` が常に None で呼び出し側ゼロ。
  根拠 `core/src/io.rs:154,543-546`。
- 【TD-L04】Reader の CoverImage.id / media_type が allow(dead_code) で捨てられる。
  根拠 `web/src/lib.rs:2119-2126,2731-2738`。
- 【TD-L05】E2E のコメントが期待総数 245 のまま（`Checker(expected=246)` と不整合）。
  根拠 `tools/verify-web-reader.py:820,850`。
- 【TD-L06】AI コマンドの confirmation 異常系・入力変換不正値が未テスト。
  根拠 `desktop/src/commands/ai.rs:36,54,145,172,258`。
- 【TD-L07】要件の検証条件と E2E check 名のトレーサビリティ欠損。
  根拠 `docs/requirements.md:142,144,164` / `tools/verify-web-reader.py:820`。
- 【TD-L08】serde_yaml 0.9.34+deprecated / tikv-jemalloc 残存は上流 optional 化待ち。
  根拠 `Cargo.lock:5894` / `docs/decisions/0155:45,69`。受容リスクを ADR に明記済み。

- 【R1-tauri#8 / R3-release#1,#2】updater の公開鍵・署名設定と
  リリース専用ワークフロー（`contents: write` + 署名 secrets）。
- 【R3-release#3】バージョン二重管理（Cargo workspace と tauri.conf）
  のドリフト対策。
- 【R3-release#4,#5】導入・更新・ロールバック手順、既知問題の開示、
  署名秘密鍵の `.gitignore` 登録。
- 【R3-config#5】容量超過時の位置削除を通知・退避方式へ変更。
- 【R4-e2e#3,#4 / C08】swipe スタブの二重発火、
  固定 sleep と待機例外の握り潰しを条件待ちへ統一。
- 【R2-epub#1-#4】EPUB 層の API・エラー・所有権境界を整理。
  `open_owned` の limits 指定、1316 行ファイルの分割、欠落エラーの統一、
  Document 側の二重定義解消。
- 【R2-crates#2-#5】ADR-0003 の rumdl feature 記述更新、
  未実装 Wasm API の非公開化、core の Web バインディング分離、
  workspace 依存の一元化。
- 【R1-archive#2-#5】実展開サイズの累計上限、入力パーサ側の深さ上限、
  正規化後同名エントリの拒否、表紙 media-type 検証。
- 【R3-ci#3-#5】wasm ゲートの二重指定の有無、CI とローカルの
  ビルドプロファイル差、`trunk build` への `--locked` 付与。
- 【R5-adr 全体評価】ADR-0002 と ADR-0003 の日付逆転（推測）を確認。
- 【R4-unit#4】圧縮率に依存するアサーション（推測）を決定的性质へ変更。

## 完了

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

- 2026-11-19 R6: レビュー 5（architecture / error-handling / input-validation /
  cicd / documentation+ergonomics）と探索 5（static / feature / test / deps /
  value）を同時起動。子は読み取り専用で、秘密情報・子間通信・shell は禁止。
- 2026-11-19 却下: `@v1` の SHA ピン留め提案は共通指示 5 がタグ `v1` を規定
  しているため却下（既存の却下理由を維持）。advisory DB 鮮度と billing は
  外部要因のため既知として扱い、新規 TODO に重複計上しない。
- 2026-11-19 補足: 既存の高優先度項目のうち path validation / secret DTO /
  AI 既定無効 / audit gate / CSP / config patch / secret clear / audit log 等は
  既に ADR とコミットで解消済み。着手前に現 HEAD の ADR を確認すること。

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
- 「5 ゲート vs 6 ゲート」: `README.md:149` と `.github/workflows/ci.yml:3`
  は「5 ゲート」と記載。実際の検証コマンドは 6 行
  （fmt / check / test / clippy / wasm32 × 2）。
- `GOAL.md` はレビュー中に 4 回差し替わった。契約値は着手時に
  現 HEAD で再確認すること。
- 並行セッションが `docs/investigation/` と `scripts/` を追加中。
  本 TODO はその `final-report.md` の優先順位をベースに、
  本セッションの独立レビューで得た独自知見を統合したもの。
- レビュー開始時 HEAD は `5fbd114`、統合時 HEAD は `9e9dbd6`。
  その間に並行セッションの rebase と契約差し替えが発生している。
