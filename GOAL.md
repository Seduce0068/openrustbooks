# Goal

StorageBackend::read の offset / limit に上限を適用する（R6 PR-M04、ADR-0190）。

## Scope

### 成果物

- `core/src/io.rs`: DEFAULT_MAX_READ_BYTES / resolve_read_limit / probe、unit 5 件
- `docs/decisions/0190-read-limit-hardening.md`: 追加
- `todo.md`: PR-M04 を完了へ移動

### 決定

- effective = capabilities.max_file_size.unwrap_or(256 MiB)。
- 上限超過は clamp せず CoreError::validation。
- limit=None も 1 byte probe で超過検出。
- 他経路（desktop document.rs / web wasm.rs）は未決定として ADR に記録。

## Acceptance criteria

- core 246 / E2E 249、全ゲート成功（検証済み）。
