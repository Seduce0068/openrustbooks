# Goal

ホーム画面に Reader / Editor の導線を追加する（R6 レビュー / R2-UX01）。

## Scope

### 背景

HomePage の nav は Settings / About のみで、Reader / Editor は URL 直入力が必要。
New File / Open Folder / Open File は on:click が無いのにクリック可能に見え、
FileTree は固定ダミー項目を表示する。利用者は主要機能へ到達できず、
操作可能と誤解する。

### 成果物

- `web/src/lib.rs`: HomePage の nav / toolbar / FileTree を変更
- `web/index.html`: 必要なら CSS のみ
- `docs/decisions/0288-home-navigation.md`: 追加
- `todo.md` / README / 索引 / log: 更新

### 必須事項

1. HomePage の nav から Reader / Editor へ遷移できる。
2. 既存の `/reader/:id` / `/editor/:id` を壊さない。
3. 未接続ボタンは disabled にし、理由を title / aria で示す。
4. ダミー FileTree は「未接続」と分かる表示へ変更する。
5. E2E 256 件・check 名・WDIO ゲートを維持し、verify.sh 全ゲート成功。

## Acceptance criteria

- ホーム画面から主要機能へ導線があり、未接続 UI が誤解を与えない。

## Constraints

- 新規ファイル作成 / フォルダ選択の実装は行わない。
- 外部サービス / 秘密情報は扱わない。
- `master` へ push する。force push はしない。

## 完了記録

- HomePage に Reader/Editor のサンプル導線を追加。
- 未接続ボタンを disabled 化し、ダミー FileTree を未接続表示へ変更。
- R4 指摘により E2E 用 ID を本番 UI から除去。
- verify.sh 全ゲート成功（E2E 256、WDIO 1）。
