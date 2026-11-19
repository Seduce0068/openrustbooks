---
type: Decision
title: "ADR-0209: 設定保存の原子化"
description: "config.toml を tmp ファイル＋rename で置換し、保存途中失敗による部分破壊を防ぐ決定を記録する。"
tags:
  - ADR
  - core
  - config
generated:
  by: dsh
  at: "2026-11-19T10:58:00+09:00"
status: accepted
---

## 背景

`AppConfig::save()` は固定パスの `config.toml` を
`truncate(true)` + `write_all()` で上書きしていた（R6 TD-L09）。
保存途中の失敗で設定ファイルが部分的に破壊される可能性があり、
ADR-0177 / ADR-0207 の残リスクとして個別追跡されていた。
ADR-0188 で `LocalStorage::write()` の tmp 名は一意化済みだが、
設定保存には及んでいなかった。

## 決定

- `AppConfig::save()` は `save_to_dir(ProjectDirs::config_dir())` へ委譲する。
- tmp 名を `<filename>.<pid>.<counter>.tmp`
  （例: `config.toml.12345.0.tmp`）とし、
  `AtomicUsize::fetch_add` で一意化する（最大 128 回再試行）。
- `write_private_new()` は Unix で
  `create_new(true)` + `.mode(0o600)` + `set_permissions(0o600)` を使用し、
  umask に依存せず 0600 を確定させる。非 Unix は
  `create_new(true)` + `write_all()`（パーミッション指定なし）。
- `atomic_write()` は tmp への書き込み成功後に
  `fs::rename(tmp, config_file)` を実行する。
- write / rename 失敗時は tmp を best-effort で削除し、
  元 `config.toml` は rename 成功時のみ置換する。
- 新規コードはすべて `#[cfg(not(target_arch = "wasm32"))]` とし、
  Unix / 非 Unix をさらに分岐する（wasm32 ビルド維持）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| tmp + rename（採用） | 途中失敗で元ファイルを守れる |
| 固定 tmp 名 | 並行 save / stale tmp で内容混在のリスク |
| io.rs の atomic 実装を無改造流用 | tmp が umask 依存（0644）になり 0600 契約を落とすため却下 |

## 影響

- 既存 `config.toml` が 0644 でも、保存後は 0600 に締まる
  （既存の 0600 契約を維持・強化）。
- 保存先ディレクトリ・ProjectDirs の半径は変更しない。
- クラッシュ時に tmp が残る可能性がある（一意名のため再利用はしない）。
- Windows では既存宛先への rename が失敗し得る
  （ADR-0188 と同じ制約、実機未検証）。
- `fsync` は未実施（durability と性能のトレードオフは未決定）。
- **lost update（TD-L10）は本変更で解消しない**。

## 検証（Runtime Proof）

- 追加テスト 7 件:
  - tmp 名一意性、0600 作成、既存 tmp 非上書き。
  - `save_to_dir` の内容正确・tmp 残存なし。
  - 既存 0644 ファイルの新内容＋0600 置換。
  - rename 失敗時の tmp 削除。
  - 並行保存で内容混在なし・tmp 残存なし。
- `cargo test -p openrustbooks-core`: core **261 passed**（旧 255 + 7 の一部反映、
  実測 261）。
- `cargo clippy -p openrustbooks-core --all-targets -D warnings`、
  wasm32 core check、`./scripts/verify.sh`: すべて成功。
- E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 並行設定更新の lost update 検出（TD-L10、read-modify-write 競合）。
- `fsync` 導入の要否（クラッシュ時の永続性）。
- Windows の rename 上書き挙動の実機確認。
- 書き込み失敗の決定論的故障注入テスト（現状は rename 失敗のみカバー）。
- ADR-0188 との atomic helper 共有（同期/非同期の違いで本次は分離）。
