# Goal

設定画面から危険な plugin / command を呼ばないことを静的ゲートで固定する（R6 A05 前提 2）。

## Scope

### 背景

ADR-0253 / 0274 は capability 最小権限を A05 前提 2 に列挙したが、
完了証跡が無い。Tauri 2.12.5 では `invoke_handler` 登録済みの独自 command は ACL 非対象で、
plugin fs / dialog / shell の scope が広い点が残リスク。
現状の設定画面は `get_config` / `update_config_patch` のみを呼ぶ。

### 成果物

- `scripts/check-capability-scope.sh`: 追加
- `scripts/verify.sh` / `.github/workflows/ci.yml`: 変更（ゲート配線）
- `docs/decisions/0278-settings-ipc-capability-scope.md`: 追加
- `docs/decisions/0253` / `0274`: 変更（前提 2 の完了証跡を記録）
- `todo.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 設定画面の IPC 呼び出しが `get_config` / `update_config_patch` のみであることを検査する。
2. `plugin:` 参照、fs / dialog / shell / updater の invoke、`UpdaterExt::check` が web 側に無いことを検査する。
3. 独自 command は ACL 非対象である ADR-0264 の判断を維持し、capability ファイルは追加しない。
4. 負向 test（危険 command 追加 / plugin 参照追加）で失敗する。
5. 全ゲート成功（E2E 256 維持）。

## Acceptance criteria

- 負向 test が失敗し、verify.sh 全ゲート成功。

## Constraints

- tauri.conf.json / web / desktop コード / E2E check 名・件数は変更しない。
- 共通 workflow、外部 secrets、実 Tauri WebView 検証は扱わない。
- `master` へ push する。force push はしない。
