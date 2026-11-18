# log

## 2026-11-18

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
