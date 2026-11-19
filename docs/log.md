# log

## 2026-11-20

- R6 ドキュメント整合: E2E 件数の current 表記をコード出典（255）へ追従。
  `Checker(expected=255)`（check 238 + equal 17）と
  `docs/e2e-traceability.md` の 255 件は一致していたが、`todo.md` の CI-H01 と
  `docs/documentation-operations.md` の出自節に 254 件が current 値として残存。
  current は 255 件とし、254 件（check 237 + equal 17）は ADR-0225、
  253 件は ADR-0220 の履歴更新として保持。出典はコードのみ（ADR-0090）、
  同名重複検知は ADR-0182、255 件への更新根拠は ADR-0245。
  ADR-0262 を追加し decisions / docs 各 index（262 件）、README、todo を追従。
  コード / E2E check / E2E 件数は未変更（255 維持）。markdownlint 0 issues（ADR-0262）。
- R6 D05: OS キーチェーン移行を設計受容として記録。平文 config.toml 保存 +
  redaction（REDACTED_SECRET / DTO has_* / Debug マスク）+ Unix 0600 +
  `update_ai_config` の `api_key` のみ IPC 書き込み可を受容済みリスクとして明記。
  keyring（新規直接依存・target feature）/ secret-service（dbus/zbus 再利用だが
  新規直接依存）/ 独自 file（平文残留）/ 設計受容を比較し、設計受容を採用。
  Wasm では config 永続化自体が無いこと、CI は secret 不使用で env 注入不可のため
  実キーストア検証ができないこと、owner-only は Unix のみ（非 Unix fallback は
  ACL 未設定）を根拠に、移行順序と失敗時挙動・ローテーション・プラットフォーム差・
  Wasm / localStorage の平文性・CI 検証・keyring の target/feature 一次確認を
  ADR-0098 / ADR-0261 の未決定へ集約。`core/src/config.rs` の "encrypted" コメントは
  暗号化なし（Unix 0600 のみ）と矛盾することを開示し、コードは未変更。
  ADR-0261 を追加し decisions / docs 各 index（261 件）、README、runbook-release、
  todo を追従。コード / Cargo.toml / Cargo.lock / E2E は未変更で E2E 255 維持。
  markdownlint 0 issues。
- EPUB 本編を opf/navigation/text/xmlutil へ分割し epub.rs を 1884→556 行へ削減（ADR-0260）。
- Wasm バインディング本編を epub/archive/lint へ分割し wasm.rs を 537→134 行へ削減（ADR-0259）。
- R6 R2-crates#5: feature が一致しない chrono / uuid / wasm dev tokio を workspace 化せず
  現状を設計受容。chrono / uuid は member 側が feature を加算し、wasm dev tokio は
  member 側が限定するため、workspace 化すると逆方向へ feature が加算される。
  Cargo の feature は和集合で member 側から削除できないことを根拠に、
  desktop chrono へ wasmbind / uuid js の波及と wasm dev tokio への full 加算を記録。
  rumdl は workspace に無く ADR-0242 / check-rumdl-featureless.sh で除外。
  加算による具体的なビルド影響と MSRV 1.95 の override 制限は未検証の推測として
  断定せず、ADR-0242 / 0246 を継承。ADR-0258 を追加し decisions / docs 各 index
  （258 件）、README、todo を追従。Cargo.toml / Cargo.lock / コードは未変更で
  E2E 255 維持。markdownlint 0 issues。
