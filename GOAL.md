# Goal

config.toml 読み込み失敗時に既定値で上書きする経路を fail-closed 化する（R6 レビュー / R2-ER01）。

## Scope

### 背景

desktop/src/main.rs は `AppConfig::load().unwrap_or_default()` でエラーを捨てる。
その後の commit_config が `config.save()` を呼ぶため、不正 TOML や権限エラーで
読み込めなかった既存 config.toml が既定値で上書きされ、シークレットと local_roots が失われる。

### 成果物

- `core/src/config.rs`: 必要なら読み込みエラー詳細 API / 退避ヘルパ追加
- `desktop/src/main.rs`: 変更（エラー報告・退避・保存拒否）
- `desktop/src/state/mod.rs`: 変更（読み込み失敗状態の保持）
- `desktop/src/commands/config.rs`: 変更（失敗時 save 拒否）
- `docs/decisions/0284-config-load-fail-closed.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. `AppConfig::load()` の Err を握り潰さず、ログまたは起動時 status で報告する。
2. 読み込み失敗時は元 config.toml を退避し、既定値で上書きしない。
3. 読み込み失敗状態では config 保存 command を fail-closed で拒否する。
4. 既存の正常起動・保存 E2E を壊さない。
5. markdownlint 0 issues、verify.sh 全ゲート成功。

## Acceptance criteria

- 不正 config.toml を置いた状態で起動しても元ファイルが退避され、保存が拒否される。
- 正常系 config.toml は従来どおり保存できる。

## Constraints

- E2E 256 件・check 名・UPDATER 契約は変更しない。
- 秘密値をログへ出力しない。
- `master` へ push する。force push はしない。

## 完了記録

- config.toml 読み込み失敗時に退避 + 保存拒否を実装。
- 退避先を Unix 0600 に制限し、ログから絶対パスと生エラーを除去。
- R4 high 2 件を反映。core 293 / desktop 92 tests、verify.sh 全ゲート成功（E2E 256、WDIO 1）。
