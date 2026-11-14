# Goal

Web 版 EPUB リーダーで、本文内の相対リンクから同じ本の章へ移動できるように
する。フラグメント付きのリンクは移動先の章で対象位置へスクロールし、外部
リンクと本の外を指すリンクは移動しない（fail-closed）。

## Scope

### 背景

ADR-0012 は目次からの章移動と本文内の `#fragment` リンクのスクロールを
実装したが、`ch2.xhtml` のような相対リンクによる章間移動は未対応で、
クリックしても何も起きない（ADR-0012・ADR-0013 の未決定）。core には
`resolve_reference` と spine の章一覧があり、解決に必要な情報は揃っている。

### 成果物

- `core/src/epub.rs`: 変更（`ChapterLink`、`resolve_chapter_link`、モジュール doc、
  テスト追加）
- `core/src/lib.rs`: 変更（`ChapterLink` の再エクスポート）
- `core/src/wasm.rs`: 変更（`WasmEpubBook::resolve_chapter_link`）
- `web/src/lib.rs`: 変更（本文内の相対リンクの処理）
- `docs/decisions/0014-relative-chapter-links.md`: 追加（決定と検証の記録）
- `GOAL.md`: 本契約

### 公開 API（追加分）

```rust
/// 本文内の相対リンクの解決結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterLink {
    /// 移動先の spine の index。
    pub chapter_index: usize,
    /// 移動先のフラグメント（`#` の後ろ）。無ければ `None`。
    pub fragment: Option<String>,
}

impl EpubBook {
    /// 章 `index` の本文にある相対リンク `href` を同じ本の章へ解決する。
    ///
    /// `#fragment` だけのリンクは同じ章を返す。スキーム付き URL・先頭 `/`・
    /// 空のリンク・spine に無いパス・ルートを越える参照は `None` を返し、
    /// 外部や本の外へは移動しない（fail-closed）。範囲外の `index` は
    /// [`CoreError::DocumentNotFound`]。`%XX` は復号して比較する。
    pub fn resolve_chapter_link(&self, index: usize, href: &str) -> CoreResult<Option<ChapterLink>>;
}
```

### 必須事項

1. 解決規則:
   - `split_fragment` でパスとフラグメントに分ける。
   - パスが空: フラグメントがあれば同じ章、無ければ `None`。
   - スキーム付き・先頭 `/`: `None`（外部・絶対参照は移動しない）。
   - `resolve_reference(章の所属ディレクトリ, パス, "章リンクの href")` で解決。
     エラー（空・絶対・スキーム・ルート越え）は `None`。
   - 解決したパスを spine の章の `href` と比較する。`%XX` は復号して比較し、
     復号できなければ生の値で比較する。
   - 一致する章が無ければ `None`。
2. Wasm:
   - `WasmEpubBook::resolve_chapter_link(index, href)` は
     `{ chapterIndex, fragment }` か `null` を返す。
3. Web:
   - クリックした `a[href]` が `#fragment` だけなら従来どおりスクロール。
   - `http` / `https` / `mailto` は従来どおり新しいタブ
     （`noopener,noreferrer`）。
   - それ以外は `resolve_chapter_link` で解決する。同じ章ならスクロール、
     別の章ならその章を表示してフラグメントへスクロールする。
   - `None` は移動せず状態メッセージ、`Err` も状態メッセージ。
   - サニタイズ済み HTML の `href` だけを扱い、サニタイザは変更しない。
   - 章を切り替えたときは本文のスクロール位置を先頭に戻す（フラグメント
     指定時はその位置へスクロールする）。
4. モジュール doc の更新:
   - 提供するものに本文内の相対リンクの解決を加える。
   - 範囲外の記述から「アンカー単位の移動」を外し、表示位置のスクロールは
     UI の責務であることを残す。
5. 既存の挙動を変えない:
   - 既存の 113 テストが通る。`chapter_html` などの公開 API を変えない。
   - 目次からの移動、画像解決、外部リンクの扱いを変えない。

### 必須テスト（`core/src/epub.rs` の `mod tests`）

- 別の章へのリンク（`ch2.xhtml`）が index 1・fragment `None` を返す。
- フラグメント付き（`ch2.xhtml#part`）が index 1・`Some("part")` を返す。
- `#part` だけのリンクが現在の章と同じ index を返す。
- サブディレクトリの章から `../text/ch2.xhtml` を解決する。
- `%XX` を含むリンク（`ch%202.xhtml`）を復号して一致させる。
- 外部スキーム（`https://example.com`）・先頭 `/`・空・`../../evil.xhtml` は
  `None`。
- spine に無いパス（`other.xhtml`）は `None`。
- 範囲外の index は `DocumentNotFound`。
- 既存の 113 テストが通る（合計 113 件以上）。

## Acceptance criteria

- 上記の必須事項がコードとテストで確認できる。
- 次がすべて成功する。
  - `cargo fmt --all -- --check`
  - `cargo check --all-targets --locked`
  - `cargo test --all-targets --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `cargo check -p openrustbooks-core --target wasm32-unknown-unknown --locked`
  - `cargo check -p openrustbooks-web --target wasm32-unknown-unknown --locked`
  - `trunk build --offline --dist <一時出力先>`（web）
- CI（`Seduce0068/workflows/rust-gates.yml@v1`）が成功する。

## Constraints

- 新規依存を追加しない。読み取り専用の方針を維持する。
- 外部への移動・外部リソースの読み込みを追加しない。
- `desktop/` を変更しない。
- プロダクションコードで `unwrap()` / `expect()` / `panic!` / `todo!` を
  使わない。`unsafe` を使わない。
- コメントと doc コメントは日本語で簡潔に書く。
- 無関係なリファクタリングをしない。
- `master` へ push する。force push はしない。
