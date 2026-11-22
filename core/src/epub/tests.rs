    use super::*;
    use std::io::{Cursor, Write};
    use zip::CompressionMethod;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    /// テストで使う `META-INF/container.xml`（OPF は `OEBPS/content.opf`）。
    const CONTAINER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

    /// テストで使う OPF。`linear="no"` の章と EPUB2 形式の `meta` を含む。
    const OPF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:creator>Alice</dc:creator>
    <dc:creator>Bob</dc:creator>
    <dc:language>ja</dc:language>
    <dc:identifier id="other">urn:uuid:0000</dc:identifier>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <dc:publisher>Example Press</dc:publisher>
    <dc:date>2024-05-01</dc:date>
    <meta property="dcterms:modified">2024-05-02T10:00:00Z</meta>
    <meta name="cover" content="cover-image"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2" linear="no"/>
  </spine>
</package>
"#;

    /// 接頭辞付きの OPF。要素名のローカル名だけで解析できることを確認する。
    const OPF_PREFIXED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf"
             xmlns:dc="http://purl.org/dc/elements/1.1/"
             version="3.0" unique-identifier="pub-id">
  <opf:metadata>
    <dc:title>サンプル本</dc:title>
    <dc:creator>Alice</dc:creator>
    <dc:creator>Bob</dc:creator>
    <dc:language>ja</dc:language>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <dc:publisher>Example Press</dc:publisher>
    <dc:date>2024-05-01</dc:date>
    <opf:meta property="dcterms:modified">2024-05-02T10:00:00Z</opf:meta>
  </opf:metadata>
  <opf:manifest>
    <opf:item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <opf:item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="ch1"/>
    <opf:itemref idref="ch2" linear="no"/>
  </opf:spine>
</opf:package>
"#;

    /// 本文抽出を検証する章。`head` の `title` と `script`・`style` を含む。
    const CH1: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <title>Head Title</title>
  <style>p { color: red; }</style>
</head>
<body>
  <script>var marker = "script-marker";</script>
  <h1>T</h1>
  <p>a &amp; b</p>
  <p>c &lt;d&gt; &#x30; &foo;</p>
</body>
</html>"#;

    /// 2 章目の内容（`chapter_bytes` の検証に使う）。
    const CH2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>第二章</p></body></html>"#;

    /// 章本文の期待値。
    const CH1_TEXT: &str = "T\na & b\nc <d> 0";

    /// 入れ子の `<ol>` / `<li>` / `<a>` を持つ EPUB3 の nav。
    const NAV_NESTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>目次</title></head>
<body>
  <nav epub:type="toc">
    <h1>目次</h1>
    <ol>
      <li><a href="ch1.xhtml">第一章 &amp; はじめに</a></li>
      <li><span>第二部</span>
        <ol>
          <li><a href="ch2.xhtml#s1">第二章 1 節</a></li>
          <li><span>見出しだけ</span></li>
        </ol>
      </li>
    </ol>
  </nav>
</body>
</html>
"#;

    /// `<a>` を持たない `<li>` を含む nav。
    const NAV_SPAN_ONLY: &str = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="toc">
    <ol>
      <li><span>見出しだけ</span></li>
      <li><a href="ch1.xhtml">実項目</a></li>
    </ol>
  </nav>
</body></html>
"#;

    /// `landmarks` が先、`toc` が後ろにある nav。
    const NAV_LANDMARKS_FIRST: &str = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="landmarks">
    <ol><li><a href="ch1.xhtml">表紙</a></li></ol>
  </nav>
  <nav epub:type="toc">
    <ol><li><a href="ch2.xhtml">目次の項目</a></li></ol>
  </nav>
</body></html>
"#;

    /// `epub:type` を持たない `<nav>` だけの nav。
    const NAV_WITHOUT_TYPE: &str = r#"<html><body>
  <nav><ol><li><a href="ch1.xhtml">唯一の nav</a></li></ol></nav>
</body></html>
"#;

    /// フラグメント付きと無しの `<a>` を混ぜた入れ子の nav。
    const NAV_FRAGMENT_MIXED: &str = r##"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="toc">
    <ol>
      <li><a href="ch1.xhtml#s1">第一章</a>
        <ol><li><a href="ch2.xhtml">第二章</a></li></ol>
      </li>
    </ol>
  </nav>
</body></html>
"##;

    /// パス部が空の `<a>` を挟む入れ子の nav。
    const NAV_WITH_EMPTY_PATHS: &str = r##"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
  <nav epub:type="toc">
    <ol>
      <li><a href="#only-fragment">位置だけ</a>
        <ol><li><a href="ch1.xhtml">第一章</a></li></ol>
      </li>
      <li><a href="  ">空白だけ</a></li>
      <li><a href="ch2.xhtml#s1">第二章</a></li>
    </ol>
  </nav>
</body></html>
"##;

    /// 入れ子の `navPoint` を持つ EPUB2 の NCX。
    ///
    /// `playOrder` は文書順と逆にしてあり、内容を持たない `navPoint` も含む。
    const NCX_NESTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head><meta name="dtb:uid" content="urn:isbn:9784000000000"/></head>
  <docTitle><text>サンプル本</text></docTitle>
  <navMap>
    <navPoint id="n1" playOrder="3">
      <navLabel><text>第一章</text></navLabel>
      <content src="ch1.xhtml"/>
      <navPoint id="n1-1" playOrder="2">
        <navLabel><text>第一節</text></navLabel>
        <content src="ch2.xhtml#s1"/>
      </navPoint>
    </navPoint>
    <navPoint id="n2" playOrder="1">
      <navLabel><text>第二章</text></navLabel>
      <content src="ch2.xhtml"/>
    </navPoint>
    <navPoint id="n3" playOrder="4">
      <navLabel><text>内容の無い項目</text></navLabel>
    </navPoint>
  </navMap>
