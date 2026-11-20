# Goal

設定 DTO の `read_only` / `watch` を現在値から復元し、read-only root の書き込み昇格を防ぐ（R6 レビュー / R3-AU01）。

## Scope

### 背景

`LocalRootView` は `read_only` / `watch` / 配列順をクライアント writable で公開する。
`merge_secrets()` は id 一致時に `path` だけを現在値から復元するため、
IPC を呼べるスクリプトが `read_only=false` を送ると、隠された実パスを持つ root が
writable へ永続的に昇格する。incoming 側の id 重複も検査されていない。

### 成果物

- `desktop/src/commands/config.rs`: 変更（fail-closed 復元と単体 test）
- `docs/decisions/0281-config-root-flags-fail-closed.md`: 追加
- `todo.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新
- `GOAL.md`: 更新

### 必須事項

1. `merge_secrets()` で id 一意一致時に `path` に加えて `read_only` / `watch` も現在値を採用する。
2. incoming 側の `local_roots[].id` が空／重複の場合は fail-closed で path を空にし、flags も現在値を採用しない。
3. 単体 test で `read_only=true` の現在 root に incoming の `false` を渡しても `true` が保持されること。
4. E2E 256 件・check 名・Cargo.* は変更しない。
5. markdownlint 0 issues、verify.sh 全ゲート成功。

## Acceptance criteria

- 負向の flags 変更が無視され、全ゲート成功。

## Constraints

- DTO 構造・web 画面・E2E スクリプト・Cargo.* は変更しない（flags は DTO 往復のため残す）。
- 実 Tauri WebView 検証は別タスクとして残す。
- `master` へ push する。force push はしない。
