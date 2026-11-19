---
type: Decision
title: "ADR-0003: 共通コアをターゲット別依存でNative/Wasm両対応にする"
description: "ネイティブ専用依存をtarget別に分離し、共通コアとWebアプリをwasm32-unknown-unknownでビルド可能にする決定。"
tags:
  - ADR
  - wasm
  - build
  - core
generated:
  by: opencode
  at: "2026-10-16T01:12:27+09:00"
status: accepted
---

## 実装状況（2026-11-19）

以下の既存セクションは 2026-10-16 時点の決定履歴として保持する。rumdl feature に関する
現状は本節および [ADR-0242](0242-rumdl-feature-status.md) を正とする。

- 背景節の `rumdl (default)`（旧 25 行目）が指摘する問題は当時の背景である。
  現在は `core/Cargo.toml:66`（native）と `:72`（wasm）の 2 行がいずれも
  `rumdl = { version = "0.3", default-features = false }` である（ADR-0155 / 0168）。
- 決定節の target 別 feature 選択（旧 40-42 行目）は廃止された。native / wasm とも
  featureless であり、`wasm` feature は rumdl が公開する同名 `init` export との衝突で
  禁止されている（ADR-0160）。
- 影響節の「フィーチャー差異により非並列」（旧 76 行目）は根拠を失ったが、
  featureless（parallel feature なし）のため非並列という結果は維持される。
- 現状の担保は `scripts/check-rumdl-featureless.sh` による featureless 2 行の完全一致検査
  （ADR-0195）。workspace 版 rumdl は削除済み（ADR-0168）。
- 後続 ADR: [ADR-0155](0155-rumdl-feature-reduction.md) /
  [ADR-0160](0160-real-wasm-lint.md) /
  [ADR-0168](0168-workspace-rumdl-cleanup.md) /
  [ADR-0195](0195-rumdl-feature-gate.md)。

## 背景

ADR-0001/0002 は「デスクトップ (Tauri) とブラウザ (Wasm) が同一の共通コアを使う」ことを前提としている。しかし共通コアを `wasm32-unknown-unknown` でビルドすると失敗した。

主な原因は以下のネイティブ専用依存である。

| 依存 | 問題 |
|------|------|
| `tokio` (`full`) | `mio` が wasm32 未対応 (`compile_error!`) |
| `rumdl` (default) | `native` フィーチャーが `tower-lsp` / `tokio` / `notify` を引き込む |
| `reqwest` | ネイティブTLS前提。wasm では別フィーチャーが必要 |
| `async-openai` | `reqwest` 経由で同上 |
| `figment` / `directories` | OSの設定ディレクトリ・環境変数前提 |

結果として、ブラウザ版の中核仮説（コアをそのままWasm化する）が未検証のままだった。

## 決定

共通コアの依存とコードを **ターゲット別に分割** し、移植可能部分とネイティブ専用部分を明確にする。

### 依存関係

- `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` にネイティブ専用依存を移動する
  (`tokio`, `tracing-subscriber`, `tiktoken-rs`, `async-openai`, `figment`, `directories`, `reqwest`)
- `rumdl` はフィーチャーをターゲット別に選択する
  - ネイティブ: `default` (`native` = LSP・watcher・rayon を含む)
  - Wasm: `default-features = false, features = ["wasm"]`
- `url/serde`, `chrono/wasmbind`, `uuid/js` を明示的に有効化する
  (以前は `reqwest` 経由の推移的フィーチャーに依存していた)
- `wasm-bindgen-futures` を追加する (`#[wasm_bindgen]` の async 関数が要求する)

### コード

| モジュール/項目 | 扱い |
|-----------------|------|
| `document`, `error`, `ai_markdown::{diff, lint, prompt}`, `config` の型定義 | 両対応 (移植可能) |
| `ai_markdown::{client, pipeline}`, `AiMarkdownPipeline`, `AiMarkdownService` | ネイティブ専用 |
| `io::local` (tokio::fs, xdg-open) | ネイティブ専用 |
| `AppConfig::load/save` | ネイティブ専用 (ブラウザは `localStorage` を予定) |
| `wasm` モジュール | Wasm専用 |
| `CoreError` の `figment` / `async-openai` / `reqwest` / `tokio` 変種 | ネイティブ専用 |

- `PromptRegistry` は移植可能な `with_builtins()` と、カスタムプロンプト走査を含むネイティブ専用 `load()` に分離する。
- `JsValue` は `serde` の `Serialize` / `Deserialize` を実装しないため、serde 派生型のフィールドには `serde_json::Value` を用いる。

## 選択肢と根拠

| 観点 | ターゲット別依存 + `cfg` | 単一コアをネイティブのみ | コアを2クレートに分割 |
|------|--------------------------|--------------------------|------------------------|
| Wasm対応 | ✅ 追加依存なしで可能 | ❌ ブラウザ版が成立しない | ✅ 境界は明確 |
| 変更コスト | ✅ 既存構成を維持 | ✅ ゼロ | ❌ 大 (参照・公開API再編) |
| 型の共有 | ✅ 同一型を両ターゲットで使用 | — | ⚠️ 共有型の置き場が必要 |
| 認知負荷 | ⚠️ `cfg` が増える | ✅ 単純 | ⚠️ クレート間の依存規則が必要 |

**結論:** まずターゲット別依存と `cfg` で対応する。`cfg` の増加が保守負担になった時点で2クレート分割を再検討する。

## 影響

- ブラウザ版では AI クライアントとパイプラインがまだ利用できない。`fetch` ベースの `AiClient` 実装が必要。
- `CoreError` の変種がターゲットで異なる。マッチ側は `cfg` を意識する必要がある。
- `rumdl` のフィーチャー差異により、ブラウザ版の lint はワーカー/スレッド並列化を行わない。
- ビルド検証はネイティブ (`cargo check --workspace`) と Wasm (`cargo check -p openrustbooks-core --target wasm32-unknown-unknown`) の両方で行う。
- フロントエンドは Trunk (`web/Trunk.toml`) でビルドし、`web/dist` を Tauri の `frontendDist` として共有する。

## 検証

```bash
cargo check --workspace
cargo check -p openrustbooks-core --target wasm32-unknown-unknown
cargo build -p openrustbooks-web --target wasm32-unknown-unknown
cd web && trunk build
```

2026-10-16 時点で、上記に加えて `trunk build` が `web/dist` を生成し、
`wasm-bindgen` による JS グルーと `.wasm` が出力されることを確認済み。

## 未決定

- ブラウザ版の AI 呼び出し方式 (ブラウザから直接APIを叩くか、プロキシ経由か) と API キーの扱い。
- ブラウザ版の設定永続化 (`localStorage` / IndexedDB) の設計。