- WASM binding の 14 tests を wasm/tests.rs へ切り出し、本編を 912→537 行へ削減（ADR-0257）。
- 未使用の Wasm AI cluster（`WasmAiMarkdownService` / `WasmPipelineResult` /
  `WasmDiffResult`）と自己言及 test `ai_methods_fail_closed` を属性ごと削除。
  追跡ソース（web / E2E / wasm test）の参照 0 件、型互換の根拠消滅、
  `WasmDiffResult` は未構築、削除は fail-closed より強いことを根拠とする。
  lint cluster は ADR-0160 実装済みのため維持し、ADR-0156 の AI 部分のみ supersede。
  ADR-0156 / 0227 / 0232 / 0243 / 0244 / 0246 を追従（status は `accepted` 維持）。
  wasm-bindgen-test 15 → 14 件、core 284 維持、E2E 255 維持。
  外部 JS 利用者の有無、deprecation shim、バンドルサイズ実測は未決定（ADR-0256）。
  decisions / docs 各 index（256 件）、README、todo を追従。markdownlint 0 issues。
- EPUB の 92 tests を epub/tests.rs へ切り出し、本編を 3780→1884 行へ削減（ADR-0255）。
- R6 R3-release#4,#5: 運用文書を先行開示し、公開・署名・endpoint は未決定として残す。
  `docs/runbook-release.md` を追加し、現状（公開 installer / updater / 署名済み成果物 /
  公開 endpoint は無い、設定表示は「更新確認: 未配線」、check_updates は死設定）と、
  検証済み Web dist の導入・更新・ロールバック手順、既知問題（updater 未配線 /
  browser AI 無効 / 設定 readonly / 実機 5 ブラウザ未検証 / CI billing /
  OS キーチェーン未配線）を根拠 ADR 付きで開示。
  署名秘密鍵はリポジトリ外 / CI secrets に置き、`.gitignore` での拒否を完了。
  公開鍵設定・署名 secrets・endpoint・billing・Release workflow・
  require_signed_version・実機検証は未決定（ADR-0254 / 0222 / 0219 / 0232）。
  ADR-0254 を追加し decisions / docs 各 index（254 件）、README、todo を追従。
  コード・CSP・tauri.conf・公開鍵・秘密鍵・Release workflow は未変更。
  markdownlint 0 issues。
- R6 A05 残の Tauri 設定保存 UI を段階設計として決定。browser 単体は読み取り専用
  fail-closed を維持し、desktop のみ最小 IPC（get_config + update_config_patch）と
  実装前提 6 段階（withGlobalTauri + CSP / capability 最小権限 /
  ConfigUpdateRequest.expected + 構造化エラー {code,message} + 最新 ConfigView /
  a11y 契約 / E2E 成功経路 +1 / 実 Tauri WebView 検証）を順に満たして接続する。
  reset / clear は別操作、fs / ai / window / updater は設定画面から禁止。
  AI / Lint / format は初版外を維持。ADR-0128 / 0193 は desktop 側のみ supersede し、
  browser 側は維持。decisions / docs 各 index（253 件）、README、todo を追従。
  コード・CSP・capability・E2E は未変更で E2E 255 件は不変（ADR-0253）。

## 2026-11-19

- ZIP パスの深さ上限を追加せず設計受容を記録。normalize_path / resolve_reference の
  無制限なコンポーネント数は、非再帰性・ZIP 名 u16 長・256 MiB 入力 /
  4096 entries / 64 MiB per entry / 256 MiB total / 実展開 AtomicU64 予算・
  fail-closed 検証・サニタイザ深い入れ子テスト前例により受容（ADR-0252）。
  decisions / docs 各 index（252 件）、README、todo を追従。