</ncx>
"#;

    /// NCX だけを情報源に持つ EPUB2 相当の OPF。
    const OPF_NCX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// エントリを順に書き込んだ ZIP を組み立てる。
    fn write_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (path, data) in files {
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer.start_file(*path, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// `mimetype` を先頭に `Stored` で書いた EPUB を組み立てる。
    fn build_epub(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"application/epub+zip").unwrap();
        for (path, data) in files {
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer.start_file(*path, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// `mimetype` が重複する EPUB を組み立てる。
    ///
    /// `zip` 9.1.3 は同じ raw 名を 2 回書けないため、2 回目は `./mimetype`
    /// として書き込む。`normalize_path` により両方 `mimetype` へ正規化される。
    fn build_epub_with_duplicate_mimetype() -> Vec<u8> {
        let mut files: Vec<(&str, &[u8])> = vec![("./mimetype", b"application/epub+zip")];
        files.extend_from_slice(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ]);
        build_epub(&files)
    }

    /// `container.xml` を差し替えた EPUB を組み立てる。
    fn epub_with_container(container: &str) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ])
    }

    /// OPF を差し替えた EPUB を組み立てる。
    fn epub_with_opf(opf: &str) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ])
    }

    /// 標準の EPUB3 相当（`OEBPS/content.opf` と 2 章）。
    fn sample_epub() -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
            (
                "OEBPS/nav.xhtml",
                b"<html><body><nav>nav</nav></body></html>",
            ),
        ])
    }

    /// `open` のエラーを取り出す。
    fn open_error(bytes: &[u8]) -> CoreError {
        match EpubBook::open(bytes) {
            Err(err) => err,
            Ok(_) => panic!("エラーを期待したが開けた"),
        }
    }

    /// 固定文言の Archive エラーを検証する（値・ファイル名が漏れないこと）。
    fn assert_fixed_archive_error(err: CoreError, expected: &str) {
        match err {
            CoreError::Archive(message) => {
                assert_eq!(message, expected);
                // ADR-0212 の redaction 契約: id 値・href 値・ファイル名を出さない。
                for leaked in ["ch1", "ch1.xhtml", "content.opf", "OEBPS"] {
                    assert!(
                        !message.contains(leaked),
                        "エラー文言に {leaked} が含まれています: {message}"
                    );
                }
            }
            other => panic!("CoreError::Archive を期待したが {other:?}"),
        }
    }

    /// 目次の XHTML（nav / NCX）と OPF を差し替えた EPUB を組み立てる。
    fn epub_with_toc(nav: Option<&str>, ncx: Option<&str>, opf: &str) -> Vec<u8> {
        let mut files: Vec<(&str, &[u8])> = vec![
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ];
        if let Some(nav) = nav {
            files.push(("OEBPS/nav.xhtml", nav.as_bytes()));
        }
        if let Some(ncx) = ncx {
            files.push(("OEBPS/toc.ncx", ncx.as_bytes()));
        }
        build_epub(&files)
    }

    /// nav の XHTML を差し替えた EPUB（OPF は標準）を組み立てる。
    fn epub_with_nav(nav: &str) -> Vec<u8> {
        epub_with_toc(Some(nav), None, OPF)
    }

    /// `properties` を差し替えた OPF を返す。
    fn opf_with_nav_properties(properties: &str) -> String {
        OPF.replace(
            "properties=\"nav\"",
            &format!("properties=\"{properties}\""),
        )
    }

    /// nav と NCX の両方を manifest に持つ OPF を返す。
    fn opf_with_nav_and_ncx() -> String {
        OPF.replace(
            "<manifest>\n",
            "<manifest>\n    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n",
        )
    }

    /// `<a>` を `count` 個持つ nav を組み立てる。
    fn nav_with_anchors(count: usize) -> String {
        let mut nav = String::from("<html><body><nav epub:type=\"toc\"><ol>");
        for _ in 0..count {
            nav.push_str("<li><a href=\"ch1.xhtml\">項目</a></li>");
        }
        nav.push_str("</ol></nav></body></html>");
        nav
    }

    /// `href` の並びから nav を組み立てる。`None` の要素は `href` 属性なし。
    fn nav_with_hrefs(hrefs: &[Option<&str>]) -> String {
        let mut nav = String::from("<html><body><nav epub:type=\"toc\"><ol>");
        for href in hrefs {
            match href {
                Some(value) => nav.push_str(&format!("<li><a href=\"{value}\">項目</a></li>")),
                None => nav.push_str("<li><a>項目</a></li>"),
            }
        }
        nav.push_str("</ol></nav></body></html>");
        nav
    }

    /// `src` の並びから `<navMap>` を組み立てた NCX を返す。
    fn ncx_with_sources(sources: &[&str]) -> String {
        let mut ncx = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\"><navMap>",
        );
        for (index, src) in sources.iter().enumerate() {
            ncx.push_str(&format!(
                "<navPoint id=\"n{index}\"><navLabel><text>項目{index}</text></navLabel>\
                 <content src=\"{src}\"/></navPoint>"
            ));
        }
        ncx.push_str("</navMap></ncx>");
        ncx
    }

    /// 期待する目次エントリを組み立てる（フラグメント無し）。
    fn entry(title: &str, href: &str, depth: usize) -> EpubTocEntry {
        EpubTocEntry {
            title: title.to_string(),
            href: href.to_string(),
            fragment: None,
            depth,
        }
    }

    /// フラグメント付きの期待する目次エントリを組み立てる。
    fn entry_with_fragment(title: &str, href: &str, fragment: &str, depth: usize) -> EpubTocEntry {
        EpubTocEntry {
            title: title.to_string(),
            href: href.to_string(),
            fragment: Some(fragment.to_string()),
            depth,
        }
    }

    /// `toc()` のエラーを取り出す。
    fn toc_error(bytes: &[u8]) -> CoreError {
        match EpubBook::open(bytes).and_then(|book| book.toc()) {
            Err(err) => err,
            Ok(entries) => panic!("エラーを期待したが {} 件返った", entries.len()),
        }
    }

    #[test]
    fn open_owned_matches_open() {
        let bytes = sample_epub();
        let borrowed = EpubBook::open(&bytes).unwrap();
        let owned = EpubBook::open_owned(bytes).unwrap();
        assert_eq!(owned.metadata(), borrowed.metadata());
        assert_eq!(owned.chapters(), borrowed.chapters());
        assert_eq!(
            owned.chapter_html(0).unwrap(),
            borrowed.chapter_html(0).unwrap()
        );
    }

    #[test]
    fn open_owned_with_limits_enforces_limits() {
        let bytes = sample_epub();
        let limits = ArchiveLimits {
            max_entries: 1,
            ..ArchiveLimits::default()
        };

        // 所有経路でもリミットが効き、上限超過は Validation になる。
        assert!(matches!(
            EpubBook::open_owned_with_limits(bytes, limits),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn open_owned_with_default_limits_matches_open_owned() {
        let bytes = sample_epub();
        let expected = EpubBook::open_owned(bytes.clone()).unwrap();
        let actual = EpubBook::open_owned_with_limits(bytes, ArchiveLimits::default()).unwrap();

        // 既定リミットでは旧 API と同じメタデータ・章になる。
        assert_eq!(actual.metadata(), expected.metadata());
        assert_eq!(actual.chapters(), expected.chapters());
        assert_eq!(
            actual.chapter_html(0).unwrap(),
            expected.chapter_html(0).unwrap()
        );
    }

    #[test]
    fn reads_epub3_metadata() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();
        let metadata = book.metadata();

        assert_eq!(metadata.title.as_deref(), Some("サンプル本"));
        assert_eq!(
            metadata.creators,
            vec!["Alice".to_string(), "Bob".to_string()]
        );
        assert_eq!(metadata.language.as_deref(), Some("ja"));
        // `unique-identifier` と一致する `id` を優先する。
        assert_eq!(
            metadata.identifier.as_deref(),
            Some("urn:isbn:9784000000000")
        );
        assert_eq!(metadata.publisher.as_deref(), Some("Example Press"));
        assert_eq!(metadata.date.as_deref(), Some("2024-05-01"));
        assert_eq!(metadata.modified.as_deref(), Some("2024-05-02T10:00:00Z"));
        assert_eq!(metadata.version.as_deref(), Some("3.0"));
    }

    #[test]
    fn lists_spine_chapters_in_order() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();
        let chapters = book.chapters();

        assert_eq!(chapters.len(), 2);

        assert_eq!(chapters[0].id, "ch1");
        assert_eq!(chapters[0].href, "OEBPS/ch1.xhtml");
        assert_eq!(chapters[0].media_type, "application/xhtml+xml");
        assert!(chapters[0].linear);

        assert_eq!(chapters[1].id, "ch2");
        assert_eq!(chapters[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(chapters[1].media_type, "application/xhtml+xml");
        assert!(!chapters[1].linear);
    }

    #[test]
    fn extracts_normalized_chapter_text() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
    }

    #[test]
    fn skips_head_script_and_style_content() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();
        let text = book.chapter_text(0).unwrap();

        assert!(!text.contains("Head Title"));
        assert!(!text.contains("script-marker"));
        assert!(!text.contains("color: red"));
    }

    #[test]
    fn returns_chapter_bytes() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_bytes(1).unwrap(), CH2.as_bytes().to_vec());
    }

    #[test]
    fn rejects_out_of_range_chapter() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            book.chapter_bytes(2),
            Err(CoreError::DocumentNotFound(_))
        ));
        assert!(matches!(
            book.chapter_text(9),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn rejects_plain_zip() {
        let bytes = write_zip(&[("a.txt", b"plain")]);
        assert!(matches!(
            EpubBook::open(&bytes),
            Err(CoreError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn rejects_duplicate_mimetype_entries() {
        let bytes = build_epub_with_duplicate_mimetype();
        // UnsupportedFormat ではなく、重複検出による Validation になること。
        assert!(matches!(
            open_error(&bytes),
            CoreError::Validation(message) if message.contains("mimetype")
        ));
    }

    #[test]
    fn rejects_broken_container_or_package() {
        // container.xml 欠落。
        let missing_container = build_epub(&[("OEBPS/content.opf", OPF.as_bytes())]);
        // rootfile 要素なし。
        let no_rootfile =
            epub_with_container(r#"<?xml version="1.0"?><container><rootfiles/></container>"#);
        // OPF 欠落（full-path の先にエントリが無い）。
        let missing_opf = build_epub(&[("META-INF/container.xml", CONTAINER.as_bytes())]);
        // container.xml が壊れた XML（終了タグの不一致）。
        let broken_container = epub_with_container("<container><rootfiles></container>");
        // OPF が壊れた XML（終了タグの不一致）。
        let broken_opf = epub_with_opf("<package><metadata></package>");
        // OPF が途中で切れている。
        let truncated_opf = epub_with_opf("<package><metadata>");

        for bytes in [
            missing_container,
            no_rootfile,
            missing_opf,
            broken_container,
            broken_opf,
            truncated_opf,
        ] {
            assert!(matches!(open_error(&bytes), CoreError::Archive(_)));
        }
    }

    #[test]
    fn rejects_spine_idref_missing_from_manifest() {
        let bytes = epub_with_opf(&OPF.replace("idref=\"ch1\"", "idref=\"missing\""));
        assert!(matches!(open_error(&bytes), CoreError::Archive(_)));
    }

    #[test]
    fn rejects_manifest_item_without_id() {
        // `<item id="ch1" .../>` から id だけを外した OPF。
        let opf = OPF.replace(
            r#"<item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>"#,
            r#"<item href="ch1.xhtml" media-type="application/xhtml+xml"/>"#,
        );
        let bytes = epub_with_opf(&opf);

        assert_fixed_archive_error(open_error(&bytes), "manifest の item に id がありません");
    }

    #[test]
    fn rejects_manifest_item_without_href() {
        // 既存 fixture の `<item id="ch1" .../>` から href だけを外した OPF。
        let opf = OPF.replace(
            r#"<item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>"#,
            r#"<item id="ch1" media-type="application/xhtml+xml"/>"#,
        );
        let bytes = epub_with_opf(&opf);

        assert_fixed_archive_error(open_error(&bytes), "manifest の item に href がありません");
    }

    /// manifest の id 重複は、先着を黙って残さず Validation で拒否する。
    #[test]
    fn rejects_duplicate_manifest_ids() {
        let opf = OPF.replace(
            r#"<item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>"#,
            r#"<item id="ch1" href="ch2.xhtml" media-type="application/xhtml+xml"/>"#,
        );
        match open_error(&epub_with_opf(&opf)) {
            CoreError::Validation(message) => {
                assert_eq!(message, "manifest の item の id が重複しています");
                // ADR-0212 の redaction 契約: id 値をエラー文言に出さない。
                assert!(!message.contains("ch1"));
            }
            other => panic!("Validation を期待したが {other:?} だった"),
        }
    }

    /// 不正な `%XX` の href は生名で開けても、復号フォールバック時に Validation になる。
    ///
    /// リテラル `%` を含む実文件名を受容するため、OPF 解析時点では拒否しない。
    #[test]
    fn rejects_manifest_href_with_malformed_percent_encoding_on_read() {
        let opf = OPF.replace("href=\"ch2.xhtml\"", "href=\"ch%zz2.xhtml\"");
        let book = EpubBook::open(&epub_with_opf(&opf)).unwrap();
        assert!(matches!(
            book.chapter_bytes(1),
            Err(CoreError::Validation(_))
        ));
    }

    #[test]
    fn rejects_href_escaping_root() {
        let bytes = epub_with_opf(&OPF.replace("href=\"ch1.xhtml\"", "href=\"../../evil.xhtml\""));
        assert!(matches!(open_error(&bytes), CoreError::Validation(_)));
    }

    #[test]
    fn resolves_href_relative_to_opf_directory() {
        let container = CONTAINER.replace("OEBPS/content.opf", "OEBPS/sub/content.opf");
        let opf = OPF
            .replace("href=\"ch1.xhtml\"", "href=\"../ch1.xhtml\"")
            .replace("href=\"ch2.xhtml\"", "href=\"../ch2.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("OEBPS/sub/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[0].href, "OEBPS/ch1.xhtml");
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
    }

    #[test]
    fn strips_href_fragment() {
        let bytes = epub_with_opf(&OPF.replace("href=\"ch1.xhtml\"", "href=\"ch1.xhtml#s1\""));
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[0].href, "OEBPS/ch1.xhtml");
    }

    #[test]
    fn parses_prefixed_opf() {
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF_PREFIXED.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.metadata().title.as_deref(), Some("サンプル本"));
        assert_eq!(
            book.metadata().creators,
            vec!["Alice".to_string(), "Bob".to_string()]
        );
        assert_eq!(book.metadata().version.as_deref(), Some("3.0"));
        assert_eq!(
            book.metadata().identifier.as_deref(),
            Some("urn:isbn:9784000000000")
        );
        assert_eq!(
            book.metadata().modified.as_deref(),
            Some("2024-05-02T10:00:00Z")
        );

        let chapters = book.chapters();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].href, "OEBPS/ch1.xhtml");
        assert!(chapters[0].linear);
        assert_eq!(chapters[1].href, "OEBPS/ch2.xhtml");
        assert!(!chapters[1].linear);
    }

    #[test]
    fn reads_percent_encoded_href() {
        let opf = OPF
            .replace("href=\"ch1.xhtml\"", "href=\"ch%201.xhtml\"")
            .replace("href=\"ch2.xhtml\"", "href=\"ch%202.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch 1.xhtml", CH1.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[0].href, "OEBPS/ch%201.xhtml");
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
        // 復号できない名前は元のエラーのまま。
        assert!(matches!(
            book.chapter_bytes(1),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    /// リテラル `%` を含む実文件名は、復号フォールバックより先に生名で読める。
    #[test]
    fn reads_chapter_with_literal_percent_in_name() {
        let opf = OPF.replace("href=\"ch2.xhtml\"", "href=\"100%.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/100%.xhtml", CH2.as_bytes()),
        ]);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[1].href, "OEBPS/100%.xhtml");
        assert!(book.chapter_text(1).unwrap().contains("第二章"));
        assert_eq!(
            book.resolve_chapter_link(0, "100%.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
    }

    #[test]
    fn resolves_href_for_opf_at_root() {
        let container = CONTAINER.replace("OEBPS/content.opf", "content.opf");
        let opf = OPF.replace("href=\"ch1.xhtml\"", "href=\"./text/ch1.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("content.opf", opf.as_bytes()),
            ("text/ch1.xhtml", CH1.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        // OPF がルート直下なら、`./` を挟まずそのまま解決する。
        assert_eq!(book.chapters()[0].href, "text/ch1.xhtml");
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);
    }

    #[test]
    fn breaks_lines_at_block_elements_and_br() {
        let chapter = concat!(
            "<html><body>",
            "<div>one<br/>two</div>",
            "<ul><li>three</li><li>four</li></ul>",
            "<table><tr><td>five</td><td>six</td></tr></table>",
            "</body></html>"
        );
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", chapter.as_bytes()),
        ]);

        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(
            book.chapter_text(0).unwrap(),
            "one\ntwo\nthree\nfour\nfive\nsix"
        );
    }

    #[test]
    fn classifies_chapter_decode_and_parse_errors() {
        let mut bom_chapter = Vec::from(&b"\xEF\xBB\xBF"[..]);
        bom_chapter.extend_from_slice(CH1.as_bytes());

        // BOM 付きの章は BOM を除去して読む。
        let with_bom = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", &bom_chapter),
        ]);
        let book = EpubBook::open(&with_bom).unwrap();
        assert_eq!(book.chapter_text(0).unwrap(), CH1_TEXT);

        // 不正な UTF-8。
        let invalid_utf8 = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", &[0xFF, 0xFE, 0x00]),
        ]);
        let book = EpubBook::open(&invalid_utf8).unwrap();
        assert!(matches!(book.chapter_text(0), Err(CoreError::Archive(_))));

        // XML として壊れた章。
        let broken_xml = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", b"<html><body><p>x</body></html>"),
        ]);
        let book = EpubBook::open(&broken_xml).unwrap();
        assert!(matches!(book.chapter_text(0), Err(CoreError::Archive(_))));
    }

    #[test]
    fn enforces_archive_limits() {
        let bytes = sample_epub();
        let limits = ArchiveLimits {
            max_entries: 2,
            ..ArchiveLimits::default()
        };

        assert!(matches!(
            EpubBook::with_limits(&bytes, limits),
            Err(CoreError::Validation(_))
        ));
        assert!(EpubBook::open(&bytes).is_ok());
    }

    #[test]
    fn epub_book_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EpubBook>();
    }

    #[test]
    fn reads_epub3_nav_in_document_order() {
        let bytes = epub_with_nav(NAV_NESTED);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章 & はじめに", "OEBPS/ch1.xhtml", 0),
                entry_with_fragment("第二章 1 節", "OEBPS/ch2.xhtml", "s1", 1),
            ]
        );
    }

    #[test]
    fn skips_nav_items_without_anchor() {
        let bytes = epub_with_nav(NAV_SPAN_ONLY);
        let book = EpubBook::open(&bytes).unwrap();

        // `<span>` だけの `<li>` はエントリにしない。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("実項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn drops_fragment_from_nav_href() {
        let bytes = epub_with_nav(NAV_NESTED);
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        // `<a href="ch2.xhtml#s1">` はフラグメントを落として章と同じパスになる。
        assert_eq!(toc[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(toc[1].href, book.chapters()[1].href);
        // フラグメント自体は `fragment` に保持する。
        assert_eq!(toc[1].fragment.as_deref(), Some("s1"));
        assert_eq!(toc[0].fragment, None);
    }

    #[test]
    fn keeps_nav_fragment_alongside_resolved_href() {
        let bytes = epub_with_nav(NAV_FRAGMENT_MIXED);
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        assert_eq!(toc.len(), 2);
        // フラグメント付きでも `href` は章と同じ値になる。
        assert_eq!(toc[0].href, "OEBPS/ch1.xhtml");
        assert_eq!(toc[0].href, book.chapters()[0].href);
        assert_eq!(toc[0].fragment.as_deref(), Some("s1"));
        assert_eq!(toc[0].depth, 0);
        // フラグメント無しは `None`。
        assert_eq!(toc[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(toc[1].fragment, None);
        assert_eq!(toc[1].depth, 1);
    }

    #[test]
    fn treats_empty_nav_fragment_as_none() {
        // `#` の後ろが空なら `None`。
        let bytes = epub_with_nav(&nav_with_hrefs(&[Some("ch1.xhtml#")]));
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn keeps_nav_fragment_verbatim() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[
            Some("ch1.xhtml#%E3%81%82"),
            Some("ch2.xhtml#a#b"),
        ]));
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        // パーセントエンコードは split_fragment で復号してから保持する。
        // （テスト名は ADR-0007 由来の履歴名。実際の保持値は復号済み。）
        assert_eq!(toc[0].fragment.as_deref(), Some("あ"));
        // 2 個目以降の `#` は区切りにしない。
        assert_eq!(toc[1].fragment.as_deref(), Some("a#b"));
    }

    /// nav / NCX の不正な `%XX` は目次解析時に Validation で拒否する。
    #[test]
    fn rejects_nav_and_ncx_fragments_with_malformed_percent_encoding() {
        let nav = nav_with_hrefs(&[Some("ch1.xhtml#%zz")]);
        assert!(matches!(
            toc_error(&epub_with_nav(&nav)),
            CoreError::Validation(_)
        ));

        let ncx = ncx_with_sources(&["ch1.xhtml#%2"]);
        let bytes = epub_with_toc(None, Some(&ncx), OPF_NCX);
        assert!(matches!(toc_error(&bytes), CoreError::Validation(_)));
    }

    #[test]
    fn decodes_nav_fragment_entities_and_trims_spaces() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[
            Some("ch1.xhtml#a&amp;b"),
            Some("ch2.xhtml# s1 "),
        ]));
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        // 属性の復号で戻った `&` をそのまま保持する。
        assert_eq!(toc[0].fragment.as_deref(), Some("a&b"));
        // 前後の空白は落とす。
        assert_eq!(toc[1].fragment.as_deref(), Some("s1"));
    }

    #[test]
    fn skips_nav_anchor_without_path() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[Some("#s1"), Some("ch1.xhtml")]));
        let book = EpubBook::open(&bytes).unwrap();

        // フラグメントだけの `<a>` は読み飛ばし、残りのエントリは残す。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn skips_nav_anchor_without_href() {
        let bytes = epub_with_nav(&nav_with_hrefs(&[
            None,
            Some(""),
            Some("  "),
            Some("ch1.xhtml"),
        ]));
        let book = EpubBook::open(&bytes).unwrap();

        // `href` が無い・空・空白だけの `<a>` はエントリにしない。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn keeps_depth_after_empty_path_anchor() {
        let bytes = epub_with_nav(NAV_WITH_EMPTY_PATHS);
        let book = EpubBook::open(&bytes).unwrap();

        // パス部が空の `<a>` を挟んでも深さの規則は変わらない。
        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章", "OEBPS/ch1.xhtml", 1),
                entry_with_fragment("第二章", "OEBPS/ch2.xhtml", "s1", 0),
            ]
        );
    }

    #[test]
    fn prefers_nav_marked_as_toc() {
        let bytes = epub_with_nav(NAV_LANDMARKS_FIRST);
        let book = EpubBook::open(&bytes).unwrap();

        // `landmarks` の nav ではなく `toc` の nav を使う。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("目次の項目", "OEBPS/ch2.xhtml", 0)]
        );
    }

    #[test]
    fn falls_back_to_first_nav_without_type() {
        let bytes = epub_with_nav(NAV_WITHOUT_TYPE);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![entry("唯一の nav", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn treats_multi_token_properties_as_nav() {
        let opf = opf_with_nav_properties("nav scripted");
        let bytes = epub_with_toc(Some(NAV_NESTED), None, &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.toc().unwrap().len(), 2);
    }

    #[test]
    fn reads_epub2_ncx_in_document_order() {
        let bytes = epub_with_toc(None, Some(NCX_NESTED), OPF_NCX);
        let book = EpubBook::open(&bytes).unwrap();

        // `playOrder` ではなく文書順に並べ、内容の無い `navPoint` は落とす。
        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章", "OEBPS/ch1.xhtml", 0),
                // `<content src="ch2.xhtml#s1"/>` のフラグメントを保持する。
                entry_with_fragment("第一節", "OEBPS/ch2.xhtml", "s1", 1),
                entry("第二章", "OEBPS/ch2.xhtml", 0),
            ]
        );
    }

    #[test]
    fn keeps_ncx_fragment() {
        let ncx = ncx_with_sources(&["Text/ch1.xhtml#p1", "ch2.xhtml"]);
        let bytes = epub_with_toc(None, Some(&ncx), OPF_NCX);
        let book = EpubBook::open(&bytes).unwrap();
        let toc = book.toc().unwrap();

        assert_eq!(toc[0].href, "OEBPS/Text/ch1.xhtml");
        assert_eq!(toc[0].fragment.as_deref(), Some("p1"));
        assert_eq!(toc[1].href, "OEBPS/ch2.xhtml");
        assert_eq!(toc[1].fragment, None);
    }

    #[test]
    fn skips_ncx_point_without_path() {
        let ncx = ncx_with_sources(&["#p", "ch1.xhtml"]);
        let bytes = epub_with_toc(None, Some(&ncx), OPF_NCX);
        let book = EpubBook::open(&bytes).unwrap();

        // パス部が空の `navPoint` は読み飛ばし、残りのエントリは残す。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("項目1", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn prefers_nav_over_ncx() {
        let opf = opf_with_nav_and_ncx();
        let bytes = epub_with_toc(Some(NAV_NESTED), Some(NCX_NESTED), &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.toc().unwrap(),
            vec![
                entry("第一章 & はじめに", "OEBPS/ch1.xhtml", 0),
                entry_with_fragment("第二章 1 節", "OEBPS/ch2.xhtml", "s1", 1),
            ]
        );
    }

    #[test]
    fn returns_empty_toc_without_source() {
        // `properties` に `nav` を持たず、NCX も無い OPF。
        let opf = opf_with_nav_properties("cover");
        let bytes = epub_with_toc(Some(NAV_NESTED), None, &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.toc().unwrap().is_empty());
    }

    #[test]
    fn returns_empty_toc_when_source_entry_is_missing() {
        // manifest には nav があるが、アーカイブに nav.xhtml が無い。
        let bytes = epub_with_toc(None, None, OPF);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.toc().unwrap().is_empty());
    }

    #[test]
    fn rejects_broken_nav_xml() {
        let nav = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"ch1.xhtml\">x</li></ol></nav></body></html>";
        assert!(matches!(
            toc_error(&epub_with_nav(nav)),
            CoreError::Archive(_)
        ));

        // 途中で切れた nav。
        let truncated = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"ch1.xhtml\">x</a>";
        assert!(matches!(
            toc_error(&epub_with_nav(truncated)),
            CoreError::Archive(_)
        ));
    }

    #[test]
    fn rejects_broken_ncx_xml() {
        assert!(matches!(
            toc_error(&epub_with_toc(None, Some("<ncx><navMap>"), OPF_NCX)),
            CoreError::Archive(_)
        ));
    }

    #[test]
    fn rejects_nav_href_escaping_root() {
        let nav = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"../../evil.xhtml\">x</a></li></ol></nav></body></html>";
        assert!(matches!(
            toc_error(&epub_with_nav(nav)),
            CoreError::Validation(_)
        ));

        // 空の `href` はエラーにせず読み飛ばす（契約による意図した更新）。
        // 従来は不正な nav として `CoreError::Archive` にしていた。
        let empty = "<html><body><nav epub:type=\"toc\"><ol><li><a href=\"\">x</a></li></ol></nav></body></html>";
        let book = EpubBook::open(&epub_with_nav(empty)).unwrap();
        assert!(book.toc().unwrap().is_empty());
    }

    #[test]
    fn enforces_toc_entry_limit() {
        // 上限ちょうどは通る。
        let ok = epub_with_nav(&nav_with_anchors(MAX_STRUCTURE_ENTRIES));
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.toc().unwrap().len(), MAX_STRUCTURE_ENTRIES);

        // 上限を 1 つ超えると Validation。
        let over = epub_with_nav(&nav_with_anchors(MAX_STRUCTURE_ENTRIES + 1));
        assert!(matches!(toc_error(&over), CoreError::Validation(_)));
    }

    #[test]
    fn reads_landmarks_and_page_list_from_typed_navs() {
        let nav = r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body>
          <nav epub:type="toc"><ol><li><a href="ch1.xhtml">Contents</a></li></ol></nav>
          <nav epub:type="landmarks"><ol>
            <li><a epub:type="cover" href="ch1.xhtml#cover">Cover</a></li>
            <li><a epub:type="bodymatter" href="ch2.xhtml">Start</a>
              <ol><li><a epub:type="index" href="ch2.xhtml#idx">Index</a></li></ol>
            </li>
          </ol></nav>
          <nav epub:type="page-list"><ol>
            <li><a href="ch1.xhtml#page1">1</a></li>
            <li><a href="ch2.xhtml#page2">2</a></li>
          </ol></nav>
        </body></html>"#;
        let book = EpubBook::open(&epub_with_nav(nav)).unwrap();

        assert_eq!(
            book.landmarks().unwrap(),
            vec![
                entry_with_fragment("Cover", "OEBPS/ch1.xhtml", "cover", 0),
                entry("Start", "OEBPS/ch2.xhtml", 0),
                entry_with_fragment("Index", "OEBPS/ch2.xhtml", "idx", 1),
            ]
        );
        assert_eq!(
            book.page_list().unwrap(),
            vec![
                entry_with_fragment("1", "OEBPS/ch1.xhtml", "page1", 0),
                entry_with_fragment("2", "OEBPS/ch2.xhtml", "page2", 0),
            ]
        );
        // 通常の目次選択も従来どおり toc 型を優先する。
        assert_eq!(
            book.toc().unwrap(),
            vec![entry("Contents", "OEBPS/ch1.xhtml", 0)]
        );
    }

    #[test]
    fn returns_empty_navigation_lists_when_their_navs_are_missing() {
        let book = EpubBook::open(&epub_with_nav(NAV_NESTED)).unwrap();

        assert!(book.landmarks().unwrap().is_empty());
        assert!(book.page_list().unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_href_in_landmarks() {
        let nav = r#"<html><body>
          <nav epub:type="landmarks"><ol><li><a href="../../escape">x</a></li></ol></nav>
        </body></html>"#;
        let book = EpubBook::open(&epub_with_nav(nav)).unwrap();

        assert!(matches!(book.landmarks(), Err(CoreError::Validation(_))));
    }

    /// 章と同じディレクトリの下にあるリソースのバイト列。
    const PIC: &[u8] = b"\x89PNG\r\n\x1a\npic";

    /// 章の所属ディレクトリの隣にあるリソースのバイト列。
    const PIC_SIBLING: &[u8] = b"\x89PNG\r\n\x1a\nsibling";

    /// 1 章目のバイト列を差し替えた EPUB を組み立てる。
    fn epub_with_chapter(data: &[u8]) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", data),
        ])
    }

    /// `OEBPS/<name>` にリソースを持つ EPUB を組み立てる。
    fn epub_with_resource(name: &str, data: &[u8]) -> Vec<u8> {
        let path = format!("OEBPS/{name}");
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", OPF.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            (path.as_str(), data),
        ])
    }

    /// 章が `OEBPS/text/` にあり、`OEBPS/text/images/pic.png` と
    /// `OEBPS/images/pic.png` を持つ EPUB。
    fn epub_with_subdir_chapter() -> Vec<u8> {
        let opf = OPF.replace("href=\"ch1.xhtml\"", "href=\"text/ch1.xhtml\"");
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/text/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/text/images/pic.png", PIC),
            ("OEBPS/images/pic.png", PIC_SIBLING),
        ])
    }

    /// 章がルート直下にある EPUB（基準ディレクトリが空の場合の検証用）。
    fn epub_with_root_chapter() -> Vec<u8> {
        let container = CONTAINER.replace("OEBPS/content.opf", "content.opf");
        build_epub(&[
            ("META-INF/container.xml", container.as_bytes()),
            ("content.opf", OPF.as_bytes()),
            ("ch1.xhtml", CH1.as_bytes()),
            ("pic.png", PIC),
        ])
    }

    /// `chapter_resource` のエラーを取り出す。
    fn resource_error(book: &EpubBook, index: usize, href: &str) -> CoreError {
        match book.chapter_resource(index, href) {
            Err(err) => err,
            Ok(data) => panic!("エラーを期待したが {} バイト返った", data.len()),
        }
    }

    #[test]
    fn returns_chapter_html_verbatim() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        // XML 宣言・タグ・属性をそのまま返す。
        assert_eq!(book.chapter_html(0).unwrap(), CH1);
    }

    #[test]
    fn strips_single_leading_bom_from_chapter_html() {
        let mut with_bom = Vec::from(&b"\xEF\xBB\xBF"[..]);
        with_bom.extend_from_slice(CH1.as_bytes());
        let mut with_two = Vec::from(&b"\xEF\xBB\xBF\xEF\xBB\xBF"[..]);
        with_two.extend_from_slice(CH1.as_bytes());

        let book = EpubBook::open(&epub_with_chapter(&with_bom)).unwrap();
        assert_eq!(book.chapter_html(0).unwrap(), CH1);

        // BOM は 1 個だけ除き、2 個目は本文として残す。
        let book = EpubBook::open(&epub_with_chapter(&with_two)).unwrap();
        assert_eq!(book.chapter_html(0).unwrap(), format!("\u{FEFF}{CH1}"));
    }

    #[test]
    fn rejects_chapter_html_with_invalid_utf8() {
        let bytes = epub_with_chapter(&[0xFF, 0xFE, 0x00]);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(book.chapter_html(0), Err(CoreError::Archive(_))));
    }

    #[test]
    fn rejects_out_of_range_chapter_html() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            book.chapter_html(2),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn reads_resource_next_to_chapter() {
        let bytes = epub_with_resource("pic.png", PIC);
        let book = EpubBook::open(&bytes).unwrap();

        // `OEBPS/ch1.xhtml` から見た `pic.png` は `OEBPS/pic.png`。
        assert_eq!(book.chapter_resource(0, "pic.png").unwrap(), PIC.to_vec());
    }

    #[test]
    fn resolves_resource_in_sub_and_parent_directories() {
        let bytes = epub_with_subdir_chapter();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[0].href, "OEBPS/text/ch1.xhtml");
        // 所属ディレクトリの下のサブディレクトリ。
        assert_eq!(
            book.chapter_resource(0, "images/pic.png").unwrap(),
            PIC.to_vec()
        );
        // 所属ディレクトリの親を経由した参照。
        assert_eq!(
            book.chapter_resource(0, "../images/pic.png").unwrap(),
            PIC_SIBLING.to_vec()
        );
    }

    #[test]
    fn drops_resource_fragment() {
        let bytes = epub_with_resource("pic.png", PIC);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.chapter_resource(0, "pic.png#frag").unwrap(),
            PIC.to_vec()
        );
    }

    #[test]
    fn reads_percent_encoded_resource() {
        let bytes = epub_with_resource("a b.png", PIC);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_resource(0, "a%20b.png").unwrap(), PIC.to_vec());
    }

    #[test]
    fn rejects_resource_escaping_root() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            resource_error(&book, 0, "../../evil.png"),
            CoreError::Validation(_)
        ));
    }

    #[test]
    fn rejects_resource_with_scheme_or_absolute_path() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        for href in ["https://example.com/x.png", "/x.png"] {
            assert!(matches!(
                resource_error(&book, 0, href),
                CoreError::Archive(_)
            ));
        }
    }

    #[test]
    fn rejects_resource_without_path() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        for href in ["", "   ", "#frag"] {
            assert!(matches!(
                resource_error(&book, 0, href),
                CoreError::Archive(_)
            ));
        }
    }

    #[test]
    fn rejects_missing_resource() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            resource_error(&book, 0, "missing.png"),
            CoreError::DocumentNotFound(_)
        ));
    }

    #[test]
    fn rejects_out_of_range_chapter_resource() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            resource_error(&book, 2, "pic.png"),
            CoreError::DocumentNotFound(_)
        ));
    }

    #[test]
    fn resolves_resource_for_chapter_at_root() {
        let bytes = epub_with_root_chapter();
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapters()[0].href, "ch1.xhtml");
        assert_eq!(book.chapter_resource(0, "pic.png").unwrap(), PIC.to_vec());
        // ルート直下の章からの親参照はルートを越える。
        assert!(matches!(
            resource_error(&book, 0, "../evil.png"),
            CoreError::Validation(_)
        ));
    }

    #[test]
    fn rejects_encoded_or_backslash_resource_traversal() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        // バックスラッシュ区切りの親参照は読み出しの検証で拒否する。
        assert!(matches!(
            resource_error(&book, 0, "..\\evil.png"),
            CoreError::Validation(_)
        ));
        // `%XX` 復号後の `..` も読めない（復号側の拒否は最初のエラーで報告する）。
        assert!(matches!(
            resource_error(&book, 0, "%2e%2e%2fevil.png"),
            CoreError::DocumentNotFound(_)
        ));
    }

    /// 表紙画像のバイト列。
    const COVER: &[u8] = b"\x89PNG\r\n\x1a\ncover";

    /// 表紙を EPUB3 の `properties="cover-image"` で宣言する OPF。
    ///
    /// EPUB2 の `<meta name="cover">` も併記し、EPUB3 の宣言を優先することを
    /// 確認できるようにする。
    const OPF_COVER3: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="meta-cover"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="meta-cover" href="meta.png" media-type="image/png"/>
    <item id="prop-cover" href="images/cover.png" media-type="image/png" properties="cover-image"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// 表紙を EPUB2 の `<meta name="cover">` で宣言する OPF。
    const OPF_COVER2: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="cover-img" href="cover.jpg" media-type="image/jpeg"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// 表紙が画像ではない item を指す OPF。
    const OPF_COVER_PAGE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="cover-page"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="cover-page" href="cover.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// 表紙の `href` がパーセントエンコードされた OPF。
    const OPF_COVER_ENCODED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="pub-id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>サンプル本</dc:title>
    <dc:identifier id="pub-id">urn:isbn:9784000000000</dc:identifier>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="cover-img" href="cover%20img.png" media-type="image/png"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>
