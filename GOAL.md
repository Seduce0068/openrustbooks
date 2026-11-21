# Goal

desktop ビルド前の frontend 生成を明示化する（R6 レビュー / R1-AR01）。

## Scope

### 背景

verify.sh は trunk build を一時 DIST へ出すため web/dist を更新しない。
一方 desktop/tauri.conf.json の frontendDist は ../web/dist を指し、
cargo build グラフにはその生成依存が無い。
このままだと古い／存在しない web/dist で desktop バイナリが作られ得る。

### 成果物

- `scripts/build-desktop.sh`: 追加（frontend 生成 → desktop build）
- `scripts/check-frontend-dist.sh`: 追加（frontendDist 整合の静的ゲート）
- `scripts/verify.sh`: 変更（ゲート配線）
- `docs/decisions/0289-desktop-frontend-build-dependency.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. desktop build の前に web/dist を `--offline --locked` で生成する。
2. tauri.conf.json の frontendDist と実際の生成先を静的に照合する。
3. index.html / wasm / JS グルーが無い場合は fail-closed で拒否する。
4. beforeBuildCommand の `trunk build --release` と競合しない。
5. verify.sh 全ゲート成功、E2E 256 件・WDIO ゲート維持。

## Acceptance criteria

- desktop ビルドが常に現行 frontend を参照し、欠落時に失敗する。

## Constraints

- 外部サービス / 公開ビルド / 課金は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- build-desktop.sh と check-frontend-dist.sh を追加し、desktop build 前に web/dist を生成・検証。
- frontendDist 整合、生成物存在、開発ホストパス混入を fail-closed で検査。
- verify.sh に build-desktop ゲートを追加。
- verify.sh 全ゲート成功（E2E 256、WDIO 1）。