- ZIP 実展開サイズへ AtomicU64 セッション予算を追加し core を 279→284 件へ拡充（ADR-0251）。
- Document 側の dead な EPUB 型を削除し EPUB 経路を EpubBook へ集約（ADR-0250）。
- 表紙 media-type 契約を unit test で固定し core を 278→279 件へ拡充（ADR-0249）。
- EPUB manifest の id/href 欠落を固定文言の fail-closed エラーへ統一（ADR-0248）。
- EPUB 所有 API へ limits 指定を追加し core を 274→276 件へ拡充（ADR-0247）。
- feature 差分の無い core 依存 18 件を workspace 参照へ統一（ADR-0246）。
- 容量超過時の位置削除へ件数・残件数を通知し E2E を 254→255 件へ拡充（ADR-0245）。
- 未使用 Wasm API 第二段として WasmDocument を削除（ADR-0244）。
- 未使用 Wasm API（greet / detect_format）を属性ごと削除し JS export 面を縮小（ADR-0243）。
- ADR-0003 に rumdl feature の実装状況注記（2026-11-19）を追加。target 別 feature
  選択は廃止され、native / wasm とも `default-features = false` であることを記録。
  ADR-0242 を追加して ADR-0003 を履歴化し、rumdl の workspace 化を行わず
  `wasm` feature は同名 `init` export 衝突で禁止、
  `scripts/check-rumdl-featureless.sh` の完全一致 2 行を維持すると決定。
  decisions / docs 各 index（242 件）、README、todo を追従（ADR-0242 / R2-crates#2）。
- ADR の `generated.at` は生成／正規化時点の記録で作成時刻や決定順ではないことを
  `docs/documentation-operations.md` へ明記。0001 / 0002 の 2026-11-17 は c030537 の
  frontmatter 正規化による付与で、git 作成順は 0001〜0005 が連番どおり（10-11〜10-17）。
  既存 240 件の値は変更せず、規約注記で日付逆転の誤解を解消。
  decisions / docs 各 index（241 件）、README、todo を追従（ADR-0241）。
- 正規化後同名 ZIP エントリを fail-closed で拒否し parser differential を防止（ADR-0240）。
- Cargo workspace をバージョンの単一情報源とし、tauri.conf の version 省略を静的ゲート化（ADR-0239）。
- deflate 圧縮率依存の test を方式／メタデータの直接検証へ変更（ADR-0238）。
- CI / ローカルゲートの整合を監査し、`wasm_targets` は `rustup target add` のみ、
  `extra_gates` が wasm check / wasm-pack test を実行して二重実行ではないと判定。
  `CARGO_PROFILE_DEV_DEBUG` / `TEST_DEBUG` / `INCREMENTAL` は runner の disk 対策として
  CI 側に留め、ローカルへ全局適用しない。`scripts/verify.sh:268` の trunk build へ
  `--offline --locked` を付与し、`ci.yml` / `ci-wasm-test.sh` は不変。
  bash -n OK・verify.sh 全ゲート rc=0・E2E 254/0・Cargo.lock 差分なし・
  markdownlint 0 issues。ADR-0237 を追加し decisions / docs 各 index（237 件）、
  README、GOAL、todo を追従。
- `prefers-contrast: more` で枠線と補助文字を強化。light border `#6b7280`（4.83:1）/
  muted `#374151`（10.31:1）、dark border `#94a3b8`（6.96:1）/ muted `#cbd5e1`（12.02:1）を
  `@media (prefers-contrast: more)` と `(prefers-contrast: more) and (prefers-color-scheme: dark)`
  で適用し、dark は `html.dark` 経路もカバー。forced-colors ブロックは変更せず後方に維持。
  静的コントラストを再計算し、E2E の色非依存（check 名 254 件不変）を確認。
  E2E 254/0・`verify.sh` rc=0・markdownlint 0 issues。ADR-0236 を追加し decisions / docs
  各 index（236 件）、README、GOAL、todo を追従（ADR-0236）。
- swipe の先頭 evaluate（引数未埋め込みの二重 dispatch）を削除し、f-string 版 1 回のみの
  dispatch へ変更。到達条件が明確な章移動・縦 / マルチタッチ・ダーク切替・ファイル選択後の
  DOM / 保存・リセット後を条件待ちへ置換（負向チェックは baseline＋第 1 章本文＋0.3 秒維持）。
  到達条件のない 106 件の sleep は据え置き、`wait_until` の fail-closed は維持。
  E2E check 名・254 件は不変で AST 254/254、`verify.sh` 全ゲート 0、E2E 254/0、
  markdownlint 0 issues。ADR-0235 を追加し decisions / docs 各 index（235 件）、
  README、GOAL、todo、e2e-traceability（概要 1 行）を追従。
