---
type: Decision
title: "ADR-0245: 容量超過時の位置削除通知"
description: "localStorage 容量超過で古い読書位置を削除した場合に件数と残件数を通知する決定を記録する。"
tags:
  - ADR
  - web
  - storage
generated:
  by: dsh
  at: "2026-11-19T21:35:00+09:00"
status: accepted
---

## 背景

`save_position()` は localStorage の容量超過時に古い位置を
`POSITION_MIN_KEEP = 10` / `1` まで削除して再試行していたが、
削除件数・残件数を通知していなかった（R6 R3-config#5）。
全失敗時も「保存に失敗しました」とのみ表示し、
不可逆な削除が起きた事実が隠れていた。
ADR-0019 は「容量超過時は古い位置を捨てて現在を優先する」と定めており、
本決定は縮退戦略を維持しつつ可視化を追加する。

## 決定

- 容量超過リトライで `remove_item()` が成功した件数と、
  再試行後の残件数・再保存成否を数値として記録する。
- `thread_local!` の `POSITION_PRUNE_NOTICE` に
  `(deleted, remaining, saved)` だけを置き、
  `SaveErrorTarget::Reader` の既存カスタムイベントで status へ届ける。
  `dispatch_event` は同期なので、リスナーが `take()` で読み取りクリアし、
  発行側も dispatch 後に必ずクリアする。
- status 文言は固定テンプレート＋数値のみ:
  - 成功: `空き容量を確保するため古い読書位置を {deleted} 件削除しました（残り {remaining} 件）。バックアップはエクスポートから保存できます。`
  - 失敗: `空き容量不足で読書位置を {deleted} 件削除しましたが保存できませんでした（残り {remaining} 件）。`
- `SAVE_ERROR_MESSAGE` はペイロードが無い場合の既存通知として維持する。
  Editor 側のイベント分離は不変。
- console にも操作名と数値のみを出力し、
  キー名・ファイル名・生エラー・DOMException 名は出力しない。
- 通常 prune（90 日 / 40 件）と定数は変更しない。
- 退避先（IndexedDB / 自動ダウンロード）は今回実装せず、
  既存 export / import を利用者主導の退避先として案内する。
- E2E を 1 件追加し、期待総数 254 → **255**（check 238 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 件数・残件数を通知（採用） | 不可逆な削除を利用者が把握できる |
| 黙って削除（現状） | データ消失が隠れる |
| 自動退避を実装 | IndexedDB は ADR-0019 で却下、同 storage は容量を空けず、自動 DL はブラウザ制限 |

## 影響

- 容量超過時の status が既存の保存失敗文言から専用文言へ変わる
  （ペイロード有無で切り替えるため、他の保存失敗は従来どおり）。
- 全失敗時も旧 `SAVE_ERROR_MESSAGE` ではなく専用の失敗文言になる。
- 通知は数値のみで、ADR-0154 / 0212 / 0226 の redaction 契約を維持する。
- 通常の read / prune / export / import 契約は不変。

## 検証（Runtime Proof）

- E2E「容量超過時の位置削除で削除件数と残件数を通知し現在の本の位置は残る」が ok。
  - `Storage.prototype.setItem` を一時 override し、
    位置キーへの最初の setItem だけを DOMException で失敗させる。
  - `try/finally` で override と注入キーを復元。
  - status の数値書式、機微トークン非含有、
    現在の本の位置が残ることを検証。
- AST: check 238 + equal 17 = **255**、check 名一意、
  `docs/e2e-traceability.md` の全 check 名と一致。
- `python3 -m py_compile`、`cargo fmt --all -- --check`、
  wasm-web check、`./scripts/verify.sh`: 成功。
- E2E **255 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- DOMException 名による quota 判定
  （現状は `set_item` の全失敗を容量超過として扱う既存挙動を継続）。
- 通知の寿命／自動消去と、専用 `aria-live` 領域
  （ADR-0021 / 0226 を継続）。
- 通常 prune（90 日 / 40 件）も通知するか。
- `remove_item` 失敗の格上げと、セッション内 undo バッファの要否。
- 削除後も保存失敗する分支の E2E と console の直接検証
  （現状は成功分支のみ、console は status と同じ文言を出力）。
