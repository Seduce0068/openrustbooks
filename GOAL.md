# Goal

初版スコープを ADR として確定する（R6 D01 / R5）。

## Scope

### 背景

`docs/requirements.md` は利用者の要望を網羅するが、
「要件を書いても初版で実装するとは限らない」と明記しており、
対象 OS / ブラウザ、対応形式、保存先、機能範囲が未決定だった。
実装と ADR は 231 件に達し、初版で提供される境界を
証拠付きで整理できる状態になった。

### 成果物

- `docs/decisions/0232-initial-release-scope.md`: 追加
- `docs/requirements.md`: 変更（初版スコープ ADR への参照を追加）
- `todo.md`: D01 を完了へ移動
- `GOAL.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 初版で提供する対象（native desktop / browser）、対応形式、
   Editor / Reader / AI / 保存 / 配布の範囲を実装証拠付きで記載する。
2. 初版に含まない項目（PDF 完全対応、Google Drive、OS ファイルマネージャー、
   自動更新、モバイル、実機 5 ブラウザ等）を明確にする。
3. 各決定を既存 ADR / E2E / コードへ対応付ける。
4. `requirements.md` の未決定文言を保持しつつ、初版スコープへの導線を追加する。
5. markdownlint 0 issues、verify.sh（E2E 254）成功。

## Acceptance criteria

- markdownlint 0 issues、verify.sh 成功。

## Constraints

- ユーザー要件を削除しない（初版スコープは実装判断として記録）。
- 外部サービス・配布・課金の決定は行わない（未決定として残す）。
- `master` へ push する。force push はしない。
