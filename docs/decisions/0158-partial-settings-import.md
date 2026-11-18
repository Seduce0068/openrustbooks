---
type: Decision
title: "ADR-0158: 部分設定インポートの調査結果"
description: "部分設定インポートの bool false リセットが現行ガード下では再現しないことを記録する。"
tags:
  - ADR
  - web
  - data
generated:
  by: dsh
  at: "2026-11-18T20:55:00+09:00"
status: accepted
---

## 背景

ADR-0157 の未決定として、`import_positions()` が
`dark` / `followSystem` / `downscaleImages` を `unwrap_or(false)` で読むため、
バックアップにキーが無い場合に現在設定の `true` が `false` へ
リセットされる懸念が挙げられていた。

## 調査結果（Runtime Proof: コード追跡）

- `web/src/lib.rs:1707-1709`: 設定インポートは
  `storage.get_item(SETTINGS_KEY).ok().flatten().is_none()` のときだけ実行される
  （ADR-0157 の現在値保護ガード）。
- `web/src/lib.rs:1724,1728,1742`: 確かに 3 bool は `unwrap_or(false)` を使う。
- `web/src/lib.rs:1421-1439`: `load_settings()` も同じ `SETTINGS_KEY` を読み、
  キーが無ければ `ReaderSettings { follow_system: system_prefers_dark(), ..Default::default() }`
  を返す。
- したがって:
  - 現在設定キーが存在する場合はガードでスキップされ、
    `unwrap_or(false)` は評価されない。
  - ガードを通る（キーが無い）場合は `load_settings()` も既定値を返すため、
    「現在の `true` をフォールバックにしたい」という状態が自然には成立しない。
- 「退避 → 削除 → 部分バックアップ import → 退避分を復元」という E2E 手順では、
  復元は `save_settings(&merged)` の後になり、フォールバック計算に反映されない。
  現在キーを残せばガードでスキップされ、修正の有無にかかわらず通る偽陽性になる。
- フォールバック経路を人為的に通すには `Storage.prototype.getItem` を
  「ガード呼び出しだけ null、load_settings() には保存済み true」と差し替える必要があり、
  実運用で起きない脆いテストになる。

## 決定

- コード変更・E2E 追加は **行わない**（no-op と確定）。
- 期待総数は **234**（check 217 + equal 17）のまま維持。
- 現行の `unwrap_or(false)` は「キー欠落＝新規／クリア後の環境で
  `load_settings()` の既定値（OS ダーク追従を含む）を永続化してよいか」
  という**設計論点**であり、現在値の破壊リスクではない。
- ADR-0157 の「現在値維持」ガードは維持する。ガードを外して
  per-key マージに変更すると ADR-0157 と正面から衝突するため却下。

## 影響

- 既存挙動は不変。全ゲート・E2E 234 項目は前回の緑を維持。
- 新規依存なし。

## 検証

- エージェント 2 体（実装担当・監査担当）が独立にコードを追跡し、
  いずれも no-op と結論。
- `web/src/lib.rs` / `tools/verify-web-reader.py` の差分は HEAD と同一。
- コード変更がないため compile / test / E2E は再実行せず、
  前回の `./scripts/verify.sh` rc=0（E2E 234/0）を適用。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- 新規／クリア後の環境で部分バックアップを取り込む際、
  `load_settings()` の既定値を永続化する現在の挙動を維持するか
  （ADR-0038 の初回テーマ方針との整合を含めて要判断）。
- 明示的な設定復元オプションの UI 追加（ADR-0157 から継続）。
- 改ざん検知、設定の保存時刻メタデータ、履歴のエントリ単位マージ、
  複数タブの localStorage 競合（ADR-0157 から継続）。
