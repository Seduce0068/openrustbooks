# Goal

直近 ADR の未決定のうち、後続 ADR で解消済みの項目を更新する。

## Scope

### 背景

- ADR-0099: 同意ダイアログ → ADR-0104 で実装済み
- ADR-0100: base_url / extra_headers 制限 → ADR-0101 / 0102 で実装済み
- ADR-0101 / 0103 / 0104: 同意ダイアログ → ADR-0104 で実装済み
- ADR-0105: 旧 DTO 往路 → ADR-0106、service 再初期化 → ADR-0107 で実装済み
- ADR-0106: service 再初期化・エラー伝播 → ADR-0107 で実装済み
- ADR-0107: `.hbs` の system_prompt → ADR-0108 で実装済み
- ADR-0108: system 指示の二重化 → ADR-0109 で実装済み
- ADR-0110: 未収束情報 → ADR-0111 で実装済み
- ADR-0113: expected_images → ADR-0114 で実装済み
- ADR-0112: EPUB 内 CSS → ADR-0115 で判断済み

### 成果物

- 対象 ADR（0099, 0100, 0101, 0103, 0104, 0105, 0106, 0107, 0108, 0110, 0112, 0113）: 変更
- `GOAL.md`: 本契約

### 必須事項

1. 各 ADR の該当未決定を「## 解決済み」へ移すか、取り消し線＋後続 ADR 参照を付ける。
2. まだ未実装の項目は「未決定」に残す。
3. コードは変更しない。

## Acceptance criteria

- markdownlint が通る。

## Constraints

- コードを変更しない。
- ADR の事実関係は後続 ADR を読んで確認する。
- `master` へ push する。force push はしない。