- OKF / 索引 / log / lint の運用を `docs/documentation-operations.md` へ明文化し、
  リポジトリ直下に `.markdownlint-cli2.jsonc` を追加（一時ファイル依存を解消）。
  ADR-0234 を追加して decisions / docs の各 index（234 件）、README、todo を追従させ、
  AGENTS.md のグローバル参照を `~/.agents/AGENTS.md`（正本）へ更新。
  `npx markdownlint-cli2 'docs/**/*.md' README.md todo.md` は 0 issues（ADR-0234）。
- E2E トレーサビリティへ REQ / 主要 ADR 列を追加し、未カバー行も含め VC 全件を
  対応付け。VC は ADR-0233 により初版の測定可能な受け入れ基準として参照し、
  要件側の「候補」表記は維持。主要依存の選定根拠をライセンス / 鮮度 /
  既知脆弱性 / Wasm 対応で横断比較（2026-11-19 実測、cargo audit baseline
  10 warnings / 0 vulnerabilities）。E2E check 名 254 件不変、
  AST 254/254・missing / duplicate / extra 0、markdownlint-cli2 --config
  /tmp/mdlint.jsonc（8 ファイル、MD013 tables:false）0 issues（ADR-0233）。
- ADR-0232 の監査指摘（誤引用・TXT 範囲・根拠行番号・Runtime Proof）を修正。
- 初版スコープ ADR を追加し、native desktop / browser / text / AI / 保存・検証の
  実装範囲と初版に含まない項目・未決定を実装証拠（E2E 254 件・core 270 件）付きで
  明示。requirements.md に導線を追加し、各索引を 232 件へ更新（ADR-0232）。
- 未検出 backend / provider の unit test を追加し core を 267→270 件へ拡充（ADR-0231）。
- dark のプライマリボタン文字色を暗色トークンへ変更し WCAG AA を達成（ADR-0230）。
- エラー文言の二重付与と JsValue Debug 露出を解消し、desktop run() を Result 化（ADR-0229）。
- AI 死設定を設計受容し、月間上限/timeout の表示を正直化（ADR-0228）。
- AI クライアント注入点を設計受容（公開注入点は追加せず）。既存の非公開フィールド直接構築による
  モック注入を維持し、クロスクレート要件が出た場合のみ同期 `with_client` を `pub(crate)` から検討。
  コード・E2E 不変、markdownlint 0 issues / ADR-0227。
- Reader status を日本語へ統一し E2E 文言期待を同期（ADR-0226）。
- 壊れた画像の error を記録し失敗画像を待ち対象外にして、章ごとの 2 秒セトリング待ちを解消。
  新規 E2E「壊れた画像を含む章の位置復元が 1.5 秒未満で完了する」を追加し、
  期待総数を 253 → 254（check 237 + equal 17）へ更新（ADR-0225、TD-M10 完了）。
  AST 抽出で expected / found / distinct 254・missing / extra 0 を確認し、
  トレーサビリティ（新規 check を VC-004 の読書位置行へ割り当て）／索引／
  README／TODO／GOAL 契約を追従。markdownlint 0 issues。
- 監査で解消済みと判定した TODO を ADR 実績へ照合し、未着手から完了へ移動。
  D05 は IPC マスク／DTO 境界を完了、OS キーチェーン移行を未決定へ分割。
  A05 + B01 + B02 は CSP を完了、UI 接続を段階 2 として分割。
  CI-H01 は billing 依存の外部要因として低優先度へ移し、E2E を 253 で統一
  （ADR-0224）。README の ADR 0207〜0223 の説明の一行ずれも修正。
- ZIP リーダーを WASM API として公開し、wasm test を 7→14 件へ拡充（ADR-0223）。
- updater の未配線を設計受容とし、設定表示を「更新確認: 未配線」へ正直化。
  本番接続・自動チェック・自動インストールは実装せず、公開鍵 / endpoint /
  署名 secrets / Release workflow は未決定として継続（ADR-0222、TD-M21 完了）。
