---
type: Decision
title: "ADR-0024: 章の移動履歴の永続化"
description: "移動履歴を localStorage に保存し、同じ本を開き直したときにも戻る・進むができるようにする決定を記録する。"
tags:
  - ADR
  - epub
  - web
  - wasm
generated:
  by: dsh
  at: "2026-11-15T13:28:00+09:00"
status: accepted
---

## 背景

ADR-0017 までで移動履歴はメモリ上にのみ保持され、本を開き直すと履歴が
消えていた（ADR-0017 の未決定「履歴の永続化（再起動後も戻れるようにするか）」）。
読書位置はすでに localStorage に保存しているため、履歴も同じキー体系で
保存する。

## 決定

### 保存

- 履歴のキーは `openrustbooks:history:<name>:<size>:<lastModified>`。
- 値は JSON `{ "entries": [{"chapterIndex": N, "fragment": "..." | null,
  "scrollTop": N}], "index": N }`。
- 章を移動するたびに保存する（履歴が変わるたび）。
- 履歴が空・1件のときは保存しない（初期状態）。
- localStorage が使えない場合は黙って諦める。

### 復元

- 本を開いたとき、保存された履歴があり、すべての `chapterIndex` が章数の
  範囲内なら復元する。
- 範囲外のエントリが含まれる場合は履歴を捨て、初期状態にする。
- 保存が無い場合は従来どおり初期状態。
- 復元後は Back / Forward ボタンの有効状態も復元する。

### 寿命・掃除

- 履歴は読書位置の掃除（90 日・40 件）の対象にしない。
- 本を開き直すたびに上書きする。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| localStorage に履歴を保存（採用） | 追加依存なし、読書位置と同じキー体系で一貫する |
| セッションストレージ | タブを閉じると消える。永続化の意味が薄い |
| 履歴を持たない | 目次から探し直す必要があり、読書の連続性が損なわれる |

## 影響

- 同じ本を開き直しても、前回の移動履歴で戻る・進むができる。
- 履歴はタブを閉じても消えない（localStorage に保存される）。
- 保存値は `{entries, index}` になる。
- 古い履歴は上書きされるため、件数が無制限に増えることは無い。

## 検証

- ヘッドレス Chrome 157 と `tools/verify-web-reader.py` の E2E 97 項目が
  3 回連続で成功。履歴永続化について、章を移動すると履歴が localStorage に
  保存されること、リロード後に Back が有効になること、復元した履歴で Back
  が動くことを確認した。
- `cargo test --all-targets --locked` は 119 件成功。
- 6 ゲート（fmt / check / test / clippy / wasm32 × 2）と `trunk build` が成功。
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## 未決定

- 履歴の永続化をオプトアウトできるようにするか。
- 履歴一覧の UI。
- EPUB 内 CSS、縦書き・ページ送り（ADR-0012 から継続）。
EOF
