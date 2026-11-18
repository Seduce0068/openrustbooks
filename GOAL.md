# Goal

wasm-pack test を CI のマージゲートへ追加する（R6 TD-M05、ADR-0196）。

## Scope

### 成果物

- `scripts/ci-wasm-test.sh`: wasm-pack 導入（不在時のみ 0.13.1）＋ `wasm-pack test --node --locked`
- `scripts/verify.sh`: 共通スクリプトを呼ぶ薄いラッパー
- `.github/workflows/ci.yml`: extra_gates に追加（verify.sh と同順）
- `docs/decisions/0196-ci-wasm-test-gate.md`: 追加
- `todo.md`: TD-M05 を完了へ移動

### 決定

- 共通ワークフローは変更せず、呼び出し側 extra_gates で対応。
- 単一情報源のスクリプトを verify.sh と CI から呼ぶ。
- 実行順は rumdl → wasm-core check → wasm-web check → wasm test。

## Acceptance criteria

- E2E 250 項目、全ゲート成功（検証済み）。
