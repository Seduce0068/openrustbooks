---
type: Decision
title: "ADR-0145: 縦書きとページ送りの段階導入方針"
description: "縦書き表示とページ送りを即座に実装せず、E2E の軸非依存化から始める段階導入方針を記録する。"
tags:
  - ADR
  - web
  - epub
  - ui
generated:
  by: dsh
  at: "2026-11-18T15:50:00+09:00"
status: accepted
---

## 背景

Web リーダーは横書き・連続スクロール（`overflow-y: auto`）で、
位置保存は `scrollTop` 一択（`SavedPosition { chapterIndex, fragment, scrollTop, savedAt }`）。
`writing-mode` / `column-count` / `scroll-snap` の指定は無く、
core の `ReaderConfig` には `vertical_writing` / `scroll_mode` 等が
**宣言のみ**で配線されていない（デッド設定）。
ADR-0012 以来、縦書き・ページ送りは未決定として引き継がれてきた。

## 現状（2 観点の調査）

- スクロール: `.reader-content`（`web/index.html:347-352`）が `overflow-y: auto`。
  保存は `on_scroll` 200ms デバウンス（`web/src/lib.rs:2697-2749`）、
  復元は `apply_position_frame`（:693-789）で `scrollTop` を直接操作。
- フラグメント: `getBoundingClientRect().top` 差（:249-270）。
- 画像: `images_settled()` は `complete() && naturalWidth > 0`（:417-446）、
  縮小は `content.client_width()` 基準（:573,589）。
- E2E: `scrollTop` キー 4 つ固定（`tools/verify-web-reader.py:837-842`）、
  2px / 50px / 30px 閾値（ADR-0140）。
- キーボード: 既存ハンドラに IME / dialog / 入力欄ガード（:2959-2988）。
  ページ送りキーは未実装。

## 決定

- **今回はコード変更なし**（調査と決定のみ）。
- **縦書きはオプトイン**、EPUB 内 CSS は ADR-0115 のまま非適用を維持する。
- **ページ送りは段階導入**:
  - ① CSS `column-count` + `scroll-snap`: 後段で検討（位置復元と 229 項目への影響が大きい）。
  - ② 1 章を DOM でページ分割: 今回は見送り（大規模改修、新規依存リスク）。
  - ③ 連続スクロールのまま「ページ」概念だけ追加（既存 page-list の延長）: 最初に検討する価値あり。
- **最初のステップ**（次の実装タスク）:
  1. E2E のスクロール読み書きを「`writing-mode` の computed value を見て
     `scrollTop`/`scrollLeft` と `rect.top`/`left` を切り替えるヘルパー」へ
     リファクタする。**挙動は一切変えず**、期待総数 229 とキー 4 つ固定を維持し、
     横書きで 10 回連続 229/0 のベースラインを取り直す。
  2. 次に Web `ReaderSettings` へ `writingMode` を `serde(default)` で追加し、
     既定は現状の横書きを維持したままオプトインで縦書きを試す。
  3. `scrollLeft` は縦書き時のみシリアライズするなど、既定のキー 4 つを維持する
     工夫が必要（**推測・要検証**）。
- ページ番号は復元の主キーにしない（設定変更・画像ロードで不安定なため）。
  主キーは従来の章＋フラグメント＋オフセットを維持する。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| E2E 軸非依存化→オプトイン縦書き→ページ概念（採用） | 229/0 を守りつつ段階的に進める |
| 一括実装（縦書き＋改ページ） | 位置復元・履歴・E2E が一斉に壊れる |
| 完全見送り | 調査済みの判断材料を活かせない |

## 影響

- コード変更なし。既存 246 テスト＋wasm 3、E2E 229 項目は維持。
- core のデッド設定（`vertical_writing` 等）は今回も未配線のまま。

## 検証

- 2 観点のエージェント調査（実装経路と UX / アクセシビリティ）を実施。
- 縦書きの実ブラウザ挙動は一切未検証（読み取り専用のため）。
  影響リストはコードフローからの**推測**。
- 5 ブラウザ（Chrome / Edge / Brave / Firefox / Safari）での確認は未実施
  （現状は Chrome 157 のみで実証）。

## 未決定

- `writing-mode: vertical-rl` の実スクロール軸（`scrollLeft` か負の `scrollTop` か）の実機確認。
- 位置保存スキーマの後方互換とバージョン移行（ADR-0016 / 0036 / 0076 と整合）。
- ページ送りの併用／排他、キー割当て（`[` / `]` 章移動との関係）、
  ショートカット一覧と `aria-keyshortcuts` の更新。
- `text-orientation: mixed` 固定か選択制か、`font-feature-settings`（`vert`/`vrt2`）、
  `text-combine-upright` の 5 ブラウザ対応。
- 画像ルールの論理プロパティ化（`max-width` → `max-inline-size`）と
  縮小判定の軸入れ替え。
- タッチ判定（30px / 50px）の意味再定義。
- E2E 229 項目の軸非依存化後の 2px / 50px / 30px 閾値の据え置きか再調整か。