"#;

    /// OPF と追加エントリから EPUB を組み立てる（章は標準の 2 つ）。
    fn epub_with_opf_and_files(opf: &str, extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut files: Vec<(&str, &[u8])> = vec![
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ];
        files.extend_from_slice(extra);
        build_epub(&files)
    }

    /// 非対応の圧縮方式として扱われるコード（既知の方式と重ならない値）。
    const UNSUPPORTED_COMPRESSION: u16 = 99;

    /// 指定エントリの圧縮方式だけを書き換えた ZIP を返す。
    ///
    /// 一覧の走査は通り、そのエントリの読み出しだけが失敗するアーカイブを作る
    /// ために使う。ローカルファイルヘッダーとセントラルディレクトリの両方を
    /// 書き換えないと整合しないため、両方を探して同じ値にする。
    fn patch_compression_method(zip: &[u8], name: &str, method: u16) -> Vec<u8> {
        let mut bytes = zip.to_vec();
        let name = name.as_bytes();
        for (signature, name_offset, method_offset) in
            [(b"PK\x03\x04", 30, 8), (b"PK\x01\x02", 46, 10)]
        {
            let at = find_compression_method(&bytes, signature, name_offset, method_offset, name);
            bytes[at..at + 2].copy_from_slice(&method.to_le_bytes());
        }
        bytes
    }

    /// `name` を持つヘッダーの圧縮方式の位置を返す（テスト専用）。
    ///
    /// `name_offset` は署名の先頭からファイル名まで、`method_offset` は署名の
    /// 先頭から圧縮方式までの距離。見つからない場合はテストの前提が崩れて
    /// いるため panic する。
    fn find_compression_method(
        zip: &[u8],
        signature: &[u8; 4],
        name_offset: usize,
        method_offset: usize,
        name: &[u8],
    ) -> usize {
        for start in 0..zip.len().saturating_sub(4) {
            if &zip[start..start + 4] != signature {
                continue;
            }
            let name_start = start + name_offset;
            if zip.get(name_start..name_start + name.len()) == Some(name) {
                let at = start + method_offset;
                assert!(at + 2 <= zip.len(), "ヘッダーが途中で切れています");
                return at;
            }
        }
        panic!("ZIP のヘッダーが見つかりません");
    }

    /// 1 章目を差し替えた EPUB を組み立てる（目次と `<title>` の検証用）。
    fn epub_with_chapter_and_opf(chapter: &str, opf: &str) -> Vec<u8> {
        build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", chapter.as_bytes()),
            ("OEBPS/ch2.xhtml", CH2.as_bytes()),
        ])
    }

    /// 目次の情報源を持たない OPF を返す。
    fn opf_without_toc() -> String {
        opf_with_nav_properties("cover")
    }

    #[test]
    fn is_image_media_type_accepts_empty_and_image_types() {
        // 未宣言（空）を許容し、明示された image/ を大文字・前後空白も含めて受け付け、
        // 画像以外（text/html）は拒否する（ADR-0008 の契約）。
        assert!(is_image_media_type(""));
        assert!(is_image_media_type("image/png"));
        assert!(is_image_media_type(" IMAGE/SVG+XML "));
        assert!(!is_image_media_type("text/html"));
    }

    #[test]
    fn reads_cover_image_from_epub3_properties() {
        let bytes = epub_with_opf_and_files(OPF_COVER3, &[("OEBPS/images/cover.png", COVER)]);
        let book = EpubBook::open(&bytes).unwrap();
        let cover = book.cover_image().unwrap().unwrap();

        // EPUB3 の `properties` を優先する（`meta` は別の item を指している）。
        assert_eq!(cover.id, "prop-cover");
        assert_eq!(cover.href, "OEBPS/images/cover.png");
        assert_eq!(cover.media_type, "image/png");
        assert_eq!(cover.bytes, COVER.to_vec());
    }

    #[test]
    fn reads_cover_image_from_epub2_meta() {
        let bytes = epub_with_opf_and_files(OPF_COVER2, &[("OEBPS/cover.jpg", COVER)]);
        let book = EpubBook::open(&bytes).unwrap();
        let cover = book.cover_image().unwrap().unwrap();

        assert_eq!(cover.id, "cover-img");
        assert_eq!(cover.href, "OEBPS/cover.jpg");
        assert_eq!(cover.media_type, "image/jpeg");
        assert_eq!(cover.bytes, COVER.to_vec());
    }

    #[test]
    fn returns_none_without_cover_declaration() {
        // `properties="nav"` を差し替えた OPF は表紙を宣言していない。
        let bytes = epub_with_toc(None, None, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn returns_none_when_cover_id_is_unknown() {
        // 標準の OPF は `cover-image` を指すが、その id の item が無い。
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn returns_none_when_cover_entry_is_missing() {
        // 宣言はあるがアーカイブにエントリが無い場合は表紙なしとする。
        let bytes = epub_with_opf_and_files(OPF_COVER2, &[]);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn ignores_cover_item_that_is_not_an_image() {
        let bytes = epub_with_opf_and_files(
            OPF_COVER_PAGE,
            &[("OEBPS/cover.xhtml", b"<html><body>cover</body></html>")],
        );
        let book = EpubBook::open(&bytes).unwrap();

        // 表紙ページ（XHTML）を指す `id` は画像ではないため表紙にしない。
        assert!(book.cover_image().unwrap().is_none());
    }

    #[test]
    fn reads_percent_encoded_cover_href() {
        let bytes = epub_with_opf_and_files(OPF_COVER_ENCODED, &[("OEBPS/cover img.png", COVER)]);
        let book = EpubBook::open(&bytes).unwrap();
        let cover = book.cover_image().unwrap().unwrap();

        // `href` は元の綴りのまま、バイト列は復号した名前で読む。
        assert_eq!(cover.href, "OEBPS/cover%20img.png");
        assert_eq!(cover.bytes, COVER.to_vec());
    }

    #[test]
    fn reads_chapter_title_from_toc() {
        let bytes = epub_with_nav(NAV_NESTED);
        let book = EpubBook::open(&bytes).unwrap();

        // 目次の表示名を章と `href` で対応付けて返す（章の `<title>` より優先）。
        assert_eq!(
            book.chapter_title(0).unwrap().as_deref(),
            Some("第一章 & はじめに")
        );
        assert_eq!(
            book.chapter_title(1).unwrap().as_deref(),
            Some("第二章 1 節")
        );
    }

    #[test]
    fn falls_back_to_chapter_document_title() {
        // 目次の情報源を持たない EPUB では章の `<title>` を使う。
        let bytes = epub_with_toc(None, None, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(
            book.chapter_title(0).unwrap().as_deref(),
            Some("Head Title")
        );
        // `<title>` を持たない章は `None`。
        assert_eq!(book.chapter_title(1).unwrap(), None);
    }

    #[test]
    fn normalizes_chapter_document_title() {
        let chapter = "<html><head><title>  第一章 &amp; 序 &#x3042;  </title></head>\
                       <body><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        // 実体参照を戻し、連続する空白をまとめて前後を落とす。
        assert_eq!(
            book.chapter_title(0).unwrap().as_deref(),
            Some("第一章 & 序 あ")
        );
    }

    #[test]
    fn treats_empty_document_title_as_none() {
        let chapter = "<html><head><title/></head><body><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_title(0).unwrap(), None);
    }

    #[test]
    fn rejects_out_of_range_chapter_title() {
        let bytes = sample_epub();
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(
            book.chapter_title(9),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn rejects_chapter_title_when_document_is_broken() {
        // 目次が無く、`<title>` も持たないまま要素が閉じられていない章。
        let chapter = "<html><body><p>x</body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert!(matches!(book.chapter_title(0), Err(CoreError::Archive(_))));
    }

    #[test]
    fn ignores_titles_outside_the_document_head() {
        // `<head>` に `<title>` を持たず、本文のインライン SVG にだけ持つ章。
        let chapter = "<html><head></head><body><svg><title>図</title></svg><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_title(0).unwrap(), None);
    }

    #[test]
    fn prefers_head_title_over_inline_svg_title() {
        let chapter = "<html><head><title>章の名前</title></head>\
                       <body><svg><title>図</title></svg><p>x</p></body></html>";
        let bytes = epub_with_chapter_and_opf(chapter, &opf_without_toc());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_title(0).unwrap().as_deref(), Some("章の名前"));
    }

    #[test]
    fn reports_cover_errors_other_than_a_missing_entry() {
        // 表紙エントリの圧縮方式だけを非対応の値に書き換えた EPUB を組み立てる。
        let bytes = epub_with_opf_and_files(OPF_COVER2, &[("OEBPS/cover.jpg", COVER)]);
        let broken = patch_compression_method(&bytes, "OEBPS/cover.jpg", UNSUPPORTED_COMPRESSION);
        let book = EpubBook::open(&broken).unwrap();

        // エントリの欠落は表紙なしにするが、読み出しの失敗は報告する。
        assert!(matches!(book.cover_image(), Err(CoreError::Archive(_))));
    }

    #[test]
    fn resolves_links_to_another_chapter() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        assert_eq!(
            book.resolve_chapter_link(0, "ch2.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
        // 同じ章を指すリンクも解決する。
        assert_eq!(
            book.resolve_chapter_link(0, "ch1.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 0,
                fragment: None,
            })
        );
    }

    #[test]
    fn resolves_links_with_fragments() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        assert_eq!(
            book.resolve_chapter_link(0, "ch2.xhtml#part").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: Some(String::from("part")),
            })
        );
        // フラグメントだけのリンクは同じ章を指す。
        assert_eq!(
            book.resolve_chapter_link(0, " #part ").unwrap(),
            Some(ChapterLink {
                chapter_index: 0,
                fragment: Some(String::from("part")),
            })
        );
    }

    #[test]
    fn resolves_parent_directory_links_from_a_subdirectory_chapter() {
        let opf = OPF
            .replace("href=\"ch1.xhtml\"", "href=\"text/ch1.xhtml\"")
            .replace("href=\"ch2.xhtml\"", "href=\"text/ch2.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/text/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/text/ch2.xhtml", CH2.as_bytes()),
        ]);
        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[1].href, "OEBPS/text/ch2.xhtml");

        assert_eq!(
            book.resolve_chapter_link(0, "../text/ch2.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
    }

    #[test]
    fn resolves_percent_encoded_chapter_links() {
        // 章の実体は空白入りの名前で、リンクは `%20` で書かれている。
        let opf = OPF.replace("href=\"ch2.xhtml\"", "href=\"ch 2.xhtml\"");
        let bytes = build_epub(&[
            ("META-INF/container.xml", CONTAINER.as_bytes()),
            ("OEBPS/content.opf", opf.as_bytes()),
            ("OEBPS/ch1.xhtml", CH1.as_bytes()),
            ("OEBPS/ch 2.xhtml", CH2.as_bytes()),
        ]);
        let book = EpubBook::open(&bytes).unwrap();
        assert_eq!(book.chapters()[1].href, "OEBPS/ch 2.xhtml");

        assert_eq!(
            book.resolve_chapter_link(0, "ch%202.xhtml").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: None,
            })
        );
    }

    #[test]
    fn rejects_links_outside_the_book() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        for href in [
            "",
            "#",
            "https://example.com/ch2.xhtml",
            "/ch2.xhtml",
            "../../evil.xhtml",
            "other.xhtml",
            "ch2.xhtml?query",
        ] {
            assert_eq!(
                book.resolve_chapter_link(0, href).unwrap(),
                None,
                "unexpected link: {href:?}"
            );
        }
    }

    /// http(s) 以外のスキームも core と同じ規則で拒否する。
    #[test]
    fn rejects_non_http_external_schemes_in_chapter_links() {
        let book = EpubBook::open(&sample_epub()).unwrap();
        for href in [
            "file:///etc/passwd",
            "data:text/html,<p>x</p>",
            "javascript:alert(1)",
            "ftp://example.com/ch2.xhtml",
            "mailto:test@example.com",
        ] {
            assert_eq!(
                book.resolve_chapter_link(0, href).unwrap(),
                None,
                "unexpected link: {href:?}"
            );
        }
    }

    /// fragment は percent decode してから `ChapterLink` に格納する。
    #[test]
    fn decodes_percent_encoded_chapter_link_fragment() {
        let book = EpubBook::open(&sample_epub()).unwrap();
        assert_eq!(
            book.resolve_chapter_link(0, "ch2.xhtml#%E7%AC%AC1%E7%AF%80").unwrap(),
            Some(ChapterLink {
                chapter_index: 1,
                fragment: Some("第1節".to_string()),
            })
        );
        assert_eq!(
            book.resolve_chapter_link(0, "#%E7%AC%AC1%E7%AF%80").unwrap(),
            Some(ChapterLink {
                chapter_index: 0,
                fragment: Some("第1節".to_string()),
            })
        );
    }

    /// 復号失敗は Validation として報告し、未復号文字列を位置指定子に使わない。
    #[test]
    fn rejects_malformed_percent_encoded_chapter_link() {
        let book = EpubBook::open(&sample_epub()).unwrap();
        for href in [
            "ch2.xhtml#%zz",
            "#%2",
            "ch%zz2.xhtml",
            "ch2.xhtml#%C3%28",
        ] {
            assert!(
                matches!(
                    book.resolve_chapter_link(0, href),
                    Err(CoreError::Validation(_))
                ),
                "Validation を期待したが {href:?} だった"
            );
        }
    }

    /// 章リソースの `%XX` 復号失敗も Validation として報告する。
    #[test]
    fn rejects_malformed_percent_encoded_resource() {
        let book = EpubBook::open(&sample_epub()).unwrap();
        for href in ["%zz.png", "%2.png", "%C3%28.png"] {
            assert!(
                matches!(
                    resource_error(&book, 0, href),
                    CoreError::Validation(_)
                ),
                "Validation を期待したが {href:?} だった"
            );
        }
    }

    #[test]
    fn reports_out_of_range_index_for_chapter_links() {
        let book = EpubBook::open(&sample_epub()).unwrap();

        assert!(matches!(
            book.resolve_chapter_link(9, "ch2.xhtml"),
            Err(CoreError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn malformed_chapter_xhtml_is_passed_through() {
        // 章本文は XML 検証せずそのまま返す（表示側のサニタイズに委ねる）。
        let malformed = "<p>unclosed";
        let bytes = epub_with_chapter(malformed.as_bytes());
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.chapter_html(0).unwrap(), malformed);
    }

    #[test]
    fn malformed_opf_is_rejected() {
        // OPF は XML として解析し、構文エラーは fail-closed で拒否する。
        let bytes = epub_with_chapter_and_opf(CH1, "<package><metadata>");
        assert!(EpubBook::open(&bytes).is_err());
    }

    #[test]
    fn remote_resources_property_does_not_break_navigation() {
        // `remote-resources` は解釈せず無視するが、nav と章本文は従来どおり読める。
        // リモート資源への実際のアクセスは表示側のサニタイズで塞がれる。
        let opf = opf_with_nav_properties("nav remote-resources");
        let bytes = epub_with_toc(Some(NAV_NESTED), None, &opf);
        let book = EpubBook::open(&bytes).unwrap();

        assert!(!book.toc().unwrap().is_empty());
        assert!(book.chapter_html(0).unwrap().contains("<h1>T</h1>"));
    }

    /// `count` 個の `<item>` を持つ OPF（`id` は `item0` から一意）。
    fn opf_with_manifest_items(count: usize) -> String {
        let mut opf = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"pub-id\">\
             <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>t</dc:title></metadata><manifest>",
        );
        for index in 0..count {
            opf.push_str(&format!(
                "<item id=\"item{index}\" href=\"ch1.xhtml\" media-type=\"application/xhtml+xml\"/>"
            ));
        }
        opf.push_str("</manifest><spine><itemref idref=\"item0\"/></spine></package>");
        opf
    }

    /// `count` 個の `<itemref>` を持つ OPF（manifest は 1 件）。
    fn opf_with_spine_refs(count: usize) -> String {
        let mut opf = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"pub-id\">\
             <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>t</dc:title></metadata>\
             <manifest><item id=\"spine-target-secret\" href=\"ch1.xhtml\" media-type=\"application/xhtml+xml\"/></manifest><spine>",
        );
        for _ in 0..count {
            opf.push_str("<itemref idref=\"spine-target-secret\"/>");
        }
        opf.push_str("</spine></package>");
        opf
    }

    /// `count` 個の `<dc:creator>` を並べた OPF（`collected` を増やす）。
    fn opf_with_creators(count: usize) -> String {
        let mut opf = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"pub-id\">\
             <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">",
        );
        for index in 0..count {
            opf.push_str(&format!("<dc:creator>creator{index}</dc:creator>"));
        }
        opf.push_str(OPF_LIMIT_TAIL);
        opf
    }

    /// `count` 個の `<dc:creator>` を入れ子にした OPF（`captures` を増やす）。
    fn opf_with_nested_creators(count: usize) -> String {
        let mut opf = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"pub-id\">\
             <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">",
        );
        for _ in 0..count {
            opf.push_str("<dc:creator id=\"capture-overflow-marker\">");
        }
        opf.push_str("inner");
        for _ in 0..count {
            opf.push_str("</dc:creator>");
        }
        opf.push_str(OPF_LIMIT_TAIL);
        opf
    }

    /// 上限テスト用 OPF の共通末尾（manifest 1 件と spine 1 件）。
    const OPF_LIMIT_TAIL: &str = "</metadata>\
        <manifest><item id=\"ch1\" href=\"ch1.xhtml\" media-type=\"application/xhtml+xml\"/></manifest>\
        <spine><itemref idref=\"ch1\"/></spine></package>";

    #[test]
    fn enforces_manifest_item_limit() {
        let ok = epub_with_opf(&opf_with_manifest_items(MAX_STRUCTURE_ENTRIES));
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.metadata().title.as_deref(), Some("t"));

        let over = epub_with_opf(&opf_with_manifest_items(MAX_STRUCTURE_ENTRIES + 1));
        match open_error(&over) {
            CoreError::Validation(message) => {
                assert!(message.contains("manifest の item"), "{message}");
                // 超過分の id（item10000）はエラー文言に出さない。
                assert!(!message.contains("item10000"), "{message}");
            }
            other => panic!("CoreError::Validation を期待したが {other:?}"),
        }
    }

    #[test]
    fn enforces_spine_itemref_limit() {
        let ok = epub_with_opf(&opf_with_spine_refs(MAX_STRUCTURE_ENTRIES));
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.chapters().len(), MAX_STRUCTURE_ENTRIES);

        let over = epub_with_opf(&opf_with_spine_refs(MAX_STRUCTURE_ENTRIES + 1));
        match open_error(&over) {
            CoreError::Validation(message) => {
                assert!(message.contains("spine の itemref"), "{message}");
                // 超過分の idref 値はエラー文言に出さない。
                assert!(!message.contains("spine-target-secret"), "{message}");
            }
            other => panic!("CoreError::Validation を期待したが {other:?}"),
        }
    }

    #[test]
    fn enforces_metadata_collected_limit() {
        let ok = epub_with_opf(&opf_with_creators(MAX_STRUCTURE_ENTRIES));
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.metadata().creators.len(), MAX_STRUCTURE_ENTRIES);

        let over = epub_with_opf(&opf_with_creators(MAX_STRUCTURE_ENTRIES + 1));
        match open_error(&over) {
            CoreError::Validation(message) => {
                assert!(message.contains("メタデータの collected"), "{message}");
                assert!(!message.contains("creator10000"), "{message}");
            }
            other => panic!("CoreError::Validation を期待したが {other:?}"),
        }
    }

    #[test]
    fn enforces_metadata_capture_limit() {
        let ok = epub_with_opf(&opf_with_nested_creators(MAX_STRUCTURE_ENTRIES));
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.metadata().creators, vec!["inner"]);

        let over = epub_with_opf(&opf_with_nested_creators(MAX_STRUCTURE_ENTRIES + 1));
        match open_error(&over) {
            CoreError::Validation(message) => {
                assert!(message.contains("メタデータの capture"), "{message}");
                // 超過分の id とテキストはエラー文言に出さない。
                assert!(!message.contains("capture-overflow-marker"), "{message}");
                assert!(!message.contains("inner"), "{message}");
            }
            other => panic!("CoreError::Validation を期待したが {other:?}"),
        }
    }

    #[test]
    fn enforces_ncx_navpoint_limit() {
        let within = vec!["ch1.xhtml"; MAX_STRUCTURE_ENTRIES];
        let ok = epub_with_toc(None, Some(&ncx_with_sources(&within)), OPF_NCX);
        let book = EpubBook::open(&ok).unwrap();
        assert_eq!(book.toc().unwrap().len(), MAX_STRUCTURE_ENTRIES);

        let over_sources = vec!["ch1.xhtml"; MAX_STRUCTURE_ENTRIES + 1];
        let over = epub_with_toc(None, Some(&ncx_with_sources(&over_sources)), OPF_NCX);
        match toc_error(&over) {
            CoreError::Validation(message) => {
                assert!(message.contains("NCX の navPoint"), "{message}");
                // 超過分の id（n10000）はエラー文言に出さない。
                assert!(!message.contains("n10000"), "{message}");
            }
            other => panic!("CoreError::Validation を期待したが {other:?}"),
        }
    }

    #[test]
    fn structure_limits_do_not_break_normal_epub() {
        let bytes = epub_with_toc(Some(NAV_NESTED), None, OPF);
        let book = EpubBook::open(&bytes).unwrap();

        assert_eq!(book.metadata().title.as_deref(), Some("サンプル本"));
        assert_eq!(book.chapters().len(), 2);
        assert_eq!(book.toc().unwrap().len(), 2);
    }
