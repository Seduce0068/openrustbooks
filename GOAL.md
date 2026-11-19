# Goal

desktop / mobile の build_app で重複する Tauri プラグイン設定を共通化する（R6 PR-L03、ADR-0201）。

## Scope

### 成果物

- `desktop/src/main.rs`: with_common_plugins + platform_plugins フック
- `docs/decisions/0201-build-app-plugin-dedup.md`: 追加
- `todo.md`: PR-L03 を完了へ移動

### 決定

- 8 共通プラグインを 1 関数へ集約。
- global_shortcut は store と notification の境界で挿入し、登録順を維持。
- Tauri は Vec で登録順に初期化するという事実をコメント化。

## Acceptance criteria

- E2E 251 項目、全ゲート成功（検証済み）。
