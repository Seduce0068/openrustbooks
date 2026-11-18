# Third-party license notices

以下は、本リポジトリを `AGPL-3.0-or-later` に変更する前に適用されていたライセンス本文です。
上流の著作権表示・許諾通知を保持するため、変更後もここに残します。

---

MIT License

Copyright (c) 2026 onoderakeita

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## imagesize 0.15.0

- ライセンス: MIT
- Copyright (c) 2017 Maiddog
- 出典: <https://crates.io/crates/imagesize>
- 用途: EPUB 画像の自然寸法取得（PNG / JPEG / GIF / WebP、ADR-0136 / 0137）

MIT License

Copyright (c) 2017 Maiddog

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## 外部依存ライセンス一覧（自動生成）

- 出典: `cargo metadata --locked --format-version 1` の出力。
  依存解決結果はリポジトリの `Cargo.lock` に固定されている。
- 対象: workspace ルート 3 クレート（`openrustbooks-core` / `openrustbooks-desktop` /
  `openrustbooks-web`）から dependency kind が `null`（通常依存）または `build` のエッジのみを
  辿って到達した外部クレート。dev-dependency のみから到達するクレートは含めない。
- ライセンス ID: 各クレートの metadata 上の `license` フィールドをそのまま転記したもの。
  値が存在しない場合は「未確認」と表記し、推測による補完は行わない。
- 出典 URL: 各クレートの crates.io ページ（`https://crates.io/crates/<name>`）。
- 注意: 本表はライセンス ID と出典リンクのみを記載する。full license text（ライセンス本文）は
  各クレートの配布物（crates.io の配布アーカイブに含まれる `LICENSE` / `LICENSE-*` 等）を参照すること。
  本文を推測して追加しない。
- 並び順: クレート名、バージョンの順。同名異なるバージョンは別行で記載する。

<!-- BEGIN AUTO-GENERATED DEPENDENCIES -->

