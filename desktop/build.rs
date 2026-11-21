fn main() {
    let mut attrs = tauri_build::Attributes::new();

    // `wdio` feature を有効にした debug ビルドのみ、WDIO 用 capability を読み込む。
    #[cfg(all(debug_assertions, feature = "wdio"))]
    {
        let merge = r#"{"app":{"security":{"capabilities":["wdio"]}}}"#;
        println!("cargo:rustc-env=TAURI_CONFIG={merge}");
        attrs = attrs.capabilities_path_pattern("capabilities/wdio/*.json");
    }

    #[cfg(not(all(debug_assertions, feature = "wdio")))]
    {
        attrs = attrs.capabilities_path_pattern("capabilities/default/*.json");
    }

    tauri_build::try_build(attrs).expect("failed to run tauri build script");
}
