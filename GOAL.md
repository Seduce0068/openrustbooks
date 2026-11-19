# Goal

ZIP 実展開サイズのセッション予算を追加した（R6 R1-archive#2、ADR-0251）。

## Scope

### 成果物

- `core/src/archive.rs`: AtomicU64 予算、CAS 予約／返金、test 5 件
- `docs/decisions/0251-expanded-size-budget.md`: 追加
- `todo.md`: R1-archive#2 完了、#3 残存
- README / 各索引 / log: 更新

### 決定

- 公開 API / Send+Sync / wasm を維持しつつ実展開の累計を制限。
- 同一 path の正当な再読は予算内で許可、枯渇後に拒否。
- 新規エラーは固定文言、path / 生エラー非含有。

## Acceptance criteria

- core 284 / E2E 255、全ゲート成功（検証済み）。