| クレート名 | バージョン | ライセンス ID | 出典 URL |
| --- | --- | --- | --- |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | <https://crates.io/crates/adler2> |
| aho-corasick | 1.1.5 | Unlicense OR MIT | <https://crates.io/crates/aho-corasick> |
| alloc-no-stdlib | 3.0.0 | BSD-3-Clause | <https://crates.io/crates/alloc-no-stdlib> |
| alloc-stdlib | 0.3.0 | BSD-3-Clause | <https://crates.io/crates/alloc-stdlib> |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | <https://crates.io/crates/allocator-api2> |
| ammonia | 4.2.5 | MIT OR Apache-2.0 | <https://crates.io/crates/ammonia> |
| android_log-sys | 0.3.2 | MIT OR Apache-2.0 | <https://crates.io/crates/android_log-sys> |
| android_logger | 0.15.1 | MIT OR Apache-2.0 | <https://crates.io/crates/android_logger> |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 | <https://crates.io/crates/android_system_properties> |
| anstream | 1.0.0 | MIT OR Apache-2.0 | <https://crates.io/crates/anstream> |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | <https://crates.io/crates/anstyle> |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 | <https://crates.io/crates/anstyle-parse> |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 | <https://crates.io/crates/anstyle-query> |
| anstyle-wincon | 3.0.11 | MIT OR Apache-2.0 | <https://crates.io/crates/anstyle-wincon> |
| any_spawner | 0.3.0 | MIT | <https://crates.io/crates/any_spawner> |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | <https://crates.io/crates/anyhow> |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 | <https://crates.io/crates/async-broadcast> |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | <https://crates.io/crates/async-channel> |
| async-executor | 1.14.0 | Apache-2.0 OR MIT | <https://crates.io/crates/async-executor> |
| async-io | 2.6.0 | Apache-2.0 OR MIT | <https://crates.io/crates/async-io> |
| async-lock | 3.4.2 | Apache-2.0 OR MIT | <https://crates.io/crates/async-lock> |
| async-once-cell | 0.5.4 | MIT OR Apache-2.0 | <https://crates.io/crates/async-once-cell> |
| async-openai | 0.42.2 | MIT | <https://crates.io/crates/async-openai> |
| async-openai-macros | 0.3.0 | MIT | <https://crates.io/crates/async-openai-macros> |
| async-process | 2.5.0 | Apache-2.0 OR MIT | <https://crates.io/crates/async-process> |
| async-recursion | 1.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/async-recursion> |
| async-signal | 0.2.14 | Apache-2.0 OR MIT | <https://crates.io/crates/async-signal> |
| async-task | 4.7.1 | Apache-2.0 OR MIT | <https://crates.io/crates/async-task> |
| async-trait | 0.1.92 | MIT OR Apache-2.0 | <https://crates.io/crates/async-trait> |
| atk | 0.18.2 | MIT | <https://crates.io/crates/atk> |
| atk-sys | 0.18.2 | MIT | <https://crates.io/crates/atk-sys> |
| atomic | 0.6.1 | Apache-2.0/MIT | <https://crates.io/crates/atomic> |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | <https://crates.io/crates/atomic-waker> |
| attribute-derive | 0.11.0 | MIT OR Apache-2.0 | <https://crates.io/crates/attribute-derive> |
| attribute-derive-macro | 0.11.0 | MIT | <https://crates.io/crates/attribute-derive-macro> |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | <https://crates.io/crates/autocfg> |
| aws-lc-rs | 1.19.0 | ISC AND (Apache-2.0 OR ISC) | <https://crates.io/crates/aws-lc-rs> |
| aws-lc-sys | 0.47.0 | ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) | <https://crates.io/crates/aws-lc-sys> |
| base16 | 0.2.1 | CC0-1.0 | <https://crates.io/crates/base16> |
| base64 | 0.21.7 | MIT OR Apache-2.0 | <https://crates.io/crates/base64> |
| base64 | 0.22.1 | MIT OR Apache-2.0 | <https://crates.io/crates/base64> |
| base64 | 0.23.1 | MIT OR Apache-2.0 | <https://crates.io/crates/base64> |
| bincode | 1.3.3 | MIT | <https://crates.io/crates/bincode> |
| bit-set | 0.8.0 | Apache-2.0 OR MIT | <https://crates.io/crates/bit-set> |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT | <https://crates.io/crates/bit-vec> |
| bitflags | 1.3.2 | MIT/Apache-2.0 | <https://crates.io/crates/bitflags> |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | <https://crates.io/crates/bitflags> |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | <https://crates.io/crates/block-buffer> |
| block2 | 0.6.2 | MIT | <https://crates.io/crates/block2> |
| blocking | 1.7.0 | Apache-2.0 OR MIT | <https://crates.io/crates/blocking> |
| brotli | 9.1.1 | BSD-3-Clause AND MIT | <https://crates.io/crates/brotli> |
| brotli-decompressor | 6.2.0 | BSD-3-Clause/MIT | <https://crates.io/crates/brotli-decompressor> |
| bs58 | 0.5.1 | MIT/Apache-2.0 | <https://crates.io/crates/bs58> |
| bstr | 1.13.1 | MIT OR Apache-2.0 | <https://crates.io/crates/bstr> |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | <https://crates.io/crates/bumpalo> |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/bytemuck> |
| byteorder | 1.5.0 | Unlicense OR MIT | <https://crates.io/crates/byteorder> |
| bytes | 1.12.1 | MIT | <https://crates.io/crates/bytes> |
| cairo-rs | 0.18.5 | MIT | <https://crates.io/crates/cairo-rs> |
| cairo-sys-rs | 0.18.2 | MIT | <https://crates.io/crates/cairo-sys-rs> |
| camino | 1.2.6 | MIT OR Apache-2.0 | <https://crates.io/crates/camino> |
| cargo-platform | 0.1.9 | MIT OR Apache-2.0 | <https://crates.io/crates/cargo-platform> |
| cargo_metadata | 0.19.2 | MIT | <https://crates.io/crates/cargo_metadata> |
| cargo_toml | 1.0.1 | Apache-2.0 OR MIT | <https://crates.io/crates/cargo_toml> |
| caseless | 0.2.2 | MIT | <https://crates.io/crates/caseless> |
| castaway | 0.2.4 | MIT | <https://crates.io/crates/castaway> |
| cc | 1.10.0 | MIT OR Apache-2.0 | <https://crates.io/crates/cc> |
| cesu8 | 1.1.0 | Apache-2.0/MIT | <https://crates.io/crates/cesu8> |
| cfb | 0.14.0 | MIT | <https://crates.io/crates/cfb> |
| cfg-expr | 0.15.8 | MIT OR Apache-2.0 | <https://crates.io/crates/cfg-expr> |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | <https://crates.io/crates/cfg-if> |
| cfg_aliases | 0.2.2 | MIT | <https://crates.io/crates/cfg_aliases> |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 | <https://crates.io/crates/chacha20> |
| chrono | 0.4.45 | MIT OR Apache-2.0 | <https://crates.io/crates/chrono> |
| clap | 4.6.7 | MIT OR Apache-2.0 | <https://crates.io/crates/clap> |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | <https://crates.io/crates/clap_builder> |
| clap_complete | 4.6.11 | MIT OR Apache-2.0 | <https://crates.io/crates/clap_complete> |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 | <https://crates.io/crates/clap_derive> |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | <https://crates.io/crates/clap_lex> |
| cmake | 0.1.58 | MIT OR Apache-2.0 | <https://crates.io/crates/cmake> |
| codee | 0.3.5 | MIT OR Apache-2.0 | <https://crates.io/crates/codee> |
| collection_literals | 1.0.3 | MIT | <https://crates.io/crates/collection_literals> |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 | <https://crates.io/crates/colorchoice> |
| combine | 4.6.9 | MIT | <https://crates.io/crates/combine> |
| compact_str | 0.9.1 | MIT | <https://crates.io/crates/compact_str> |
| comrak | 0.59.0 | BSD-2-Clause | <https://crates.io/crates/comrak> |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | <https://crates.io/crates/concurrent-queue> |
| config | 0.15.27 | MIT OR Apache-2.0 | <https://crates.io/crates/config> |
| console_error_panic_hook | 0.1.7 | Apache-2.0/MIT | <https://crates.io/crates/console_error_panic_hook> |
| const-str | 1.1.0 | MIT | <https://crates.io/crates/const-str> |
| const_format | 0.2.37 | Zlib | <https://crates.io/crates/const_format> |
| const_format_proc_macros | 0.2.37 | Zlib | <https://crates.io/crates/const_format_proc_macros> |
| const_str_slice_concat | 0.1.0 | MIT | <https://crates.io/crates/const_str_slice_concat> |
| convert_case | 0.11.0 | MIT | <https://crates.io/crates/convert_case> |
| convert_case | 0.6.0 | MIT | <https://crates.io/crates/convert_case> |
| convert_case_extras | 0.2.0 | MIT | <https://crates.io/crates/convert_case_extras> |
| cookie | 0.18.2 | MIT OR Apache-2.0 | <https://crates.io/crates/cookie> |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 | <https://crates.io/crates/core-foundation> |
| core-foundation | 0.9.4 | MIT OR Apache-2.0 | <https://crates.io/crates/core-foundation> |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 | <https://crates.io/crates/core-foundation-sys> |
| core-graphics | 0.25.0 | MIT OR Apache-2.0 | <https://crates.io/crates/core-graphics> |
| core-graphics-types | 0.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/core-graphics-types> |
| core_detect | 1.0.0 | MIT/Apache-2.0 | <https://crates.io/crates/core_detect> |
| cow-utils | 0.1.3 | MIT | <https://crates.io/crates/cow-utils> |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | <https://crates.io/crates/cpufeatures> |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | <https://crates.io/crates/cpufeatures> |
| crc32fast | 1.5.3 | MIT OR Apache-2.0 | <https://crates.io/crates/crc32fast> |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 | <https://crates.io/crates/crossbeam-channel> |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 | <https://crates.io/crates/crossbeam-deque> |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 | <https://crates.io/crates/crossbeam-epoch> |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | <https://crates.io/crates/crossbeam-utils> |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | <https://crates.io/crates/crypto-common> |
| cssparser | 0.37.0 | MPL-2.0 | <https://crates.io/crates/cssparser> |
| cssparser | 0.38.0 | MPL-2.0 | <https://crates.io/crates/cssparser> |
| cssparser-macros | 0.7.1 | MPL-2.0 | <https://crates.io/crates/cssparser-macros> |
| ctor | 1.0.13 | Apache-2.0 OR MIT | <https://crates.io/crates/ctor> |
| darling | 0.20.11 | MIT | <https://crates.io/crates/darling> |
| darling | 0.24.1 | MIT | <https://crates.io/crates/darling> |
| darling_core | 0.20.11 | MIT | <https://crates.io/crates/darling_core> |
| darling_core | 0.24.1 | MIT | <https://crates.io/crates/darling_core> |
| darling_macro | 0.20.11 | MIT | <https://crates.io/crates/darling_macro> |
| darling_macro | 0.24.1 | MIT | <https://crates.io/crates/darling_macro> |
| dbus | 0.9.12 | Apache-2.0/MIT | <https://crates.io/crates/dbus> |
| defmt | 1.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/defmt> |
| defmt-macros | 1.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/defmt-macros> |
| defmt-parser | 1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/defmt-parser> |
| deranged | 0.5.9 | MIT OR Apache-2.0 | <https://crates.io/crates/deranged> |
| derive-where | 1.7.0 | MIT OR Apache-2.0 | <https://crates.io/crates/derive-where> |
| derive_builder | 0.20.2 | MIT OR Apache-2.0 | <https://crates.io/crates/derive_builder> |
| derive_builder_core | 0.20.2 | MIT OR Apache-2.0 | <https://crates.io/crates/derive_builder_core> |
| derive_builder_macro | 0.20.2 | MIT OR Apache-2.0 | <https://crates.io/crates/derive_builder_macro> |
| derive_more | 2.1.1 | MIT | <https://crates.io/crates/derive_more> |
| derive_more-impl | 2.1.1 | MIT | <https://crates.io/crates/derive_more-impl> |
| digest | 0.10.7 | MIT OR Apache-2.0 | <https://crates.io/crates/digest> |
| directories | 6.0.0 | MIT OR Apache-2.0 | <https://crates.io/crates/directories> |
| dirs | 7.0.0 | MIT OR Apache-2.0 | <https://crates.io/crates/dirs> |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 | <https://crates.io/crates/dirs-sys> |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/dispatch2> |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 | <https://crates.io/crates/displaydoc> |
| dlopen2 | 0.8.2 | MIT | <https://crates.io/crates/dlopen2> |
| dlopen2_derive | 0.4.3 | MIT | <https://crates.io/crates/dlopen2_derive> |
| dom_query | 0.28.0 | MIT | <https://crates.io/crates/dom_query> |
| dpi | 0.1.2 | Apache-2.0 AND MIT | <https://crates.io/crates/dpi> |
| dragonbox_ecma | 0.1.12 | Apache-2.0 WITH LLVM-exception OR BSL-1.0 | <https://crates.io/crates/dragonbox_ecma> |
| drain_filter_polyfill | 0.1.3 | MIT OR Apache-2.0 | <https://crates.io/crates/drain_filter_polyfill> |
| dtoa | 1.0.11 | MIT OR Apache-2.0 | <https://crates.io/crates/dtoa> |
| dtoa-short | 0.3.5 | MPL-2.0 | <https://crates.io/crates/dtoa-short> |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | <https://crates.io/crates/dunce> |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 | <https://crates.io/crates/dyn-clone> |
| either | 1.19.0 | MIT OR Apache-2.0 | <https://crates.io/crates/either> |
| either_of | 0.1.9 | MIT | <https://crates.io/crates/either_of> |
| embed-resource | 3.0.12 | MIT | <https://crates.io/crates/embed-resource> |
| embed_plist | 1.2.2 | MIT OR Apache-2.0 | <https://crates.io/crates/embed_plist> |
| encoding_rs | 0.8.42 | (Apache-2.0 OR MIT) AND BSD-3-Clause | <https://crates.io/crates/encoding_rs> |
| endi | 1.1.1 | MIT | <https://crates.io/crates/endi> |
| entities | 1.0.1 | MIT | <https://crates.io/crates/entities> |
| enumflags2 | 0.7.12 | MIT OR Apache-2.0 | <https://crates.io/crates/enumflags2> |
| enumflags2_derive | 0.7.12 | MIT OR Apache-2.0 | <https://crates.io/crates/enumflags2_derive> |
| env_filter | 0.1.4 | MIT OR Apache-2.0 | <https://crates.io/crates/env_filter> |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | <https://crates.io/crates/equivalent> |
| erased | 0.1.2 | MIT | <https://crates.io/crates/erased> |
| erased-serde | 0.4.11 | MIT OR Apache-2.0 | <https://crates.io/crates/erased-serde> |
| errno | 0.3.14 | MIT OR Apache-2.0 | <https://crates.io/crates/errno> |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | <https://crates.io/crates/event-listener> |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | <https://crates.io/crates/event-listener-strategy> |
| eventsource-stream | 0.2.3 | MIT OR Apache-2.0 | <https://crates.io/crates/eventsource-stream> |
| fancy-regex | 0.17.0 | MIT | <https://crates.io/crates/fancy-regex> |
| fancy-regex | 0.19.2 | MIT | <https://crates.io/crates/fancy-regex> |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | <https://crates.io/crates/fastrand> |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | <https://crates.io/crates/fdeflate> |
| fearless_simd | 1.2.0 | Apache-2.0 OR MIT | <https://crates.io/crates/fearless_simd> |
| fearless_simd_macros | 0.1.0 | Apache-2.0 OR MIT | <https://crates.io/crates/fearless_simd_macros> |
| fern | 0.7.1 | MIT | <https://crates.io/crates/fern> |
| field-offset | 0.3.6 | MIT OR Apache-2.0 | <https://crates.io/crates/field-offset> |
| figment | 0.10.19 | MIT OR Apache-2.0 | <https://crates.io/crates/figment> |
| filetime | 0.2.29 | MIT/Apache-2.0 | <https://crates.io/crates/filetime> |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | <https://crates.io/crates/find-msvc-tools> |
| finl_unicode | 1.5.0 | (MIT OR Apache-2.0) AND Unicode-DFS-2016 | <https://crates.io/crates/finl_unicode> |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | <https://crates.io/crates/flate2> |
| fnv | 1.0.7 | Apache-2.0 / MIT | <https://crates.io/crates/fnv> |
| foldhash | 0.2.0 | Zlib | <https://crates.io/crates/foldhash> |
| foreign-types | 0.5.0 | MIT/Apache-2.0 | <https://crates.io/crates/foreign-types> |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 | <https://crates.io/crates/foreign-types-macros> |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 | <https://crates.io/crates/foreign-types-shared> |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | <https://crates.io/crates/form_urlencoded> |
| fs_extra | 1.3.0 | MIT | <https://crates.io/crates/fs_extra> |
| futures | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures> |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-channel> |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-core> |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-executor> |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-io> |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | <https://crates.io/crates/futures-lite> |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-macro> |
| futures-sink | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-sink> |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-task> |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/futures-util> |
| gdk | 0.18.2 | MIT | <https://crates.io/crates/gdk> |
| gdk-pixbuf | 0.18.5 | MIT | <https://crates.io/crates/gdk-pixbuf> |
| gdk-pixbuf-sys | 0.18.0 | MIT | <https://crates.io/crates/gdk-pixbuf-sys> |
| gdk-sys | 0.18.2 | MIT | <https://crates.io/crates/gdk-sys> |
| gdkwayland-sys | 0.18.2 | MIT | <https://crates.io/crates/gdkwayland-sys> |
| gdkx11 | 0.18.2 | MIT | <https://crates.io/crates/gdkx11> |
| gdkx11-sys | 0.18.2 | MIT | <https://crates.io/crates/gdkx11-sys> |
| generic-array | 0.14.7 | MIT | <https://crates.io/crates/generic-array> |
| gethostname | 1.1.0 | Apache-2.0 | <https://crates.io/crates/gethostname> |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | <https://crates.io/crates/getrandom> |
| getrandom | 0.3.4 | MIT OR Apache-2.0 | <https://crates.io/crates/getrandom> |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | <https://crates.io/crates/getrandom> |
| gio | 0.18.4 | MIT | <https://crates.io/crates/gio> |
| gio-sys | 0.18.1 | MIT | <https://crates.io/crates/gio-sys> |
| glib | 0.18.5 | MIT | <https://crates.io/crates/glib> |
| glib-macros | 0.18.5 | MIT | <https://crates.io/crates/glib-macros> |
| glib-sys | 0.18.1 | MIT | <https://crates.io/crates/glib-sys> |
| glob | 0.3.4 | MIT OR Apache-2.0 | <https://crates.io/crates/glob> |
| global-hotkey | 0.8.0 | Apache-2.0 OR MIT | <https://crates.io/crates/global-hotkey> |
| globset | 0.4.21 | Unlicense OR MIT | <https://crates.io/crates/globset> |
| gloo | 0.12.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo> |
| gloo-console | 0.4.1 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-console> |
| gloo-dialogs | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-dialogs> |
| gloo-events | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-events> |
| gloo-file | 0.4.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-file> |
| gloo-history | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-history> |
| gloo-net | 0.6.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-net> |
| gloo-net | 0.7.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-net> |
| gloo-render | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-render> |
| gloo-storage | 0.4.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-storage> |
| gloo-timers | 0.4.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-timers> |
| gloo-utils | 0.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-utils> |
| gloo-utils | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-utils> |
| gloo-worker | 0.6.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-worker> |
| gloo-worker-macros | 0.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/gloo-worker-macros> |
| gobject-sys | 0.18.0 | MIT | <https://crates.io/crates/gobject-sys> |
| gtk | 0.18.2 | MIT | <https://crates.io/crates/gtk> |
| gtk-sys | 0.18.2 | MIT | <https://crates.io/crates/gtk-sys> |
| gtk3-macros | 0.18.2 | MIT | <https://crates.io/crates/gtk3-macros> |
| guardian | 1.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/guardian> |
| h2 | 0.4.20 | MIT | <https://crates.io/crates/h2> |
| hashbrown | 0.12.3 | MIT OR Apache-2.0 | <https://crates.io/crates/hashbrown> |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | <https://crates.io/crates/hashbrown> |
| heck | 0.4.1 | MIT OR Apache-2.0 | <https://crates.io/crates/heck> |
| heck | 0.5.0 | MIT OR Apache-2.0 | <https://crates.io/crates/heck> |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 | <https://crates.io/crates/hermit-abi> |
| hex | 0.4.3 | MIT OR Apache-2.0 | <https://crates.io/crates/hex> |
| html-escape | 0.2.15 | MIT | <https://crates.io/crates/html-escape> |
| html5ever | 0.39.0 | MIT OR Apache-2.0 | <https://crates.io/crates/html5ever> |
| html5ever | 0.40.1 | MIT OR Apache-2.0 | <https://crates.io/crates/html5ever> |
| http | 1.5.0 | MIT OR Apache-2.0 | <https://crates.io/crates/http> |
| http-body | 1.1.0 | MIT | <https://crates.io/crates/http-body> |
| http-body-util | 0.1.5 | MIT | <https://crates.io/crates/http-body-util> |
| httparse | 1.10.1 | MIT OR Apache-2.0 | <https://crates.io/crates/httparse> |
| hydration_context | 0.3.2 | MIT | <https://crates.io/crates/hydration_context> |
| hyper | 1.12.0 | MIT | <https://crates.io/crates/hyper> |
| hyper-rustls | 0.27.10 | Apache-2.0 OR ISC OR MIT | <https://crates.io/crates/hyper-rustls> |
| hyper-util | 0.1.21 | MIT | <https://crates.io/crates/hyper-util> |
| iana-time-zone | 0.1.65 | MIT OR Apache-2.0 | <https://crates.io/crates/iana-time-zone> |
| iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 | <https://crates.io/crates/iana-time-zone-haiku> |
| ico | 0.6.0 | MIT | <https://crates.io/crates/ico> |
| icu_collections | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_collections> |
| icu_locale_core | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_locale_core> |
| icu_locale_fallback | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_locale_fallback> |
| icu_locale_fallback_data | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_locale_fallback_data> |
| icu_normalizer | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_normalizer> |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_normalizer_data> |
| icu_properties | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_properties> |
| icu_properties_data | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_properties_data> |
| icu_provider | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_provider> |
| icu_segmenter | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_segmenter> |
| icu_segmenter_data | 2.4.0 | Unicode-3.0 | <https://crates.io/crates/icu_segmenter_data> |
| ident_case | 1.0.1 | MIT/Apache-2.0 | <https://crates.io/crates/ident_case> |
| idna | 1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/idna> |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | <https://crates.io/crates/idna_adapter> |
| ignore | 0.4.34 | Unlicense OR MIT | <https://crates.io/crates/ignore> |
| imagesize | 0.15.0 | MIT | <https://crates.io/crates/imagesize> |
| indexmap | 1.9.3 | Apache-2.0 OR MIT | <https://crates.io/crates/indexmap> |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | <https://crates.io/crates/indexmap> |
| infer | 0.22.0 | MIT | <https://crates.io/crates/infer> |
| inlinable_string | 0.1.15 | Apache-2.0/MIT | <https://crates.io/crates/inlinable_string> |
| interpolator | 0.5.0 | MIT OR Apache-2.0 | <https://crates.io/crates/interpolator> |
| ipnet | 2.12.2 | MIT OR Apache-2.0 | <https://crates.io/crates/ipnet> |
| is-docker | 0.2.0 | MIT | <https://crates.io/crates/is-docker> |
| is-wsl | 0.4.0 | MIT | <https://crates.io/crates/is-wsl> |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 | <https://crates.io/crates/is_terminal_polyfill> |
| itertools | 0.14.0 | MIT OR Apache-2.0 | <https://crates.io/crates/itertools> |
| itoa | 1.0.18 | MIT OR Apache-2.0 | <https://crates.io/crates/itoa> |
| javascriptcore-rs | 1.1.2 | MIT | <https://crates.io/crates/javascriptcore-rs> |
| javascriptcore-rs-sys | 1.1.1 | MIT | <https://crates.io/crates/javascriptcore-rs-sys> |
| jiff | 0.2.38 | Unlicense OR MIT | <https://crates.io/crates/jiff> |
| jiff-core | 0.1.1 | Unlicense OR MIT | <https://crates.io/crates/jiff-core> |
| jiff-static | 0.2.38 | Unlicense OR MIT | <https://crates.io/crates/jiff-static> |
| jiff-tzdb | 0.1.9 | Unlicense OR MIT | <https://crates.io/crates/jiff-tzdb> |
| jiff-tzdb-platform | 0.1.3 | Unlicense OR MIT | <https://crates.io/crates/jiff-tzdb-platform> |
| jni | 0.21.1 | MIT/Apache-2.0 | <https://crates.io/crates/jni> |
| jni | 0.22.4 | MIT OR Apache-2.0 | <https://crates.io/crates/jni> |
| jni-macros | 0.22.4 | MIT OR Apache-2.0 | <https://crates.io/crates/jni-macros> |
| jni-sys | 0.3.1 | MIT OR Apache-2.0 | <https://crates.io/crates/jni-sys> |
| jni-sys | 0.4.1 | MIT OR Apache-2.0 | <https://crates.io/crates/jni-sys> |
| jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 | <https://crates.io/crates/jni-sys-macros> |
| jobserver | 0.1.35 | MIT OR Apache-2.0 | <https://crates.io/crates/jobserver> |
| js-sys | 0.3.106 | MIT OR Apache-2.0 | <https://crates.io/crates/js-sys> |
| json-patch | 4.2.0 | MIT/Apache-2.0 | <https://crates.io/crates/json-patch> |
| jsonptr | 0.7.1 | MIT OR Apache-2.0 | <https://crates.io/crates/jsonptr> |
| keyboard-types | 0.7.0 | MIT OR Apache-2.0 | <https://crates.io/crates/keyboard-types> |
| keyboard-types | 0.8.3 | MIT OR Apache-2.0 | <https://crates.io/crates/keyboard-types> |
| konst | 0.2.20 | Zlib | <https://crates.io/crates/konst> |
| konst_macro_rules | 0.2.19 | Zlib | <https://crates.io/crates/konst_macro_rules> |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | <https://crates.io/crates/lazy_static> |
| leptos | 0.8.22 | MIT | <https://crates.io/crates/leptos> |
| leptos_config | 0.8.10 | MIT | <https://crates.io/crates/leptos_config> |
| leptos_dom | 0.8.8 | MIT | <https://crates.io/crates/leptos_dom> |
| leptos_hot_reload | 0.8.7 | MIT | <https://crates.io/crates/leptos_hot_reload> |
| leptos_macro | 0.8.19 | MIT | <https://crates.io/crates/leptos_macro> |
| leptos_router | 0.8.17 | MIT | <https://crates.io/crates/leptos_router> |
| leptos_router_macro | 0.8.7 | MIT | <https://crates.io/crates/leptos_router_macro> |
| leptos_server | 0.8.8 | MIT | <https://crates.io/crates/leptos_server> |
| libappindicator | 0.9.0 | Apache-2.0 OR MIT | <https://crates.io/crates/libappindicator> |
| libappindicator-sys | 0.9.0 | Apache-2.0 OR MIT | <https://crates.io/crates/libappindicator-sys> |
| libc | 0.2.191 | MIT OR Apache-2.0 | <https://crates.io/crates/libc> |
| libdbus-sys | 0.2.7 | Apache-2.0/MIT | <https://crates.io/crates/libdbus-sys> |
| libloading | 0.7.4 | ISC | <https://crates.io/crates/libloading> |
| libm | 0.2.16 | MIT | <https://crates.io/crates/libm> |
| libmimalloc-sys | 0.1.49 | MIT | <https://crates.io/crates/libmimalloc-sys> |
| libredox | 0.1.25 | MIT | <https://crates.io/crates/libredox> |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | <https://crates.io/crates/linux-raw-sys> |
| litemap | 0.8.3 | Unicode-3.0 | <https://crates.io/crates/litemap> |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | <https://crates.io/crates/lock_api> |
| lock_api | 0.5.0 | MIT OR Apache-2.0 | <https://crates.io/crates/lock_api> |
| log | 0.4.34 | MIT OR Apache-2.0 | <https://crates.io/crates/log> |
| lru-slab | 0.1.3 | MIT OR Apache-2.0 OR Zlib | <https://crates.io/crates/lru-slab> |
| mac-notification-sys | 0.6.15 | MIT/Apache-2.0 | <https://crates.io/crates/mac-notification-sys> |
| malloc_buf | 0.0.6 | MIT | <https://crates.io/crates/malloc_buf> |
| manyhow | 0.14.1 | MIT OR Apache-2.0 | <https://crates.io/crates/manyhow> |
| manyhow-macros | 0.14.1 | MIT OR Apache-2.0 | <https://crates.io/crates/manyhow-macros> |
| maplit | 1.0.2 | MIT/Apache-2.0 | <https://crates.io/crates/maplit> |
| markup5ever | 0.39.0 | MIT OR Apache-2.0 | <https://crates.io/crates/markup5ever> |
| markup5ever | 0.40.0 | MIT OR Apache-2.0 | <https://crates.io/crates/markup5ever> |
| matchers | 0.2.0 | MIT | <https://crates.io/crates/matchers> |
| memchr | 2.8.3 | Unlicense OR MIT | <https://crates.io/crates/memchr> |
| memchr-n | 0.1.10 | MIT OR Apache-2.0 | <https://crates.io/crates/memchr-n> |
| memoffset | 0.9.1 | MIT | <https://crates.io/crates/memoffset> |
| mimalloc | 0.1.52 | MIT | <https://crates.io/crates/mimalloc> |
| mime | 0.3.17 | MIT OR Apache-2.0 | <https://crates.io/crates/mime> |
| mime_guess | 2.0.5 | MIT | <https://crates.io/crates/mime_guess> |
| minimal-lexical | 0.2.1 | MIT/Apache-2.0 | <https://crates.io/crates/minimal-lexical> |
| minisign-verify | 0.3.0 | MIT | <https://crates.io/crates/minisign-verify> |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | <https://crates.io/crates/miniz_oxide> |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | <https://crates.io/crates/miniz_oxide> |
| mio | 1.2.4 | MIT | <https://crates.io/crates/mio> |
| muda | 0.20.0 | Apache-2.0 OR MIT | <https://crates.io/crates/muda> |
| multiversion_no_op | 1.0.0 | Apache-2.0 OR MIT | <https://crates.io/crates/multiversion_no_op> |
| ndk | 0.9.0 | MIT OR Apache-2.0 | <https://crates.io/crates/ndk> |
| ndk-context | 0.1.1 | MIT OR Apache-2.0 | <https://crates.io/crates/ndk-context> |
| ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 | <https://crates.io/crates/ndk-sys> |
| new_debug_unreachable | 1.0.6 | MIT | <https://crates.io/crates/new_debug_unreachable> |
| next_tuple | 0.1.0 | MIT | <https://crates.io/crates/next_tuple> |
| nix | 0.31.3 | MIT | <https://crates.io/crates/nix> |
| nom | 7.1.3 | MIT | <https://crates.io/crates/nom> |
| nonmax | 0.5.5 | MIT OR Apache-2.0 | <https://crates.io/crates/nonmax> |
| notify-rust | 4.18.2 | MIT OR Apache-2.0 | <https://crates.io/crates/notify-rust> |
| nu-ansi-term | 0.50.3 | MIT | <https://crates.io/crates/nu-ansi-term> |
| num-bigint | 0.4.8 | MIT OR Apache-2.0 | <https://crates.io/crates/num-bigint> |
| num-conv | 0.2.2 | MIT OR Apache-2.0 | <https://crates.io/crates/num-conv> |
| num-integer | 0.1.47 | MIT OR Apache-2.0 | <https://crates.io/crates/num-integer> |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | <https://crates.io/crates/num-traits> |
| num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | <https://crates.io/crates/num_enum> |
| num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | <https://crates.io/crates/num_enum_derive> |
| num_threads | 0.1.8 | MIT OR Apache-2.0 | <https://crates.io/crates/num_threads> |
| objc | 0.2.7 | MIT | <https://crates.io/crates/objc> |
| objc2 | 0.6.5 | MIT | <https://crates.io/crates/objc2> |
| objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-app-kit> |
| objc2-cloud-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-cloud-kit> |
| objc2-core-data | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-core-data> |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-core-foundation> |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-core-graphics> |
| objc2-core-image | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-core-image> |
| objc2-core-location | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-core-location> |
| objc2-core-text | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-core-text> |
| objc2-encode | 4.1.0 | MIT | <https://crates.io/crates/objc2-encode> |
| objc2-exception-helper | 0.1.1 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-exception-helper> |
| objc2-foundation | 0.3.2 | MIT | <https://crates.io/crates/objc2-foundation> |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-io-surface> |
| objc2-osa-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-osa-kit> |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-quartz-core> |
| objc2-ui-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-ui-kit> |
| objc2-user-notifications | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-user-notifications> |
| objc2-web-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/objc2-web-kit> |
| oco_ref | 0.2.1 | MIT | <https://crates.io/crates/oco_ref> |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | <https://crates.io/crates/once_cell> |
| once_cell_polyfill | 1.70.2 | MIT OR Apache-2.0 | <https://crates.io/crates/once_cell_polyfill> |
| open | 5.4.5 | MIT | <https://crates.io/crates/open> |
| openssl-probe | 0.2.2 | MIT OR Apache-2.0 | <https://crates.io/crates/openssl-probe> |
| option-ext | 0.2.0 | MPL-2.0 | <https://crates.io/crates/option-ext> |
| or_poisoned | 0.1.0 | MIT | <https://crates.io/crates/or_poisoned> |
| ordered-stream | 0.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/ordered-stream> |
| os_pipe | 1.2.3 | MIT | <https://crates.io/crates/os_pipe> |
| osakit | 0.3.1 | MIT OR Apache-2.0 | <https://crates.io/crates/osakit> |
| owo-colors | 4.4.0 | MIT | <https://crates.io/crates/owo-colors> |
| oxc-miette | 3.0.1 | Apache-2.0 | <https://crates.io/crates/oxc-miette> |
| oxc-miette-derive | 3.0.1 | Apache-2.0 | <https://crates.io/crates/oxc-miette-derive> |
| oxc_allocator | 0.137.0 | MIT | <https://crates.io/crates/oxc_allocator> |
| oxc_ast | 0.137.0 | MIT | <https://crates.io/crates/oxc_ast> |
| oxc_ast_macros | 0.137.0 | MIT | <https://crates.io/crates/oxc_ast_macros> |
| oxc_data_structures | 0.137.0 | MIT | <https://crates.io/crates/oxc_data_structures> |
| oxc_diagnostics | 0.137.0 | MIT | <https://crates.io/crates/oxc_diagnostics> |
| oxc_ecmascript | 0.137.0 | MIT | <https://crates.io/crates/oxc_ecmascript> |
| oxc_estree | 0.137.0 | MIT | <https://crates.io/crates/oxc_estree> |
| oxc_index | 5.0.0 | MIT | <https://crates.io/crates/oxc_index> |
| oxc_parser | 0.137.0 | MIT | <https://crates.io/crates/oxc_parser> |
| oxc_regular_expression | 0.137.0 | MIT | <https://crates.io/crates/oxc_regular_expression> |
| oxc_span | 0.137.0 | MIT | <https://crates.io/crates/oxc_span> |
| oxc_str | 0.137.0 | MIT | <https://crates.io/crates/oxc_str> |
| oxc_syntax | 0.137.0 | MIT | <https://crates.io/crates/oxc_syntax> |
| pango | 0.18.3 | MIT | <https://crates.io/crates/pango> |
| pango-sys | 0.18.0 | MIT | <https://crates.io/crates/pango-sys> |
| parking | 2.2.1 | Apache-2.0 OR MIT | <https://crates.io/crates/parking> |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | <https://crates.io/crates/parking_lot> |
| parking_lot | 0.13.0 | MIT OR Apache-2.0 | <https://crates.io/crates/parking_lot> |
| parking_lot_core | 0.10.0 | MIT OR Apache-2.0 | <https://crates.io/crates/parking_lot_core> |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | <https://crates.io/crates/parking_lot_core> |
| paste | 1.0.15 | MIT OR Apache-2.0 | <https://crates.io/crates/paste> |
| pathdiff | 0.2.3 | MIT/Apache-2.0 | <https://crates.io/crates/pathdiff> |
| pear | 0.2.9 | MIT OR Apache-2.0 | <https://crates.io/crates/pear> |
| pear_codegen | 0.2.9 | MIT OR Apache-2.0 | <https://crates.io/crates/pear_codegen> |
| pep440_rs | 0.7.3 | Apache-2.0 OR BSD-2-Clause | <https://crates.io/crates/pep440_rs> |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | <https://crates.io/crates/percent-encoding> |
| phf | 0.13.1 | MIT | <https://crates.io/crates/phf> |
| phf | 0.14.0 | MIT | <https://crates.io/crates/phf> |
| phf_codegen | 0.13.1 | MIT | <https://crates.io/crates/phf_codegen> |
| phf_codegen | 0.14.0 | MIT | <https://crates.io/crates/phf_codegen> |
| phf_generator | 0.13.1 | MIT | <https://crates.io/crates/phf_generator> |
| phf_generator | 0.14.0 | MIT | <https://crates.io/crates/phf_generator> |
| phf_macros | 0.13.1 | MIT | <https://crates.io/crates/phf_macros> |
| phf_shared | 0.13.1 | MIT | <https://crates.io/crates/phf_shared> |
| phf_shared | 0.14.0 | MIT | <https://crates.io/crates/phf_shared> |
| pin-project | 1.1.13 | Apache-2.0 OR MIT | <https://crates.io/crates/pin-project> |
| pin-project-internal | 1.1.13 | Apache-2.0 OR MIT | <https://crates.io/crates/pin-project-internal> |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | <https://crates.io/crates/pin-project-lite> |
| pinned | 0.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/pinned> |
| piper | 0.2.5 | MIT OR Apache-2.0 | <https://crates.io/crates/piper> |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | <https://crates.io/crates/pkg-config> |
| plist | 1.10.3 | MIT | <https://crates.io/crates/plist> |
| png | 0.18.1 | MIT OR Apache-2.0 | <https://crates.io/crates/png> |
| polling | 3.11.0 | Apache-2.0 OR MIT | <https://crates.io/crates/polling> |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | <https://crates.io/crates/portable-atomic> |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT | <https://crates.io/crates/portable-atomic-util> |
| potential_utf | 0.1.6 | Unicode-3.0 | <https://crates.io/crates/potential_utf> |
| powerfmt | 0.2.1 | MIT OR Apache-2.0 | <https://crates.io/crates/powerfmt> |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 | <https://crates.io/crates/ppv-lite86> |
| precomputed-hash | 0.1.1 | MIT | <https://crates.io/crates/precomputed-hash> |
| prettyplease | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/prettyplease> |
| proc-macro-crate | 1.3.1 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro-crate> |
| proc-macro-crate | 2.0.0 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro-crate> |
| proc-macro-crate | 3.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro-crate> |
| proc-macro-error | 1.0.4 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro-error> |
| proc-macro-error-attr | 1.0.4 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro-error-attr> |
| proc-macro-utils | 0.10.0 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro-utils> |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | <https://crates.io/crates/proc-macro2> |
| proc-macro2-diagnostics | 0.10.1 | MIT/Apache-2.0 | <https://crates.io/crates/proc-macro2-diagnostics> |
| proc-macro2-diagnostics | 0.11.0 | MIT/Apache-2.0 | <https://crates.io/crates/proc-macro2-diagnostics> |
| pulldown-cmark | 0.13.6 | MIT | <https://crates.io/crates/pulldown-cmark> |
| pulldown-cmark-escape | 0.11.0 | MIT | <https://crates.io/crates/pulldown-cmark-escape> |
| quick-xml | 0.42.0 | MIT | <https://crates.io/crates/quick-xml> |
| quinn | 0.11.12 | MIT OR Apache-2.0 | <https://crates.io/crates/quinn> |
| quinn-proto | 0.11.19 | MIT OR Apache-2.0 | <https://crates.io/crates/quinn-proto> |
| quinn-udp | 0.5.16 | MIT OR Apache-2.0 | <https://crates.io/crates/quinn-udp> |
| quote | 1.0.47 | MIT OR Apache-2.0 | <https://crates.io/crates/quote> |
| quote-use | 0.9.0 | MIT | <https://crates.io/crates/quote-use> |
| quote-use-macros | 0.9.0 | MIT | <https://crates.io/crates/quote-use-macros> |
| r-efi | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | <https://crates.io/crates/r-efi> |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | <https://crates.io/crates/r-efi> |
| rand | 0.10.3 | MIT OR Apache-2.0 | <https://crates.io/crates/rand> |
| rand | 0.9.5 | MIT OR Apache-2.0 | <https://crates.io/crates/rand> |
| rand_chacha | 0.9.0 | MIT OR Apache-2.0 | <https://crates.io/crates/rand_chacha> |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | <https://crates.io/crates/rand_core> |
| rand_core | 0.9.5 | MIT OR Apache-2.0 | <https://crates.io/crates/rand_core> |
| rand_pcg | 0.10.2 | MIT OR Apache-2.0 | <https://crates.io/crates/rand_pcg> |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | <https://crates.io/crates/raw-window-handle> |
| reactive_graph | 0.2.16 | MIT | <https://crates.io/crates/reactive_graph> |
| reactive_stores | 0.4.4 | MIT | <https://crates.io/crates/reactive_stores> |
| reactive_stores_macro | 0.4.4 | MIT | <https://crates.io/crates/reactive_stores_macro> |
| redox_syscall | 0.5.18 | MIT | <https://crates.io/crates/redox_syscall> |
| redox_syscall | 0.9.5 | MIT | <https://crates.io/crates/redox_syscall> |
| redox_users | 0.5.3 | MIT | <https://crates.io/crates/redox_users> |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 | <https://crates.io/crates/ref-cast> |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 | <https://crates.io/crates/ref-cast-impl> |
| regex | 1.13.1 | MIT OR Apache-2.0 | <https://crates.io/crates/regex> |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | <https://crates.io/crates/regex-automata> |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | <https://crates.io/crates/regex-syntax> |
| reqwest | 0.13.5 | MIT OR Apache-2.0 | <https://crates.io/crates/reqwest> |
| rfd | 0.17.2 | MIT | <https://crates.io/crates/rfd> |
| ring | 0.17.14 | Apache-2.0 AND ISC | <https://crates.io/crates/ring> |
| rstml | 0.13.1 | MIT | <https://crates.io/crates/rstml> |
| rumdl | 0.3.2 | MIT | <https://crates.io/crates/rumdl> |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | <https://crates.io/crates/rustc-hash> |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 | <https://crates.io/crates/rustc_version> |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | <https://crates.io/crates/rustix> |
| rustls | 0.23.46 | Apache-2.0 OR ISC OR MIT | <https://crates.io/crates/rustls> |
| rustls-native-certs | 0.8.5 | Apache-2.0 OR ISC OR MIT | <https://crates.io/crates/rustls-native-certs> |
| rustls-pki-types | 1.15.1 | MIT OR Apache-2.0 | <https://crates.io/crates/rustls-pki-types> |
| rustls-platform-verifier | 0.7.1 | MIT OR Apache-2.0 | <https://crates.io/crates/rustls-platform-verifier> |
| rustls-platform-verifier-android | 0.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/rustls-platform-verifier-android> |
| rustls-webpki | 0.103.16 | ISC | <https://crates.io/crates/rustls-webpki> |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | <https://crates.io/crates/rustversion> |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | <https://crates.io/crates/ryu> |
| same-file | 1.0.6 | Unlicense/MIT | <https://crates.io/crates/same-file> |
| schannel | 0.1.30 | MIT | <https://crates.io/crates/schannel> |
| schemars | 0.8.22 | MIT | <https://crates.io/crates/schemars> |
| schemars | 0.9.0 | MIT | <https://crates.io/crates/schemars> |
| schemars | 1.2.3 | MIT | <https://crates.io/crates/schemars> |
| schemars_derive | 0.8.22 | MIT | <https://crates.io/crates/schemars_derive> |
| schemars_derive | 1.2.3 | MIT | <https://crates.io/crates/schemars_derive> |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | <https://crates.io/crates/scopeguard> |
| secrecy | 0.10.3 | Apache-2.0 OR MIT | <https://crates.io/crates/secrecy> |
| security-framework | 3.7.0 | MIT OR Apache-2.0 | <https://crates.io/crates/security-framework> |
| security-framework-sys | 2.17.0 | MIT OR Apache-2.0 | <https://crates.io/crates/security-framework-sys> |
| selectors | 0.38.0 | MPL-2.0 | <https://crates.io/crates/selectors> |
| semver | 1.0.28 | MIT OR Apache-2.0 | <https://crates.io/crates/semver> |
| send_wrapper | 0.6.0 | MIT/Apache-2.0 | <https://crates.io/crates/send_wrapper> |
| seq-macro | 0.3.6 | MIT OR Apache-2.0 | <https://crates.io/crates/seq-macro> |
| serde | 1.0.229 | MIT OR Apache-2.0 | <https://crates.io/crates/serde> |
| serde-untagged | 0.1.9 | MIT OR Apache-2.0 | <https://crates.io/crates/serde-untagged> |
| serde-wasm-bindgen | 0.6.5 | MIT | <https://crates.io/crates/serde-wasm-bindgen> |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_core> |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_derive> |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_derive_internals> |
| serde_derive_internals | 0.30.0 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_derive_internals> |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_json> |
| serde_norway | 0.9.42 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_norway> |
| serde_qs | 0.15.0 | MIT/Apache-2.0 | <https://crates.io/crates/serde_qs> |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_repr> |
| serde_spanned | 0.6.9 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_spanned> |
| serde_spanned | 1.1.2 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_spanned> |
| serde_urlencoded | 0.7.1 | MIT/Apache-2.0 | <https://crates.io/crates/serde_urlencoded> |
| serde_with | 3.24.0 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_with> |
| serde_with_macros | 3.24.0 | MIT OR Apache-2.0 | <https://crates.io/crates/serde_with_macros> |
| serde_yaml | 0.9.34+deprecated | MIT OR Apache-2.0 | <https://crates.io/crates/serde_yaml> |
| serialize-to-javascript | 0.1.2 | MIT OR Apache-2.0 | <https://crates.io/crates/serialize-to-javascript> |
| serialize-to-javascript-impl | 0.1.2 | MIT OR Apache-2.0 | <https://crates.io/crates/serialize-to-javascript-impl> |
| server_fn | 0.8.13 | MIT | <https://crates.io/crates/server_fn> |
| server_fn_macro | 0.8.12 | MIT | <https://crates.io/crates/server_fn_macro> |
| server_fn_macro_default | 0.8.6 | MIT | <https://crates.io/crates/server_fn_macro_default> |
| servo_arc | 0.4.3 | MIT OR Apache-2.0 | <https://crates.io/crates/servo_arc> |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | <https://crates.io/crates/sha2> |
| sharded-slab | 0.1.7 | MIT | <https://crates.io/crates/sharded-slab> |
| shared_child | 1.1.2 | MIT | <https://crates.io/crates/shared_child> |
| shlex | 2.0.1 | MIT OR Apache-2.0 | <https://crates.io/crates/shlex> |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 | <https://crates.io/crates/signal-hook-registry> |
| simd-adler32 | 0.3.10 | MIT | <https://crates.io/crates/simd-adler32> |
| simd_cesu8 | 1.2.0 | Apache-2.0 OR MIT | <https://crates.io/crates/simd_cesu8> |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 | <https://crates.io/crates/simdutf8> |
| similar | 3.2.1 | Apache-2.0 | <https://crates.io/crates/similar> |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | <https://crates.io/crates/siphasher> |
| slab | 0.4.12 | MIT | <https://crates.io/crates/slab> |
| slotmap | 1.1.1 | Zlib | <https://crates.io/crates/slotmap> |
| smallvec | 1.16.4 | MIT OR Apache-2.0 | <https://crates.io/crates/smallvec> |
| smawk | 0.3.3 | MIT | <https://crates.io/crates/smawk> |
| socket2 | 0.6.5 | MIT OR Apache-2.0 | <https://crates.io/crates/socket2> |
| softbuffer | 0.4.8 | MIT OR Apache-2.0 | <https://crates.io/crates/softbuffer> |
| soup3 | 0.5.0 | MIT | <https://crates.io/crates/soup3> |
| soup3-sys | 0.5.0 | MIT | <https://crates.io/crates/soup3-sys> |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | <https://crates.io/crates/stable_deref_trait> |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/static_assertions> |
| static_vcruntime | 3.0.0 | MIT OR Apache-2.0 OR Zlib | <https://crates.io/crates/static_vcruntime> |
| string_cache | 0.11.0 | MIT OR Apache-2.0 | <https://crates.io/crates/string_cache> |
| string_cache | 0.9.0 | MIT OR Apache-2.0 | <https://crates.io/crates/string_cache> |
| string_cache_codegen | 0.11.2 | MIT OR Apache-2.0 | <https://crates.io/crates/string_cache_codegen> |
| string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 | <https://crates.io/crates/string_cache_codegen> |
| strsim | 0.11.1 | MIT | <https://crates.io/crates/strsim> |
| subtle | 2.6.1 | BSD-3-Clause | <https://crates.io/crates/subtle> |
| swift-rs | 1.0.9 | MIT OR Apache-2.0 | <https://crates.io/crates/swift-rs> |
| syn | 1.0.109 | MIT OR Apache-2.0 | <https://crates.io/crates/syn> |
| syn | 2.0.119 | MIT OR Apache-2.0 | <https://crates.io/crates/syn> |
| syn | 3.0.9 | MIT OR Apache-2.0 | <https://crates.io/crates/syn> |
| sync_wrapper | 1.0.2 | Apache-2.0 | <https://crates.io/crates/sync_wrapper> |
| synstructure | 0.14.0 | MIT | <https://crates.io/crates/synstructure> |
| system-configuration | 0.7.0 | MIT OR Apache-2.0 | <https://crates.io/crates/system-configuration> |
| system-configuration-sys | 0.6.0 | MIT OR Apache-2.0 | <https://crates.io/crates/system-configuration-sys> |
| system-deps | 6.2.2 | MIT OR Apache-2.0 | <https://crates.io/crates/system-deps> |
| tachys | 0.2.19 | MIT | <https://crates.io/crates/tachys> |
| tao | 0.37.2 | Apache-2.0 | <https://crates.io/crates/tao> |
| tao-macros | 0.1.4 | MIT OR Apache-2.0 | <https://crates.io/crates/tao-macros> |
| tar | 0.4.46 | MIT OR Apache-2.0 | <https://crates.io/crates/tar> |
| target-lexicon | 0.12.16 | Apache-2.0 WITH LLVM-exception | <https://crates.io/crates/target-lexicon> |
| tauri | 2.12.5 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri> |
| tauri-build | 2.7.4 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-build> |
| tauri-codegen | 2.7.4 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-codegen> |
| tauri-macros | 2.7.4 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-macros> |
| tauri-plugin | 2.7.2 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin> |
| tauri-plugin-dialog | 2.8.4 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-dialog> |
| tauri-plugin-fs | 2.7.1 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-fs> |
| tauri-plugin-global-shortcut | 2.4.1 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-global-shortcut> |
| tauri-plugin-log | 2.10.1 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-log> |
| tauri-plugin-notification | 2.8.0 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-notification> |
| tauri-plugin-opener | 2.7.2 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-opener> |
| tauri-plugin-shell | 2.6.0 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-shell> |
| tauri-plugin-store | 2.5.0 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-store> |
| tauri-plugin-updater | 2.14.1 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-plugin-updater> |
| tauri-runtime | 2.12.2 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-runtime> |
| tauri-runtime-wry | 2.12.3 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-runtime-wry> |
| tauri-utils | 2.10.2 | Apache-2.0 OR MIT | <https://crates.io/crates/tauri-utils> |
| tauri-winres | 0.3.7 | MIT | <https://crates.io/crates/tauri-winres> |
| tauri-winrt-notification | 0.8.1 | MIT OR Apache-2.0 | <https://crates.io/crates/tauri-winrt-notification> |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | <https://crates.io/crates/tempfile> |
| tendril | 0.5.1 | MIT OR Apache-2.0 | <https://crates.io/crates/tendril> |
| textwrap | 0.16.4 | MIT | <https://crates.io/crates/textwrap> |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | <https://crates.io/crates/thiserror> |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | <https://crates.io/crates/thiserror> |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | <https://crates.io/crates/thiserror-impl> |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | <https://crates.io/crates/thiserror-impl> |
| thread_local | 1.1.10 | MIT OR Apache-2.0 | <https://crates.io/crates/thread_local> |
| throw_error | 0.3.1 | MIT | <https://crates.io/crates/throw_error> |
| tiktoken-rs | 0.12.1 | MIT | <https://crates.io/crates/tiktoken-rs> |
| tikv-jemalloc-sys | 0.6.1+5.3.0-1-ge13ca993e8ccb9ba9847cc330696e02839f328f7 | MIT/Apache-2.0 | <https://crates.io/crates/tikv-jemalloc-sys> |
| tikv-jemallocator | 0.6.1 | MIT/Apache-2.0 | <https://crates.io/crates/tikv-jemallocator> |
| time | 0.3.55 | MIT OR Apache-2.0 | <https://crates.io/crates/time> |
| time-core | 0.1.9 | MIT OR Apache-2.0 | <https://crates.io/crates/time-core> |
| time-macros | 0.2.32 | MIT OR Apache-2.0 | <https://crates.io/crates/time-macros> |
| tinystr | 0.8.5 | Unicode-3.0 | <https://crates.io/crates/tinystr> |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | <https://crates.io/crates/tinyvec> |
| tokio | 1.54.1 | MIT | <https://crates.io/crates/tokio> |
| tokio-macros | 2.7.2 | MIT | <https://crates.io/crates/tokio-macros> |
| tokio-rustls | 0.26.6 | MIT OR Apache-2.0 | <https://crates.io/crates/tokio-rustls> |
| tokio-stream | 0.1.19 | MIT | <https://crates.io/crates/tokio-stream> |
| tokio-util | 0.7.20 | MIT | <https://crates.io/crates/tokio-util> |
| toml | 0.8.2 | MIT OR Apache-2.0 | <https://crates.io/crates/toml> |
| toml | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/toml> |
| toml_datetime | 0.6.11 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_datetime> |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_datetime> |
| toml_edit | 0.19.15 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_edit> |
| toml_edit | 0.20.2 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_edit> |
| toml_edit | 0.22.27 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_edit> |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_edit> |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_parser> |
| toml_writer | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/toml_writer> |
| tower | 0.5.3 | MIT | <https://crates.io/crates/tower> |
| tower-http | 0.6.11 | MIT | <https://crates.io/crates/tower-http> |
| tower-layer | 0.3.3 | MIT | <https://crates.io/crates/tower-layer> |
| tower-service | 0.3.3 | MIT | <https://crates.io/crates/tower-service> |
| tracing | 0.1.44 | MIT | <https://crates.io/crates/tracing> |
| tracing-attributes | 0.1.31 | MIT | <https://crates.io/crates/tracing-attributes> |
| tracing-core | 0.1.36 | MIT | <https://crates.io/crates/tracing-core> |
| tracing-log | 0.2.0 | MIT | <https://crates.io/crates/tracing-log> |
| tracing-subscriber | 0.3.23 | MIT | <https://crates.io/crates/tracing-subscriber> |
| tracing-wasm | 0.2.1 | MIT OR Apache-2.0 | <https://crates.io/crates/tracing-wasm> |
| tray-icon | 0.25.1 | MIT OR Apache-2.0 | <https://crates.io/crates/tray-icon> |
| try-lock | 0.2.5 | MIT | <https://crates.io/crates/try-lock> |
| typed-arena | 2.0.2 | MIT | <https://crates.io/crates/typed-arena> |
| typed-builder | 0.23.2 | MIT OR Apache-2.0 | <https://crates.io/crates/typed-builder> |
| typed-builder-macro | 0.23.2 | MIT OR Apache-2.0 | <https://crates.io/crates/typed-builder-macro> |
| typed-path | 0.13.0 | MIT OR Apache-2.0 | <https://crates.io/crates/typed-path> |
| typeid | 1.0.3 | MIT OR Apache-2.0 | <https://crates.io/crates/typeid> |
| typenum | 1.20.1 | MIT OR Apache-2.0 | <https://crates.io/crates/typenum> |
| uds_windows | 1.2.1 | MIT | <https://crates.io/crates/uds_windows> |
| uncased | 0.9.10 | MIT OR Apache-2.0 | <https://crates.io/crates/uncased> |
| unicase | 2.10.0 | MIT OR Apache-2.0 | <https://crates.io/crates/unicase> |
| unicode-id | 0.3.7 | MIT OR Apache-2.0 | <https://crates.io/crates/unicode-id> |
| unicode-id-start | 1.5.0 | (MIT OR Apache-2.0) AND Unicode-3.0 | <https://crates.io/crates/unicode-id-start> |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | <https://crates.io/crates/unicode-ident> |
| unicode-normalization | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | <https://crates.io/crates/unicode-normalization> |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | <https://crates.io/crates/unicode-segmentation> |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 | <https://crates.io/crates/unicode-width> |
| unicode-xid | 0.2.6 | MIT OR Apache-2.0 | <https://crates.io/crates/unicode-xid> |
| unsafe-libyaml | 0.2.11 | MIT | <https://crates.io/crates/unsafe-libyaml> |
| unsafe-libyaml-norway | 0.2.15 | MIT | <https://crates.io/crates/unsafe-libyaml-norway> |
| unscanny | 0.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/unscanny> |
| untrusted | 0.9.0 | ISC | <https://crates.io/crates/untrusted> |
| url | 2.5.8 | MIT OR Apache-2.0 | <https://crates.io/crates/url> |
| urlpattern | 0.6.0 | MIT | <https://crates.io/crates/urlpattern> |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | <https://crates.io/crates/utf8_iter> |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | <https://crates.io/crates/utf8parse> |
| uuid | 1.29.0 | Apache-2.0 OR MIT | <https://crates.io/crates/uuid> |
| valuable | 0.1.1 | MIT | <https://crates.io/crates/valuable> |
| version-compare | 0.2.1 | MIT | <https://crates.io/crates/version-compare> |
| version_check | 0.9.5 | MIT/Apache-2.0 | <https://crates.io/crates/version_check> |
| vswhom | 0.1.0 | MIT | <https://crates.io/crates/vswhom> |
| vswhom-sys | 0.1.3 | MIT | <https://crates.io/crates/vswhom-sys> |
| walkdir | 2.5.0 | Unlicense/MIT | <https://crates.io/crates/walkdir> |
| want | 0.3.2 | MIT | <https://crates.io/crates/want> |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | <https://crates.io/crates/wasi> |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | <https://crates.io/crates/wasip2> |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm-bindgen> |
| wasm-bindgen-futures | 0.4.79 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm-bindgen-futures> |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm-bindgen-macro> |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm-bindgen-macro-support> |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm-bindgen-shared> |
| wasm-streams | 0.5.0 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm-streams> |
| wasm_split_helpers | 0.2.3 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm_split_helpers> |
| wasm_split_macros | 0.2.2 | MIT OR Apache-2.0 | <https://crates.io/crates/wasm_split_macros> |
| web-sys | 0.3.106 | MIT OR Apache-2.0 | <https://crates.io/crates/web-sys> |
| web-time | 1.1.0 | MIT OR Apache-2.0 | <https://crates.io/crates/web-time> |
| web_atoms | 0.2.6 | MIT OR Apache-2.0 | <https://crates.io/crates/web_atoms> |
| web_atoms | 0.3.0 | MIT OR Apache-2.0 | <https://crates.io/crates/web_atoms> |
| webkit2gtk | 2.0.2 | MIT | <https://crates.io/crates/webkit2gtk> |
| webkit2gtk-sys | 2.0.2 | MIT | <https://crates.io/crates/webkit2gtk-sys> |
| webpki-root-certs | 1.0.11 | CDLA-Permissive-2.0 | <https://crates.io/crates/webpki-root-certs> |
| webview2-com | 0.39.1 | MIT | <https://crates.io/crates/webview2-com> |
| webview2-com-macros | 0.8.1 | MIT | <https://crates.io/crates/webview2-com-macros> |
| webview2-com-sys | 0.39.1 | MIT | <https://crates.io/crates/webview2-com-sys> |
| winapi | 0.3.9 | MIT/Apache-2.0 | <https://crates.io/crates/winapi> |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | <https://crates.io/crates/winapi-i686-pc-windows-gnu> |
| winapi-util | 0.1.11 | Unlicense OR MIT | <https://crates.io/crates/winapi-util> |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | <https://crates.io/crates/winapi-x86_64-pc-windows-gnu> |
| window-vibrancy | 0.8.1 | Apache-2.0 OR MIT | <https://crates.io/crates/window-vibrancy> |
| windows | 0.62.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows> |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-collections> |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-core> |
| windows-future | 0.3.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-future> |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-implement> |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-interface> |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-link> |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-numerics> |
| windows-registry | 0.6.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-registry> |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-result> |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-strings> |
| windows-sys | 0.45.0 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-sys> |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-sys> |
| windows-sys | 0.59.0 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-sys> |
| windows-sys | 0.60.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-sys> |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-sys> |
| windows-targets | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-targets> |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-targets> |
| windows-targets | 0.53.5 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-targets> |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-threading> |
| windows-version | 0.1.7 | MIT OR Apache-2.0 | <https://crates.io/crates/windows-version> |
| windows_aarch64_gnullvm | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_aarch64_gnullvm> |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_aarch64_gnullvm> |
| windows_aarch64_gnullvm | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_aarch64_gnullvm> |
| windows_aarch64_msvc | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_aarch64_msvc> |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_aarch64_msvc> |
| windows_aarch64_msvc | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_aarch64_msvc> |
| windows_i686_gnu | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_gnu> |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_gnu> |
| windows_i686_gnu | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_gnu> |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_gnullvm> |
| windows_i686_gnullvm | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_gnullvm> |
| windows_i686_msvc | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_msvc> |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_msvc> |
| windows_i686_msvc | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_i686_msvc> |
| windows_x86_64_gnu | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_gnu> |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_gnu> |
| windows_x86_64_gnu | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_gnu> |
| windows_x86_64_gnullvm | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_gnullvm> |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_gnullvm> |
| windows_x86_64_gnullvm | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_gnullvm> |
| windows_x86_64_msvc | 0.42.2 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_msvc> |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_msvc> |
| windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 | <https://crates.io/crates/windows_x86_64_msvc> |
| winnow | 0.5.40 | MIT | <https://crates.io/crates/winnow> |
| winnow | 0.7.15 | MIT | <https://crates.io/crates/winnow> |
| winnow | 1.0.4 | MIT | <https://crates.io/crates/winnow> |
| winreg | 0.56.0 | MIT | <https://crates.io/crates/winreg> |
| wit-bindgen | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | <https://crates.io/crates/wit-bindgen> |
| writeable | 0.6.4 | Unicode-3.0 | <https://crates.io/crates/writeable> |
| wry | 0.57.1 | Apache-2.0 OR MIT | <https://crates.io/crates/wry> |
| x11 | 2.21.0 | MIT | <https://crates.io/crates/x11> |
| x11-dl | 2.21.0 | MIT | <https://crates.io/crates/x11-dl> |
| x11rb | 0.13.2 | MIT OR Apache-2.0 | <https://crates.io/crates/x11rb> |
| x11rb-protocol | 0.13.2 | MIT OR Apache-2.0 | <https://crates.io/crates/x11rb-protocol> |
| xattr | 1.7.0 | MIT OR Apache-2.0 | <https://crates.io/crates/xattr> |
| xkeysym | 0.2.1 | MIT OR Apache-2.0 OR Zlib | <https://crates.io/crates/xkeysym> |
| xxhash-rust | 0.8.19 | BSL-1.0 | <https://crates.io/crates/xxhash-rust> |
| yansi | 1.0.1 | MIT OR Apache-2.0 | <https://crates.io/crates/yansi> |
| yoke | 0.8.4 | Unicode-3.0 | <https://crates.io/crates/yoke> |
| yoke-derive | 0.8.4 | Unicode-3.0 | <https://crates.io/crates/yoke-derive> |
| zbus | 5.19.0 | MIT | <https://crates.io/crates/zbus> |
| zbus_macros | 5.19.0 | MIT | <https://crates.io/crates/zbus_macros> |
| zbus_names | 4.3.5 | MIT | <https://crates.io/crates/zbus_names> |
| zcheapstr | 1.1.0 | MIT | <https://crates.io/crates/zcheapstr> |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | <https://crates.io/crates/zerocopy> |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | <https://crates.io/crates/zerocopy-derive> |
| zerofrom | 0.1.8 | Unicode-3.0 | <https://crates.io/crates/zerofrom> |
| zerofrom-derive | 0.1.8 | Unicode-3.0 | <https://crates.io/crates/zerofrom-derive> |
| zeroize | 1.9.1 | Apache-2.0 OR MIT | <https://crates.io/crates/zeroize> |
| zerotrie | 0.2.5 | Unicode-3.0 | <https://crates.io/crates/zerotrie> |
| zerovec | 0.11.9 | Unicode-3.0 | <https://crates.io/crates/zerovec> |
| zerovec-derive | 0.11.7 | Unicode-3.0 | <https://crates.io/crates/zerovec-derive> |
| zip | 9.1.3 | MIT | <https://crates.io/crates/zip> |
| zlib-rs | 0.6.8 | Zlib | <https://crates.io/crates/zlib-rs> |
| zmij | 1.0.23 | MIT | <https://crates.io/crates/zmij> |
| zvariant | 5.15.1 | MIT | <https://crates.io/crates/zvariant> |
| zvariant_derive | 5.15.1 | MIT | <https://crates.io/crates/zvariant_derive> |
| zvariant_utils | 4.2.0 | MIT | <https://crates.io/crates/zvariant_utils> |

<!-- END AUTO-GENERATED DEPENDENCIES -->
