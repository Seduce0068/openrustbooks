    use super::{
        ERR_ENTRY_READ, ERR_INVALID_ARCHIVE, ERR_INVALID_UTF8, WasmArchive, WasmArchiveEntry,
        WasmMarkdownLinter,
    };
    use crate::archive::{ArchiveLimits, ArchiveReader};
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

    fn linter() -> WasmMarkdownLinter {
        // null は LintConfig としてパースできないため既定設定へフォールバックする。
        WasmMarkdownLinter::new(JsValue::NULL).expect("linter should build")
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

    #[wasm_bindgen_test]
    fn incomplete_config_is_rejected() {
        // 部分オブジェクトを黙って既定へ落とすと auto_fix:false が反転するためエラーにする。
        let partial = serde_wasm_bindgen::to_value(&serde_json::json!({ "auto_fix": false }))
            .expect("config should serialize");
        match WasmMarkdownLinter::new(partial) {
            Ok(_) => panic!("incomplete config should be rejected"),
            Err(error) => assert_eq!(
                error.as_string().as_deref().unwrap_or("").split(':').next(),
                Some("Invalid lint config")
            ),
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
