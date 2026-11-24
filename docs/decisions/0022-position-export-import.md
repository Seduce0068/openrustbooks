---
type: Decision
title: "ADR-0022: 読書位置のエクスポート・インポート"
description: "保存した読書位置を JSON で書き出し・読み込みする決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T12:58:00+09:00"
status: accepted
---

## 背景

ADR-0019 までで読書位置を localStorage に保存・掃除するようにしたが、
バックアップや別ブラウザへの移行の手段が無かった（ADR-0019 の未決定
「読書位置のエクスポート・インポート」）。ブラウザ内で完結する JSON の
書き出し・読み込みを追加する。

## 決定

### エクスポート

- `Export` ボタンで `openrustbooks:position:` で始まる全キーを
  `{ key: value }` の JSON オブジェクトにまとめ、
  `openrustbooks-positions.json` としてダウンロードする。
- ダウンロードは `<a download>` と `URL.createObjectURL` を使い、1 秒後に
  URL を解放する。
- 削除されたキー（掃除済み）は含まれない。

### インポート

- `Import` ボタン（非表示の `<input type="file" accept=".json">`）で JSON を
  読み込む。
- 形式が `{ key: value }` のオブジェクトで、キーが
  `openrustbooks:position:` で始まるものだけを取り込む。
- 値は文字列でもオブジェクトでも受け付ける（後方互換）。
- 値が `SavedPosition` として読めないエントリは飛ばす。
- 既存のキーは上書きする。
- 取り込んだ件数を状態メッセージに表示する。
- 形式が不正な場合は状態メッセージで報告し、何も取り込まない。
- `openrustbooks:settings` はインポートの対象にしない。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| JSON ファイルのダウンロード・アップロード（採用） | 追加依存なし、ブラウザ内で完結、手動バックアップに十分 |
| IndexedDB への移行 | 非同期化と移行が必要。単一の小さな値には過剰 |
| クラウド同期 | 認証・サーバが必要。範囲外 |
| ファイルシステム Access API | ブラウザ対応が限定的 |

## 影響

- 読書位置をファイルとして保存し、別ブラウザや別端末へ移せる。
- インポートは既存のキーを上書きするため、復元前にバックアップを取れる。
- 不正な JSON では何も取り込まない（fail-closed）。
- 追加依存は無く、既存の `web-sys` 機能の範囲で実装している。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 93 項目が
  3 回連続で成功。エクスポート・インポートについて、ボタンの存在、
  インポート後の状態メッセージ、読書位置の取り込み、不正 JSON の拒否、
  表示設定の保持を確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## 未決定

- インポート時のマージ（上書きではなく統合）。
- エクスポート内容に表示設定を含めるか。
- 読書位置のエクスポート形式のバージョニング。
- クラウド連携時の自動同期（ADR-0015 から継続）。
