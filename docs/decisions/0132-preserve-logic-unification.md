---
type: Decision
title: "ADR-0132: 隠しフィールド保持の一元化"
description: "ConfigPatch::apply_to を公開フィールドの上書きのみとし、隠しフィールドの保持を merge_secrets に一本化する決定を記録する。"
tags:
  - ADR
  - config
  - refactor
generated:
  by: dsh
  at: "2026-11-18T10:40:00+09:00"
status: accepted
---

## 背景

ADR-0131 で追加した `ConfigPatch::apply_to()` と既存 `merge_secrets()` は
ともに隠しフィールド（secret / パス等）を保持しており、
将来のフィールド追加時に片方だけ追従すると経路で挙動が割れるリスクがあった。

## 決定

- `apply_to()` は公開フィールドの上書きのみを行い、DTO 外の
  **13 項目**はプレースホルダ／既定値を入れる。
  - `data_dir` / `cache_dir` → `None`
  - `max_retries` → `0`
  - `model_overrides` / `extra_headers` / `ollama.options` / `rule_config` → 空
  - `api_key` → `None`、`client_secret` → `""`、`refresh_token` → `None`
  - `recent_paths` → 空、`local_roots[].path` → 空 `PathBuf`
  - `config_file` → `None`
- **保持は `merge_secrets()` が唯一の箇所**とする。
  不変条件「`apply_to()` の結果は必ず `merge_secrets()` を通す」を
  コードコメントに明記（本番呼び出しは `update_config_patch` の 1 箇所のみ、
  `reset_config` は意図的にバイパス）。
- `local_roots[].path` は、`id` が一意一致した場合のみ現在値を復元し、
  不一致／新規／空 id は `apply_to()` と `merge_secrets()` の両方で
  空 `PathBuf` に統一する（ADR-0129 の fail-closed と整合）。
  `update_config` 経路でも新規 id の path は空になる（挙動変更）。
- ADR-0131 の「隠しフィールド 11 種」は実際には 13 項目だったため、
  本 ADR で修正する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| merge_secrets に集約（採用） | 単一情報源となり追加漏れを防ぐ |
| apply_to にも保持させ続ける | 二重管理が残る |
| 型で強制 | リファクタ範囲が広がる（将来課題） |

## 影響

- 新規／不一致 id の `local_roots` は `path=""` で保存され、
  ADR-0129 の root 検証で拒否される（fail-closed で安全）。
- 既存テストを `apply_to()` 単体から
  `merge_secrets(&current, &patch.apply_to(&current))` 経路へ変更し、
  さらに「`apply_to()` がプレースホルダを出す」契約テストを追加した。
- 新規依存なし、unsafe なし。

## 検証

- `cargo test --all-targets --locked`: core 186 + desktop 44 + web 2、
  wasm-test 3、audit 脆弱性 0 件、E2E 229 項目すべて成功。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定（preserve-auditor の指摘）

- `apply_to()` の可視性を `pub` のままにするか絞るか。
  「必ず merge を通す」が型で担保されていない。
- `google_drive: None` が「削除」として扱われる挙動の仕様化。
- `has_*` 更新／削除用 dedicated command の要否（ADR-0131 から継続）。
- Web UI 接続と `withGlobalTauri` + CSP/nonce の同時実施（ADR-0128 から継続）。