- OpenAI extra_headers / Ollama options を安全に配線し core を 260→267 件へ拡充（ADR-0221）。
- E2E トレーサビリティを 251 → 253 件へ更新（ADR-0206 の追従漏れを解消）。
  不正 EPUB の固定文言通知を VC-009、reduce の即時フラグメント移動を VC-004 の
  既存行へ割り当て、Python AST で check 236 + equal 17 = 253 件を抽出して
  対応表と完全一致（missing / duplicate / extra 0）。一時ファイル指定を廃し
  AST 抽出を手順化（ADR-0220）。
- `verify.sh --dist` を追加し、全ゲート成功時のみ検証済み dist をリポジトリ外へ保持。
  manifest（commit sha / verified at / rustc / trunk / e2e / source dist / SHA256 5 行）と
  取得・検証・serve・復旧の Runbook を追加（ADR-0219、TD-M16 完了）。
- AI カスタムエンドポイント opt-in を設計受容（見送り）。allowlist の現状契約と
  将来の最小安全設計を記録し、ADR-0101 の未決定を継続（ADR-0218）。
- Editor / Reader の死設定を注記で正直化し DTO 互換を維持（ADR-0217）。
- ストリーミング未接続を設計受容し、設定表示を「未配線」へ正直化（ADR-0216）。
- Dependabot で Cargo / GitHub Actions の patch / minor を週次自動検出（ADR-0215）。
- MSRV を 1.95 と宣言し CI ツールチェーンを固定（ADR-0214）。
- prefers-reduced-motion を尊重し、reduce 時も色 transition と位置復元を維持（ADR-0213、E2E 253）。
- Reader エラー status/console を固定文言化し情報漏洩を防止（ADR-0212、E2E 252）。
- AppConfig / ReaderSettings / SettingsView を権威・寿命が異なるレイヤー分離として設計受容。
  localStorage `openrustbooks:settings` と camelCase 8 項目を互換契約化し、統合を却下（ADR-0211）。
