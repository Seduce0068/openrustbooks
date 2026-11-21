# Goal

EPUB / ZIP の解釈差を core / web で統一する（R6 レビュー / R3-IV01）。

## Scope

### 背景

core と web で次の解釈差がある:
BOM 入り mimetype、percent-encoded fragment、重複 manifest id、
%XX 復号失敗の Validation 飲み込み、外部スキーム判定。
fail-closed 契約と EPUB の拡張機能動作を統一する。

### 成果物

- `core/src/archive.rs`: 必要なら変更
- `core/src/epub/*`: 変更
- `web/src/lib.rs`: 必要なら変更
- `docs/decisions/0305-epub-zip-interpretation.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. mimetype の BOM / 前後空白を両実装で同じく扱う。
2. fragment の percent decode 失敗を Validation として扱い、黙って通さない。
3. manifest id の重複を拒否する。
4. 外部スキーム / パス脱出を両実装で fail-closed にする。
5. 既存 EPUB / ZIP test と E2E 256 件を維持。
6. verify.sh 全ゲート成功。

## Constraints

- 新規 ZIP / EPUB 機能は追加しない。
- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- mimetype BOM / fragment 復号 / manifest id 重複 / %XX 失敗 / 外部スキームを core+web 統一。
- ADR-0007 の fragment 生保持は ADR-0305 が supersede。
- core 341 / web 9 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
- 外部リンク allowlist / 全 URI 属性厳格検証は ADR-0305 の未決定。
