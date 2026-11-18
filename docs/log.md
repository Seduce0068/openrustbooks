# log

## 2026-11-19

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
