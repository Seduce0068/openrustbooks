---
type: Decision
title: "ADR-0213: prefers-reduced-motion の尊重"
description: "OS の reduce 設定で空間移動を抑制し、色 transition と位置復元の正確性を維持する決定を記録する。"
tags:
  - ADR
  - web
  - accessibility
generated:
  by: dsh
  at: "2026-11-19T12:16:00+09:00"
status: accepted
---

## 背景

`web/index.html` は `.btn` に無条件の `transition: all 0.2s` を適用し、
Reader の位置復元はスクロール再試行ループを使っていた（R6 TD-M13）。
modern-web-guidance (`motion`) は、色や背景色の非空間 transition は維持しつつ、
空間移動／scale／rotation は
`@media (prefers-reduced-motion: no-preference)` で opt-in することを推奨する。

## 決定

- `.btn` の transition を `color` / `background-color` / `border-color` のみへ変更する
  （reduce 時も維持）。
- hover 時の `transform: translateY(-1px)` と `box-shadow`、およびその transition は
  `@media (prefers-reduced-motion: no-preference)` 内にのみ置く。
- `transition: none` や `animation-duration: 0.01ms` は使用しない。
- `prefers_reduced_motion()` を
  `#[cfg(target_arch = "wasm32")]` で追加し、
  `matchMedia("(prefers-reduced-motion: reduce)")` を
  `Ok(Some(q)) => q.matches()` / `Ok(None) | Err(_) => false` で処理する。
  起動時にキャッシュせず、位置適用時に毎回読む。
- `apply_position_frame()` は画像セトリング待ちを無条件に維持し、
  確定後に reduce を判定してスクロール再試行ループだけをスキップする。
  位置保存、巨大画像警告、`last_apply_time`、`applying = false` の後始末は
  従来どおり必ず通過させる。
- E2E を 1 件追加し、期待総数 252 → **253**（check 236 + equal 17）。

## 選択肢と根拠

| 選択肢 | 判断 |
| --- | --- |
| 色維持＋空間 opt-in（採用） | ガイド準拠で視覚フィードバックも残る |
| 全 transition を none | 色フィードバックまで失われるため却下 |
| 0.01ms に短縮 | 一部のアニメーションが逆に刺激性を持つため却下 |

## 影響

- reduce 時は hover のリフトが消えるが、色変化は残る。
- 位置復元時に画像セトリング待ちは残るため、
  レイアウト確定前の誤位置保存は防がれる（ADR-0136 / 0149 を維持）。
- OS 設定であってアプリ設定ではないため、設定フォームには追加しない。
- 新規依存なし、unsafe なし。

## 検証（Runtime Proof）

- E2E「Reduce で色 transition を保ち空間移動なしでフラグメントへ即時移動する」が ok。
  - `Emulation.setEmulatedMedia` で reduce を設定し、finally で初期化。
  - `matchMedia` が true。
  - computed `transitionProperty` に `color` を含み `all` / `transform` を含まない。
  - `animationName` / `transform` が none。
  - フラグメント移動が 3 秒以内に scrollTop > 0 で反映。
- `cargo fmt --all -- --check`、wasm32 core / web check、
  `python3 -m py_compile`、`./scripts/verify.sh`: すべて成功。
- E2E **253 項目すべて成功**。
- CI は billing issue で実行不可（ローカル検証で代替）。

## 未決定

- EPUB 内 CSS のアニメーション（ADR-0012 / 0115 を継続）。
- 将来の View Transition / scroll-driven animation / smooth scroll を追加する場合の reduce 方針。
- OS 設定の実行中変更を購読して即時反映するか（現状は位置適用時に毎回読む）。
- dialog アニメーションの reduce 対応要否。
