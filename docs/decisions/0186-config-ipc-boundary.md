---
type: Decision
title: "ADR-0186: 設定 IPC 境界の強化"
description: "生 config を受け取るコマンドでも DTO 境界へ正規化し、機微フィールドの注入を防ぐ決定を記録する。"
tags:
  - ADR
  - desktop
  - security
generated:
  by: dsh
  at: "2026-11-19T05:41:00+09:00"
status: accepted
---

## 背景

`update_config` は完全な `AppConfig` を受け取り、
Web 向け DTO（`ConfigView` / `ConfigPatch`）で除外されている
`extra_headers` や秘密値、ローカルパスを JS から注入できた（R6 PR-M03）。
また `are_headers_placeholder()` が `any` 判定だったため、
1 つのマスク値でヘッダ全体の変更が捨てられた（R6 PR-M05）。

## 決定

- `normalize_to_dto_boundary(raw, current)` を追加し、
  `ConfigPatch::from(raw).apply_to(current)` で既存 DTO 境界を再利用する。
  - `update_config` は merge 前にこれを適用する。
  - 13 項目（extra_headers / api_key / client_secret / refresh_token /
    data_dir / cache_dir / config_file / rule_config / recent_paths /
    ollama.options / model_overrides / max_retries / local_roots[].path）
    がプレースホルダ／空値へ正規化され、`merge_secrets()` が現在値を復元する。
- `normalize_ai_from_js()` を追加し、`update_ai_config` の
  `AiConfig.openai.extra_headers` を空にしてから merge に渡す。
- `merge_secrets()` の `extra_headers` をキー単位マージへ変更:
  - 空 map: 全件 current 復元（既存 ConfigPatch 往復契約を維持）。
  - 非空: `***` キーのみ current 復元、ほかは incoming を採用。
  - current に無い `***` キーは sentinel を実値保存しないため破棄（fail-closed）。
- `update_config_patch()` は ConfigPatch 経路を維持（正規化は冪等で不要）。
- `clear_secrets()` / `commit_validated_config()` は `merge_secrets()` を
  通さない例外（ADR-0177）を維持。
- `validate_ai_base_urls()` は全経路で維持。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| DTO 境界で正規化（採用） | 手書きブラックリストより漏れが少ない |
| 生 AppConfig をそのまま受け入れる | 機微フィールド注入を許す |
| update_config の署名を ConfigPatch に変更 | IPC 破壊変更になる |

## 影響

- `update_config` からは秘密値・ヘッダ・パスを IPC で設定できなくなり、
  `config.toml` が権威になる。
- `update_ai_config` は ADR-0098 の契約に基づき、
  **明示的な `api_key` の新値のみ受け入れる**（受容済みリスク）。
  送信先は `validate_ai_base_urls()` で制限される。
- `merge_secrets()` のキー単位マージは defence-in-depth であり、
  本番の主境界は normalization 側。
- ヘッダキー照合は厳密一致（HTTP の大文字小文字規則とは不一致）。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- unit test 3 件追加:
  - JS が注入した extra_headers / api_key / data_dir / recent_paths が
    保存されない。
  - マスク値キーは現在値を維持し、他キーの変更は反映される。
  - `update_ai_config` で注入ヘッダは破棄されつつ、
    明示的新 `api_key` は採用される。
- `cargo test --all-targets --locked`: core 220 + desktop **62** + web 2。
- `cargo clippy --all-targets --locked -- -D warnings`、
  `./scripts/verify.sh`: 成功。E2E **249 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `update_ai_config` の `api_key` 受け入れを完全に廃止するか
  （ADR-0098 の既存契約との調整が必要）。
- `AiConfigView` のような DTO 型経由の正規化へ移行し、
  `normalize_ai_from_js()` の手書きフィールド指定を無くすか。
- ヘッダキーの大文字小文字正規化。
- `merge_secrets()` のキー単位マージを本番経路で活かす设计（現状は defence-in-depth）。
