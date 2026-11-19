---
type: Decision
title: "ADR-0201: Tauri プラグイン設定の共通化"
description: "desktop / mobile の build_app で重複していた共通プラグイン設定を 1 関数へ集約する決定を記録する。"
tags:
  - ADR
  - desktop
  - tauri
generated:
  by: dsh
  at: "2026-11-19T09:12:00+09:00"
status: accepted
---

## 背景

`desktop/src/main.rs` の desktop / mobile `build_app()` は
log / opener / fs / dialog / shell / store / notification / updater の
8 プラグイン設定が同一で、global_shortcut の有無だけが異なっていた（R6 PR-L03）。
プラグイン追加時に片方だけ更新されるリスクがあった。

## 決定

- `with_common_plugins(builder, platform_plugins)` を追加し、
  8 つの共通プラグインを 1 箇所で適用する。
- `platform_plugins` は `FnOnce(Builder<Wry>) -> Builder<Wry>` の挿入フックとし、
  `store` と `notification` の境界で呼び出す。
  - desktop: `global_shortcut` を追加。
  - mobile: 何も追加しない。
- プラグインの種類・設定・登録順を維持する:
  - desktop: log → opener → fs → dialog → shell → store →
    **global_shortcut** → notification → updater
  - mobile: global_shortcut なし（他は同一）。
- Tauri 2.12.5 の `PluginStore` は `Vec<Box<dyn Plugin<R>>>` で
  `register()` が `push` し、`initialize_all()` が順に初期化するため、
  登録順は意図的に保持する（誤った「順序は無意味」コメントは削除）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 挿入フック付き共通関数（採用） | 重複を消しつつ順序も維持できる |
| global_shortcut を末尾へ移動 | 登録順が変わり契約違反 |
| 共通化しない | 更新漏れリスクが残る |

## 影響

- 8 プラグイン、特に log の 3 targets
  （Stdout / LogDir{file_name: "openrustbooks"} / Webview）と
  `LevelFilter::Info` は変更前と同一（差分確認済み）。
- desktop / mobile の `#[cfg(target_os)]` 分岐は維持。
- 新規依存なし、unsafe なし、E2E 251 維持。

## 検証（Runtime Proof）

- `cargo fmt --all -- --check`、
  `cargo check --all-targets --locked`、
  `cargo test --all-targets --locked`、
  `cargo clippy --all-targets --locked -- -D warnings`: すべて成功。
- `./scripts/verify.sh` rc=0、E2E **251 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- `#[cfg(target_os)]` と `#[cfg(mobile)]` の分岐を一本化するか
  （現在は併存、動作上の問題なし）。
- `desktop/Cargo.toml` のコメントアウト済み global_shortcut 行の削除
  （本次書き込み範囲外）。
