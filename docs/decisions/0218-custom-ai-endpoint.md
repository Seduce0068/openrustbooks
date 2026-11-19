---
type: Decision
title: "ADR-0218: AI カスタムエンドポイント opt-in の設計受容"
description: "カスタム AI 送信先の opt-in を見送り、allowlist の現状契約と将来の最小安全設計を記録する。"
tags:
  - ADR
  - security
  - ai
generated:
  by: dsh
  at: "2026-11-19T13:32:49+09:00"
status: accepted
---

## 背景

`validate_base_url()`（ADR-0101）は AI の送信先を公式 OpenAI ホストと
Ollama loopback のみに制限している。カスタムホストを許可すると
`api_key` / `extra_headers` / 本文がそのまま第三者へ送信され得る。
加えて reqwest は既定で redirect を追従し、path / query の迂回も検査されない。
Web 設定フォームは読み取り専用（ADR-0128）、wasm AI は fail-closed
（ADR-0117 / 0156）で、opt-in の需要先も設定面も未整備だった（R6 TD-M17）。

## 現状契約

`core/src/ai_markdown/client.rs` の `validate_base_url()` が契約の権威である。

- userinfo（`username` / `password`）は常に拒否する。
- scheme / host / port の厳密一致。大小文字は無視するが、`ends_with` 等の
  部分一致は使わない。
  - OpenAI: `https` + `api.openai.com` + port なし / 443。
  - Ollama: `http` + loopback（`localhost` / `127.0.0.1` / `[::1]`）+ port 11434。
- path は互換性のため検査しない。
- エラーメッセージには host と理由のみを含め、userinfo を含み得る URL 全体は出さない。

適用点は 2 種類（4 呼び出し）。

- クライアント生成時: `OpenAiClient::new` / `OllamaClient::new` の先頭で検証し、
  `create_client` 経由で全経路へ適用する。
- desktop IPC 検証: `update_config` / `update_ai_config` が merge 後に同期的に検証し、
  不正なら Err を返す。

## 決定

- カスタムエンドポイント opt-in は**見送り**、設計受容とする。
- opt-in フラグは未実装のままとし、コード・設定・E2E は変更しない。
- ADR-0101 の「カスタムエンドポイントの安全な opt-in 経路」は未決定のまま継続する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 設計受容＋未決定として明記（採用） | 同意・フラグ配置・資格情報の境界が未定義で、実装しても誤用を防げない |
| JS が届く DTO にフラグを追加して即実装 | ADR-0101 の許可リストを JS から上書き可能になり、防御が無効化される |
| 設定フォームからカスタム host を編集可能にする | ADR-0128 の読み取り専用契約と衝突し、UI も未接続 |

## 将来の最小安全設計

opt-in を実装する場合、同時に次を満たすものとする（今回は実装しない）。

### フラグ配置

- opt-in フラグは環境変数、または DTO 外かつ IPC 更新不能・`merge_secrets` 保持の
  起動時読込とする。IPC が `AppConfig` 全体を受け取るため、
  JS が届く設定に置くと allowlist と同じく書き換えられ、無意味になる
  （ADR-0101:45 と整合）。

### 同意

- Rust ネイティブダイアログで host + path と送る秘密の種類（API キー、
  extra headers、本文）を明示し、再同意を要求する（ADR-0104 の拡張）。
  秘密の値と query は画面に出さない。

### 許可対象と接続

- 既定 allowlist を維持し、明示登録した host のみ追加する。
- 非 loopback は `https` 必須、loopback も port を明示する。
- userinfo 拒否を維持する。
- path / query / fragment は拒否または正規化し、迂回を防ぐ。
- redirect は無効化し、追従する場合は送信前に宛先を再検証する。

### 資格情報と wasm

- 公式 host と API キーをバインドし、登録済みの公式 host にのみ既存キーを使う。
- カスタム先には専用 credential を使い、公式用キーを転用しない。
- wasm には追加しない（fail-closed を維持、ADR-0117 / 0156）。

## 根拠

- ADR-0100: AI 無効化後の service 残存を防ぐガード。送信先差し替えと同型のリスク。
- ADR-0101: 現状の allowlist 契約と未決定（カスタム opt-in 経路、wasm 適用）。
- ADR-0102: `extra_headers` の IPC マスク。カスタム先ではマスク対象が送信される。
- ADR-0104: 送信前のネイティブ同意ダイアログ。再同意の設計基盤。
- ADR-0117: wasm AI 既定値の整合（ADR-0205 で WasmConfig は削除済み）。
- ADR-0128: 設定フォームの読み取り専用化。UI 側から host を変更できない。
- R6 TD-M17: 本件的対処。
- UI 未接続: カスタム host を入力・保存する経路が存在しない。
- wasm fail-closed: ブラウザ側に送信先も API キーも配信しない設計。
- reqwest の既定 redirect 追従: 検証済み URL と実際の送信先が乖離し得る。
  async-openai 0.42.2 は `reqwest::Client::new()` を使い（reqwest 0.13 依存）、
  reqwest の既定は最大 10 回の redirect 追従（同期・非同期とも同じ既定）。

## 影響

- コード・設定・E2E は不変。E2E 253 件を維持する。
- Azure OpenAI / OpenRouter / LiteLLM 等のカスタムエンドポイントは引き続き拒否される。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- `core/src/ai_markdown/client.rs:106-149` を読み、現状契約（userinfo 拒否、
  厳密一致、path 未検査、host のみのエラー文言）をコードで確認した。
- redirect 設定がクライアントに存在しないことも同ファイルの grep で確認した
  （reqwest 既定の追従が残る）。
- `Cargo.lock` の async-openai 0.42.2 とローカルレジストリの
  `async-openai-0.42.2/src/client.rs:112,140`（`reqwest::Client::new()`）、
  `reqwest-0.13.5/src/redirect.rs:163`（`Policy::limited(10)`）で根拠を確認した。
- 文書のみの変更のため compile / test は省略。既定ゲートは不変。
- markdownlint-cli2（`/tmp/mdlint.jsonc`）: 全対象 0 issues。

## 未決定

- opt-in フラグの配置（環境変数 / 専用セクション / その他）。
- 同意の粒度（送信ごと / host ごと / セッションごと）。
- 登録 host の形式（完全一致文字列 / 正規化ルール / 設定ファイルのスキーマ）。
- DNS 再解決と redirect の扱い（都度再検証 / 無効化）。
- path / query / fragment の拒否または正規化の詳細。
- credential のスコープ（公式 / カスタムの分離、ローテーション）。
- 起動時の通知（カスタム host が有効な旨の告知方法）。
- wasm に fetch ベースの AI クライアントを実装する際の同設計の適用
  （ADR-0101 未決定を継続）。
- 登録 host の失効 / kill switch と、登録・変更・使用の監査ログ。
- 既存 `config.toml` に保存済みのカスタム `base_url` の移行時挙動
  （拒否のままか、明示登録を促すか）。
- ブラウザ版の同意UI（ADR-0104 は Tauri ネイティブ前提、ADR-0117 の wasm fail-closed と整合）。
- `extra_headers` のバインド（カスタム先へどのヘッダー名・値を渡すか、
  ADR-0102 のマスク対象との境界）。