- 設定更新へ revision CAS を導入し lost update を検出（ADR-0210、desktop 74→79）。
- config.toml を tmp + rename で原子保存し、0600 を維持（ADR-0209）。
- 文書削除時に unsaved_changes を削除し、desktop を 72→74 件へ拡充（ADR-0208）。
- AppState を composition root として設計受容し、設定系トランザクションの完了を確認。残リスク 3 件を低優先度 todo へ切り出し（ADR-0207）。
- 検証条件へ VC ID を付与し E2E 251 件の対応表を追加（ADR-0206）。
- 検証条件へ VC-001〜015 を付与し、E2E check（251 件）との対応表を新規作成（R6 TD-L07）。
- WasmConfig と Wasm test 3 件を削除し、wasm テストを 10→7 件へ縮小（ADR-0205）。
- figment env を明示マップ＋生値検証へ変更し core を 248→255 件へ拡充（ADR-0204）。
- CoverImage DTO を href / bytes のみへ縮小し dead_code 警告を解消（ADR-0203）。
- AI コマンドの confirmation / format / alignment を純粋関数化し desktop を 62→72 件へ拡充（ADR-0202）。
- build_app の共通プラグイン設定を 1 関数へ集約し登録順を維持（ADR-0201）。
- send_wrapper を直接依存化し Leptos の doc(hidden) re-export 依存を解消（ADR-0200）。
- 第三者通知へ配布依存 781 件のライセンス ID と出典を追加（ADR-0199）。
- Reader の ARIA 属性を修正し E2E を 250→251 に更新（ADR-0198、ADR-0058 を supersede）。
- 未使用 trait メソッド is_done / available_space を削除（ADR-0197）。
- wasm-pack test を CI extra_gates へ追加し、verify.sh と単一情報源化（ADR-0196）。
- rumdl featureless 検査を独立スクリプト化し verify.sh と CI へ接続（ADR-0195）。
- auto_fix 分岐を lint.rs の共通関数へ集約し core を 246→248 件へ拡充（ADR-0194）。
- 設定フォームを読み取り専用化し、E2E を 249→250 に更新（ADR-0193）。
- Reader アンマウント時に残余 Blob URL を revoke し、SPA 往復のメモリ残存を防止（ADR-0192）。
- Google Fonts の外部送信を削除し verify.sh に静的ゲートを追加（ADR-0191）。
- read の要求サイズへ 256 MiB 上限を fail-closed 適用し core を 241→246 件へ拡充（ADR-0190）。
- Ollama ストリームに行バッファ・fail-closed エラー・timeout 配線を追加し core を 228→241 件へ拡充（ADR-0189）。
- atomic write の tmp 名を PID + counter で一意化し、core を 224→228 件へ拡充（ADR-0188）。
- table 数値属性を fail-closed 検証し core を 220→224 件へ拡充（ADR-0187）。
- 設定 IPC 境界を DTO 正規化へ統一し、extra_headers / 機微値の注入を遮断（ADR-0186）。
- cargo-audit 未インストール時に audit ゲートを fail-closed 化（ADR-0185）。
- image_dimensions に JPEG / WebP / 境界値テストを追加し core を 216→220 件へ拡充（ADR-0184）。
- E2E wait_until の Python 例外を再送出するよう変更し、基盤失敗の隠蔽を防止（ADR-0183）。
- E2E Checker に check 名の一意性検証を追加し、総数ゲートの形骸化を防止（ADR-0182）。
- SVG サニタイズを core の ammonia 共通実装へ一本化し、unit を 212→216 件へ拡充（ADR-0181）。
- エディタ下書きをバックアップの optional `drafts` field へ追加し、E2E を 248→249 に更新（ADR-0180）。
- README の ADR 一覧（0098〜0131）を decisions 索引と整合させ、誤入れ子を解消（R6 PR-H07）。
- エディタ下書きを 800ms デバウンス自動保存へ変更し、E2E を 247→248 に更新（ADR-0179）。
- watch 未実装に合わせ can_watch=false 化と IPC 削除を実施（ADR-0178）。
- 設定保存と AI service 初期化を prepare/commit に分離し、部分コミットを防止（ADR-0177）。
- EPUB 非同期ロードへ独立した世代カウンタを導入し、古い完了の上書きを防止（ADR-0176）。
- 設定リセットの保存失敗通知を成功文言で上書きしないよう修正し、E2E を 246→247 に更新（ADR-0175）。
- `read_text` の不正 UTF-8 を U+FFFD 置換へ変更し、エラー内容漏洩を解消（ADR-0174）。
- `WriteOptions.overwrite=false` の既存ファイル拒否を実装し、unit を 206→208 件へ拡充（ADR-0173）。
- インポート前に確認ダイアログを追加し、E2E を 245→246 に更新（ADR-0172）。
- Clear data に window.confirm() の確認を追加し、E2E を 244→245 に更新（ADR-0171）。
- 1 MiB 超の保存済み下書きを本文保持で復元し固定警告を表示、E2E を 243→244 に更新（ADR-0170）。
- docs/index.md の ADR 件数を 90 → 169 に更新し、実態と整合。
- 全消去／インポートの部分失敗を操作単位で通知し、E2E を 242→243 に更新（ADR-0169）。

## 2026-11-18

