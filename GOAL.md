# Goal

EPUB 画像の自然寸法を取得して `width` / `height` 属性を付与する（ADR-0136 / 0137 の実装）。

## Scope

### 背景

ADR-0136 で「core で自然寸法を取得し、web の `resolve_images()` で
`width` / `height` 属性を付与する」方針を決定し、
ADR-0137 で `imagesize 0.15.0` の wasm32 互換性を検証済み。
位置復元 E2E のベースライン（ADR-0140 / 0146）と
画像縮小の安定化（ADR-0148）も完了している。

### 成果物

- `core/Cargo.toml` / `Cargo.lock`: 変更（`imagesize` 依存を追加）
- `core/src/wasm.rs`: 変更（自然寸法取得の Wasm API）
- `web/src/lib.rs`: 変更（`resolve_images()` での属性付与）
- `tools/verify-web-reader.py`: 変更（属性チェックを追加、期待総数を更新）
- `THIRD_PARTY_NOTICES.md`: 変更（MIT 通知）
- `docs/decisions/0149-image-dimension-reservation.md`: 追加
- `GOAL.md`: 本契約

### 必須事項

1. `imagesize = { version = "0.15", default-features = false, features = ["png","jpeg","gif","webp"] }` を core に追加する（新版・ライセンス・脆弱性調査は ADR-0137 済み）。
2. core に自然寸法取得の純粋関数を追加し、wasm32 / native の両方でビルドできるようにする。
3. `resolve_images()` で PNG / JPEG / WebP / GIF の画像に
   `width` / `height` を自然ピクセルで付与する。SVG は既存属性を尊重し、
   viewBox のみの場合は付与しない。
4. 0 / 非数 / 16384px 超の値は拒否（fail-closed）。取得失敗時は属性を付けない。
5. 既存の画像セトリング・警告・縮小ロジックを壊さない。
6. E2E に 2 件追加（`img` に自然寸法と一致する属性が付くこと、
   SVG の既存属性が維持されること）、期待総数を 229 → 231 に更新。
7. `cargo audit --no-fetch` で脆弱性 0 件・警告 baseline 10 のまま。
8. 全ゲート成功、E2E 231 項目 5 回連続 0 失敗。

## Acceptance criteria

- 6 ゲート + audit + wasm-test + E2E 231 が成功する。

## Constraints

- 新規依存は `imagesize` のみ（MIT・実行時依存 0・ADR-0137 で調査済み）。
- `unsafe` 禁止。
- 属性は「縦横比の予約」専用とし、警告・縮小判定には使わない。
- `master` へ push する。force push はしない。
