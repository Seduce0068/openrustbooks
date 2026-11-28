    use super::{
        ERR_ENTRIES, ERR_ENTRY_READ, ERR_INVALID_ARCHIVE, ERR_INVALID_UTF8, WasmArchive,
        WasmArchiveEntry, WasmEpubBook, WasmLintFixResult, WasmMarkdownLinter,
    };
    use crate::ai_markdown::lint::{validate_with_auto_fix, LintResultWrapper, MarkdownLinter};
    use crate::archive::{ArchiveLimits, ArchiveReader};
    use crate::config::LintConfig;
    use std::io::{Cursor, Write};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::wasm_bindgen_test;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipArchive, ZipWriter};

    /// `Result<T, JsValue>` が期待どおりのエラー文字列を返すことを検証する。
    /// `unwrap_err()` は `T: Debug` を要求するため `match` で回避する。
    fn assert_js_error<T>(result: Result<T, JsValue>, expected: &str) {
        match result {
            Ok(_) => panic!("expected an error: {expected}"),
            Err(error) => assert_eq!(error.as_string().as_deref(), Some(expected)),
        }
    }

    /// 固定文言への完全一致に加え、機密マーカーが一切含まれないことを検証する。
    fn assert_redacted_js_error<T>(result: Result<T, JsValue>, expected: &str, hidden: &[&str]) {
        let message = match result {
            Ok(_) => panic!("expected an error: {expected}"),
            Err(error) => error.as_string().expect("fixed error is a string"),
        };
        assert_eq!(message, expected);
        for needle in hidden {
            assert!(
                !message.contains(needle),
                "error message leaked {needle:?}: {message:?}"
            );
        }
    }

    fn linter() -> WasmMarkdownLinter {
        // null は LintConfig としてパースできないため既定設定へフォールバックする。
        WasmMarkdownLinter::new(JsValue::NULL).expect("linter should build")
    }

    /// `conflicting_rules()` getter の JS 配列を `Vec<String>` へ戻す。
    fn conflicting_rules(result: &WasmLintFixResult) -> Vec<String> {
        serde_wasm_bindgen::from_value(
            result
                .conflicting_rules()
                .expect("conflicting_rules should serialize"),
        )
        .expect("conflicting_rules should deserialize")
    }

    /// CoreError::validation に secret / path を入れても js_error へ漏れないこと。
    #[wasm_bindgen_test]
    fn js_error_masks_validation_secret_and_path() {
        let secret = "sk-wasm-validation-leak";
        let path = "/home/user/secret/book.epub";
        let error = crate::error::CoreError::validation(format!("invalid {path} with {secret}"));

        let rendered = super::js_error(error)
            .as_string()
            .expect("fixed boundary errors are strings");
        assert_eq!(rendered, "Configuration validation failed");
        assert!(!rendered.contains(secret));
        assert!(!rendered.contains(path));
    }

    /// `lint()` が返した JS 配列を rule ID のリストへ変換する。
    fn rule_names(value: JsValue) -> Vec<String> {
        let issues: Vec<super::WasmLintIssue> =
            serde_wasm_bindgen::from_value(value).expect("issues should deserialize");
        issues.into_iter().map(|issue| issue.rule).collect()
    }

    #[wasm_bindgen_test]
    fn lint_returns_no_issues_for_clean_markdown() {
        let rules = rule_names(
            linter()
                .lint("# Title\n\nClean paragraph.\n".into())
                .expect("lint should succeed"),
        );
        assert!(
            rules.is_empty(),
            "clean markdown should have no issues: {rules:?}"
        );
    }

    #[wasm_bindgen_test]
    fn lint_detects_inline_html() {
        let rules = rule_names(
            linter()
                .lint("# Title\n\n<div>html</div>\n".into())
                .expect("lint should succeed"),
        );
        assert!(
            !rules.is_empty(),
            "inline HTML should produce at least one issue"
        );
        assert!(
            rules.iter().any(|rule| rule == "MD033"),
            "MD033 should be detected: {rules:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn lint_and_fix_returns_passed_and_fixed_content() {
        let dirty = "# Title\n\nTrailing spaces here   \n";
        let result = linter()
            .lint_and_fix(dirty.into())
            .await
            .expect("lint_and_fix should succeed");

        assert!(result.passed(), "fixed markdown should pass lint");
        assert_ne!(result.fixed_content(), dirty, "content should be fixed");
        assert!(
            result.fixes_applied() > 0,
            "at least one fix should be applied"
        );
    }

    #[wasm_bindgen_test]
    async fn auto_fix_false_keeps_content_unchanged() {
        // auto_fix の権威は native LintConfig（ADR-0160/0194）。constructor に渡した値が使われる。
        let config = crate::config::LintConfig {
            auto_fix: false,
            ..Default::default()
        };
        let js_config = serde_wasm_bindgen::to_value(&config).expect("config should serialize");
        let linter = WasmMarkdownLinter::new(js_config).expect("linter should build");

        let dirty = "# Title\n\nTrailing spaces here   \n";
        let result = linter
            .lint_and_fix(dirty.into())
            .await
            .expect("lint_and_fix should succeed");

        assert_eq!(
            result.fixed_content(),
            dirty,
            "auto_fix=false なら本文を変更しないこと"
        );
        assert_eq!(result.fixes_applied(), 0, "auto_fix=false なら修正数は 0");
        let rules = rule_names(result.issues().expect("issues should serialize"));
        assert!(
            rules.iter().any(|rule| rule == "MD009"),
            "lint のみの経路でも問題は報告されること: {rules:?}"
        );
    }

    /// `auto_fix=true` の Wasm DTO が native `LintResultWrapper` と一致すること。
    #[wasm_bindgen_test]
    async fn convergence_matches_native_with_auto_fix() {
        let config = LintConfig::default();
        assert!(config.auto_fix, "default config should enable auto_fix");
        let wasm = WasmMarkdownLinter::new(
            serde_wasm_bindgen::to_value(&config).expect("config should serialize"),
        )
        .expect("wasm linter should build");
        let native = MarkdownLinter::new(&config).expect("native linter should build");

        let content = "# Title\n\nTrailing spaces here   \n";
        let dto = wasm
            .lint_and_fix(content.into())
            .await
            .expect("lint_and_fix should succeed");
        let expected = validate_with_auto_fix(&native, content, config.auto_fix)
            .await
            .expect("native validation should succeed");

        assert_eq!(dto.converged(), expected.converged);
        assert_eq!(conflicting_rules(&dto), expected.conflicting_rules);
        assert_eq!(dto.passed(), expected.passed);
        assert_eq!(dto.fixed_content(), expected.fixed_content);
        assert_eq!(dto.fixes_applied(), expected.fixes_applied);
        // 通常の修正経路では収束し、競合ルールは空になる。
        assert!(dto.converged());
        assert!(conflicting_rules(&dto).is_empty());
    }

    /// `auto_fix=false` の lint のみ経路でも native と一致すること。
    #[wasm_bindgen_test]
    async fn convergence_matches_native_without_auto_fix() {
        let config = LintConfig {
            auto_fix: false,
            ..Default::default()
        };
        let wasm = WasmMarkdownLinter::new(
            serde_wasm_bindgen::to_value(&config).expect("config should serialize"),
        )
        .expect("wasm linter should build");
        let native = MarkdownLinter::new(&config).expect("native linter should build");

        let content = "# Title\n\nTrailing spaces here   \n";
        let dto = wasm
            .lint_and_fix(content.into())
            .await
            .expect("lint_and_fix should succeed");
        let expected = validate_with_auto_fix(&native, content, config.auto_fix)
            .await
            .expect("native validation should succeed");

        assert_eq!(dto.converged(), expected.converged);
        assert_eq!(conflicting_rules(&dto), expected.conflicting_rules);
        assert_eq!(dto.passed(), expected.passed);
        assert_eq!(dto.fixed_content(), expected.fixed_content);
        assert_eq!(dto.fixes_applied(), expected.fixes_applied);
        // lint のみの経路は修正しないため収束扱い（native の契約）。
        assert!(dto.converged());
        assert!(conflicting_rules(&dto).is_empty());
    }

    /// 未収束・競合ルール非空の値も DTO へそのまま伝わること。
    /// rumdl の入力からは安定して再現できないため native 構造体を直接変換して検証する。
    #[wasm_bindgen_test]
    fn non_converged_native_result_is_exposed_without_mutation() {
        let original = "# Title\n\nTrailing spaces here   \n";
        let native = LintResultWrapper {
            passed: false,
            issues: vec![],
            fixed_content: original.to_string(),
            fixes_applied: 0,
            converged: false,
            conflicting_rules: vec!["MD009".to_string(), "MD010".to_string()],
        };
        let dto = WasmLintFixResult::from(native);

        assert!(!dto.converged(), "未収束はそのまま公開されること");
        assert_eq!(
            conflicting_rules(&dto),
            vec!["MD009".to_string(), "MD010".to_string()]
        );
        assert!(!dto.passed(), "未収束なら fail-closed で不合格のこと");
        assert_eq!(dto.fixed_content(), original);
    }

    #[wasm_bindgen_test]
    fn incomplete_config_is_rejected() {
        // 部分オブジェクトを黙って既定へ落とすと auto_fix:false が反転するためエラーにする。
        let partial = serde_wasm_bindgen::to_value(&serde_json::json!({ "auto_fix": false }))
            .expect("config should serialize");
        match WasmMarkdownLinter::new(partial) {
            Ok(_) => panic!("incomplete config should be rejected"),
            Err(error) => {
                let message = error.as_string().expect("fixed error is a string");
                assert_eq!(message, super::INVALID_LINT_CONFIG_MESSAGE);
                // serde の型名・フィールド名・生エラー本文が漏れないこと。
                for leaked in ["serde", "LintConfig", "auto_fix", "missing field"] {
                    assert!(
                        !message.contains(leaked),
                        "lint config error leaked {leaked:?}: {message:?}"
                    );
                }
            }
        }
    }

    #[wasm_bindgen_test]
    fn image_dimensions_wrapper() {
        // 1x1 PNG の最小ヘッダー。
        let mut png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
        png.extend_from_slice(&1u32.to_be_bytes());
        png.extend_from_slice(&1u32.to_be_bytes());
        assert_eq!(super::image_dimensions_wasm(&png), Some(vec![1, 1]));
        assert_eq!(super::image_dimensions_wasm(b"not an image"), None);
    }

    /// 有効 EPUB の `container.xml`（OPF は `OEBPS/content.opf`）。
    const EPUB_CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

    /// 有効 EPUB の最小 OPF（章 1 章のみ）。
    const EPUB_OPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Sample</dc:title>
    <dc:language>ja</dc:language>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>
"#;

    /// 有効 EPUB の章本文。
    const EPUB_CH1: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>Chapter</title></head><body><p>ok</p></body></html>"#;

    /// `mimetype` を先頭に `Stored` で書き、有効な EPUB を組み立てる。
    fn build_valid_epub() -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .expect("mimetype should start");
        writer
            .write_all(b"application/epub+zip")
            .expect("mimetype should write");
        for (path, data) in [
            ("META-INF/container.xml", EPUB_CONTAINER.as_bytes()),
            ("OEBPS/content.opf", EPUB_OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", EPUB_CH1.as_bytes()),
        ] {
            writer
                .start_file(
                    path,
                    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
                )
                .expect("entry should start");
            writer.write_all(data).expect("entry should write");
        }
        writer.finish().expect("zip should finish").into_inner()
    }

    /// 有効 EPUB に対して不正 href を渡しても、JS 境界へは固定文言だけが出る。
    #[wasm_bindgen_test]
    fn epub_invalid_href_errors_are_fixed_without_href_or_type_names() {
        let book = WasmEpubBook::new(build_valid_epub()).expect("valid epub should open");
        // 不正な `%XX` は chapter_resource / resolve_chapter_link の両方で
        // CoreError::Validation（CONFIG_VALIDATION）になる。
        let href = "%ZZ-SECRET_HREF.xhtml";
        let hidden = [
            "SECRET_HREF",
            "javascript:",
            "file:",
            "CoreError",
            "Validation",
        ];
        assert_redacted_js_error(
            book.chapter_resource(0, href.to_string()),
            "Configuration validation failed",
            &hidden,
        );
        assert_redacted_js_error(
            book.resolve_chapter_link(0, href.to_string()),
            "Configuration validation failed",
            &hidden,
        );
    }

    /// テスト用 ZIP のエントリ。
    enum Item<'a> {
        File(&'a str, &'a [u8], CompressionMethod),
        Dir(&'a str),
    }

    /// 渡したエントリを順に書き込んだ ZIP バイト列を組み立てる。
    fn build_zip(items: &[Item<'_>]) -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for item in items {
            match item {
                Item::File(name, data, method) => {
                    let options = SimpleFileOptions::default().compression_method(*method);
                    writer
                        .start_file(*name, options)
                        .expect("start_file should succeed");
                    writer.write_all(data).expect("write_all should succeed");
                }
                Item::Dir(name) => writer
                    .add_directory(*name, SimpleFileOptions::default())
                    .expect("add_directory should succeed"),
            }
        }
        writer.finish().expect("finish should succeed").into_inner()
    }

    #[wasm_bindgen_test]
    fn wasm_archive_lists_entries_and_reads_content() {
        let bytes = build_zip(&[
            Item::Dir("docs/"),
            Item::File("docs/readme.txt", b"hello", CompressionMethod::Stored),
            Item::File(
                "docs/body.txt",
                b"world world world",
                CompressionMethod::Deflated,
            ),
        ]);
        // 元バイト列は方式・圧縮サイズの直接検証にも使うため複製して渡す。
        let archive = WasmArchive::new(bytes.clone()).expect("valid zip should open");

        let entries: Vec<WasmArchiveEntry> =
            serde_wasm_bindgen::from_value(archive.entries().expect("entries should serialize"))
                .expect("entries should deserialize");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].path, "docs");
        assert!(entries[0].is_dir);
        assert_eq!(entries[1].path, "docs/readme.txt");
        assert!(!entries[1].is_dir);
        assert_eq!(entries[1].size, 5);
        assert_eq!(entries[1].compressed_size, 5);
        assert_eq!(entries[2].path, "docs/body.txt");
        assert_eq!(entries[2].size, 17);

        // 圧縮率ではなく、ZIP の方式と圧縮サイズそのものを検証する。
        let mut zip = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        let file = zip.by_index(2).unwrap();
        assert_eq!(file.compression(), CompressionMethod::Deflated);
        assert_eq!(entries[2].compressed_size, file.compressed_size());

        assert_eq!(
            archive
                .read_entry("docs/readme.txt")
                .expect("read should succeed"),
            b"hello"
        );
        assert_eq!(
            archive
                .read_entry_text("./docs//body.txt")
                .expect("read text should succeed"),
            "world world world"
        );
        assert!(!archive.is_epub());
    }

    #[wasm_bindgen_test]
    fn wasm_archive_detects_epub() {
        let bytes = build_zip(&[
            Item::File(
                "mimetype",
                b"application/epub+zip\n",
                CompressionMethod::Stored,
            ),
            Item::File(
                "META-INF/container.xml",
                b"<container/>",
                CompressionMethod::Stored,
            ),
        ]);
        let archive = WasmArchive::new(bytes).expect("valid epub should open");
        assert!(archive.is_epub());
        assert_eq!(
            archive
                .read_entry_text("mimetype")
                .expect("mimetype should read"),
            "application/epub+zip\n"
        );
    }

    #[wasm_bindgen_test]
    fn wasm_archive_rejects_path_traversal() {
        let bytes = build_zip(&[Item::File("../evil.txt", b"x", CompressionMethod::Stored)]);
        assert_js_error(WasmArchive::new(bytes), ERR_INVALID_ARCHIVE);

        // 正常なアーカイブでも、照合パスの逸脱は読み出し時に拒否する。
        let bytes = build_zip(&[Item::File("ok.txt", b"x", CompressionMethod::Stored)]);
        let archive = WasmArchive::new(bytes).expect("valid zip should open");
        assert_js_error(archive.read_entry("../evil.txt"), ERR_ENTRY_READ);
    }

    #[wasm_bindgen_test]
    fn wasm_archive_rejects_duplicate_entries_after_normalization() {
        // zip 9.1.3 は同じ raw 名を 2 回書けないため、`a.txt` と `./a.txt`
        // （正規化後は両方 `a.txt`）で重複を作る。
        let bytes = build_zip(&[
            Item::File("a.txt", b"x", CompressionMethod::Stored),
            Item::File("./a.txt", b"y", CompressionMethod::Stored),
        ]);
        assert_js_error(WasmArchive::new(bytes), ERR_INVALID_ARCHIVE);
    }

    #[wasm_bindgen_test]
    fn wasm_archive_rejects_invalid_bytes() {
        assert_js_error(
            WasmArchive::new(b"not a zip archive".to_vec()),
            ERR_INVALID_ARCHIVE,
        );
        assert_js_error(WasmArchive::new(Vec::new()), ERR_INVALID_ARCHIVE);

        let valid = build_zip(&[Item::File("a.txt", b"a", CompressionMethod::Stored)]);
        assert_js_error(
            WasmArchive::new(valid[..valid.len() / 2].to_vec()),
            ERR_INVALID_ARCHIVE,
        );
    }

    #[wasm_bindgen_test]
    fn wasm_archive_read_errors_use_fixed_messages() {
        let bytes = build_zip(&[Item::File(
            "bad.txt",
            &[0xFF, 0xFE, 0xFD],
            CompressionMethod::Stored,
        )]);
        let archive = WasmArchive::new(bytes).expect("valid zip should open");

        assert_js_error(archive.read_entry("missing.txt"), ERR_ENTRY_READ);
        assert_js_error(archive.read_entry_text("missing.txt"), ERR_ENTRY_READ);
        assert_js_error(archive.read_entry_text("bad.txt"), ERR_INVALID_UTF8);
    }

    /// ADR-0223 の 4 つ目の固定文言（`entries()` のシリアライズ失敗）も
    /// 内部詳細を漏らさない固定文字列であることを検証する。
    /// 通常の入力からはシリアライズ失敗を誘発できないため定数自体を検証する。
    #[wasm_bindgen_test]
    fn wasm_archive_entries_error_message_is_fixed() {
        assert_eq!(ERR_ENTRIES, "ZIP entries unavailable");
        for marker in ["SECRET", "CoreError", "Validation", "ZipError"] {
            assert!(
                !ERR_ENTRIES.contains(marker),
                "ERR_ENTRIES leaked {marker:?}: {ERR_ENTRIES:?}"
            );
        }
    }

    /// エラー文言にパス・本文・内部型名が混ざらないことを明示的に検証する。
    #[wasm_bindgen_test]
    fn wasm_archive_errors_never_leak_details() {
        const MARKERS: [&str; 6] = [
            "SECRET_TRAVERSAL",
            "SECRET_MISSING",
            "SECRET_BODY",
            "CoreError",
            "Validation",
            "ZipError",
        ];

        let traversal = build_zip(&[Item::File(
            "../SECRET_TRAVERSAL.txt",
            b"SECRET_BODY",
            CompressionMethod::Stored,
        )]);
        let mut errors: Vec<JsValue> = Vec::new();
        if let Err(error) = WasmArchive::new(traversal) {
            errors.push(error);
        }
        if let Err(error) = WasmArchive::new(b"SECRET_BODY-not-a-zip".to_vec()) {
            errors.push(error);
        }

        let valid = build_zip(&[Item::File(
            "ok.txt",
            b"SECRET_BODY\xff\xfe",
            CompressionMethod::Stored,
        )]);
        let archive = WasmArchive::new(valid).expect("valid zip should open");
        if let Err(error) = archive.read_entry("SECRET_MISSING.txt") {
            errors.push(error);
        }
        if let Err(error) = archive.read_entry_text("SECRET_MISSING.txt") {
            errors.push(error);
        }
        if let Err(error) = archive.read_entry_text("ok.txt") {
            errors.push(error);
        }

        assert!(!errors.is_empty(), "at least one error should be produced");
        for error in errors {
            let message = error.as_string().expect("fixed errors are strings");
            for marker in MARKERS {
                assert!(
                    !message.contains(marker),
                    "error message leaked {marker:?}: {message:?}"
                );
            }
        }
    }

    #[wasm_bindgen_test]
    fn archive_limits_are_enforced_on_wasm() {
        let bytes = build_zip(&[
            Item::File("a.txt", b"aaaa", CompressionMethod::Stored),
            Item::File("b.txt", b"bbbb", CompressionMethod::Deflated),
        ]);

        // Wasm 側の new は既定 limits を使うため、小さな limits での拒否は
        // 権威である ArchiveReader 側の既存カバレッジを wasm 上でも確認する。
        assert!(
            ArchiveReader::with_limits(
                &bytes,
                ArchiveLimits {
                    max_entries: 1,
                    ..ArchiveLimits::default()
                },
            )
            .is_err()
        );
        assert!(
            ArchiveReader::with_limits(
                &bytes,
                ArchiveLimits {
                    max_entry_size: 3,
                    ..ArchiveLimits::default()
                },
            )
            .is_err()
        );
        assert!(
            ArchiveReader::with_limits(
                &bytes,
                ArchiveLimits {
                    max_total_size: 7,
                    ..ArchiveLimits::default()
                },
            )
            .is_err()
        );

        // 既定 limits では Wasm の new が成功する。
        assert!(WasmArchive::new(bytes).is_ok());
    }