- 未使用の workspace.dependencies.rumdl を削除し、default features 復活の罠を排除（ADR-0168）。
- エディタ下書きに 1 MiB の UTF-8 バイト上限を追加し、E2E を 241→242 に更新（ADR-0167）。
- 4 つのイベントリスナーを on_cleanup + SendWrapper で解除し、forget() を 0 件化（ADR-0166）。
- 保存失敗通知を reader / editor 別イベントへ分離し、E2E を 240→241 に更新（ADR-0165）。
- README / architecture の依存一覧を comrak / ammonia / rumdl の実装事実へ更新（ADR-0163 の未決定を解消）。
- 未使用の pulldown-cmark 直接依存を削除し、Cargo.lock も整理（ADR-0163、推移依存は残存）。
- Markdown Preview を comrak + 二段サニタイズで実装し、E2E を 237→238 に更新（ADR-0162）。
- Web エディタを textarea + リアルタイム Preview の最小構成へ配線し、E2E を 235→237 に更新（ADR-0161）。
- WasmMarkdownLinter を featureless rumdl の実 lint へ配線し、wasm テストを 6→10 件に拡充（ADR-0160）。
- 部分設定インポートの false リセット懸念を調査し、現行ガード下では再現しない no-op と記録（ADR-0158）。
- インポート時に現在の設定／履歴を維持する二層マージを実装し、E2E を 233→234 に更新（ADR-0157）。
- Wasm AI/lint の偽成功スタブを明示的エラーへ変更し、wasm テストを 4→6 件に追加（ADR-0156）。
- rumdl を featureless 化して依存 28 個を削減（ADR-0155）。serde_yaml / jemalloc は non-optional のため残存。
- localStorage 保存失敗を固定文言の status と console.error で可視化し、E2E を 232→233 に更新（ADR-0154）。
- About 画面のライセンスと対応形式を実装事実に合わせ、E2E を 231→232 に更新（ADR-0153）。
- 残タスクを 5 観点のエージェントで探索し、成果を ADR-0151 に集約。
- `verify.sh` の WASM パス検査を fail-closed 化（WASM＋JS グルー、ADR-0152）。
- ADR を ADR-0091〜0150 まで追加し、`docs/decisions/index.md` を全 150 件に更新。
- 画像寸法予約: EPUB の HTML へ自然な width/height を埋め込み、img 属性を fail-closed 検証（ADR-0136/0149/0150）。
- E2E 期待総数を 229→231 に変更（寸法予約でチェック 2 件追加、ADR-0149）。
- 画像縮小を安定化: 読込後再チェック・deterministic 化・完了シグナル・連続ベースライン検証（ADR-0141/0148）。
- CDP 接続を安定化: ブラウザ級 WebSocket で対象選択、recv タイムアウト、fail-closed 診断、起動 1 回リトライ（ADR-0142〜0144）。
- 縦書きは見送り: オプトイン縦書きを却下し E2E 軸非依存化を先行、ページ分割は段階方針のみ決定（ADR-0145〜0147）。
- CI 監査ゲート: `cargo audit` を共通ワークフロー入力と `verify.sh` に追加（ADR-0124〜0126）。
- CI は billing issue で実行不可のまま、検証はローカル代替に依存（多数の ADR に重複記載）。
- CSP: Web に CSP meta を追加してブラウザ検証、Desktop は wasm-unsafe-eval で硬化（ADR-0091/0130）。
- 設定: patch コマンド、秘密削除コマンド、削除時の監査ログ（名前のみ）、隠し字段保持の一元化（ADR-0131/0132/0134/0135）。
- セキュリティ: 秘密マスク、AI 既定オフ、ベース URL 許可リスト、redaction、送信同意ダイアログ、パス封じ込め（ADR-0097〜0105）。
- AI パイプライン: rumdl 自動修正・warnings_as_errors・rule_config の配線、retry 設定と env 変数対応、ゼロリトライ回帰テスト。
- その他: ライセンスを AGPL-3.0-or-later に変更、WASM テストゲート、IME/モーダルのショートカットガード、設定フォーム配線、エディタ UI の段階統合。

## 2026-11-17

- `docs/index.md` を追加し、要件・設計・ADR・調査レポートへの導線を整備した。
- `docs/decisions/index.md` を追加し、全 90 件の ADR のタイトル・status・
  description を列挙した（ADR 本体は変更していない）。
- 共通指示の予約文件格式に従い、両ファイルはフロントマター無しの
  単一 `#` 見出しから始める。
