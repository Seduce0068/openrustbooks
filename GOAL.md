# Goal

updater 死設定の正直表示と保存時 fail-closed を復元する（R6 A05 / ADR-0222 の再確認）。

## Scope

### 背景

ADR-0222 は `check_updates` の値に関わらず「更新確認: 未配線」を表示すると決定した。
しかし設定保存 UI の editable 化（ADR-0272）で `check_updates` が通常の checkbox になり、
DTО 往復で config.toml へ保存されるようになった。
updater は plugin 初期化のみで `UpdaterExt::check` / pubkey / endpoint は無く、
値は動作に接続されないため、利用者へ誤解を与える。

### 成果物

- `web/src/lib.rs`: 変更（`check_updates` を固定表示へ復元、editable トグル廃止）
- `desktop/src/commands/config.rs`: 変更（`merge_secrets` で現在値を保持し JS 変更を無視）
- `scripts/check-updater-honesty.sh`: 追加（静的ゲート）
- `scripts/verify.sh` / `.github/workflows/ci.yml`: 変更（ゲート配線）
- `tools/verify-web-reader.py`: 変更（既存 check の条件拡張、check 名・件数は 256 維持）
- `docs/decisions/0277-updater-honesty-restoration.md`: 追加
- `docs/decisions/0222-updater-deferred.md`: 変更（再確認節）
- `todo.md` / `README.md` / `docs/index.md` / `docs/decisions/index.md` / `docs/log.md`: 更新

### 必須事項

1. 設定画面に `check_updates` の editable checkbox を置かず、
   「更新確認: 未配線」を固定表示する。
2. desktop 側で `merge_secrets()` が `check_updates` の現在値を復元し、
   JS からの変更が config.toml へ保存されないようにする。
3. 静的ゲートで `UpdaterExt::check` / pubkey / endpoint が無く、
   固定表示と現在値復元が残ることを検査する。
4. E2E check 名・期待総数 256（check 239 + equal 17）を維持する。
5. 全ゲート成功。

## Acceptance criteria

- 負向 test が失敗し、verify.sh 全ゲート成功（E2E 256/0）。

## Constraints

- 公開鍵 / endpoint / 署名 secrets / 自動更新は導入しない。
- E2E check 名・件数、共通 workflow、Cargo.* は変更しない。
- `master` へ push する。force push はしない。
