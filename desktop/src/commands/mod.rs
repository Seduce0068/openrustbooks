//! Command modules for Tauri.

pub mod ai;
pub mod config;
pub mod document;
pub mod fs;
pub mod window;

use openrustbooks_core::CoreError;

/// IPC 境界で使う固定 code → 固定文言のマッパー。
///
/// `CoreError` の `Display` は内部向けで、ファイルパス / provider host /
/// レスポンス本文 / API key を含み得る。IPC へは `CoreError::code()` が返す
/// 固定 code を通し、core の共有 mapper が返す固定文言だけを返す
/// （ADR-0287 / R1-ER01）。既存文言は core 側で完全一致するよう定義されている。
pub(crate) fn ipc_message(code: &str) -> &'static str {
    openrustbooks_core::error::fixed_message(code)
}

/// `CoreError` を IPC 用の固定文言文字列へ変換する。
///
/// `e.to_string()` を IPC へ返す既存経路をすべてこれに置き換える。
/// 内部ログを残す場合も `Display` ではなく `code()` だけを記録すること。
pub(crate) fn ipc_error(error: CoreError) -> String {
    ipc_message(error.code()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_message_returns_fixed_text_and_masks_unknown_codes() {
        assert_eq!(ipc_message("CONFIG_CONFLICT"), "Configuration conflict");
        assert_eq!(
            ipc_message("CONFIG_VALIDATION"),
            "Configuration validation failed"
        );
        assert_eq!(
            ipc_message("CONFIG_STORAGE"),
            "Configuration storage failed"
        );
        assert_eq!(
            ipc_message("CONFIG_LOAD_FAILED"),
            crate::state::CONFIG_LOAD_FAILED_MESSAGE
        );
        assert_eq!(ipc_message("AI_PROVIDER"), "AI provider request failed");
        // 未知 code は code 文字列自体を出さず既定文言へ落とす。
        assert_eq!(
            ipc_message("sk-unknown-code"),
            "Internal configuration error"
        );
    }

    /// 共有 mapper へ委譲しても既存 IPC 文言が変わらないこと。
    #[test]
    fn ipc_message_matches_core_fixed_message() {
        for code in [
            "CONFIG_CONFLICT",
            "CONFIG_VALIDATION",
            "CONFIG_STORAGE",
            "CONFIG_LOAD_FAILED",
            "AI_PROVIDER",
            "WINDOW_INTERNAL",
            "sk-unknown-code",
        ] {
            assert_eq!(
                ipc_message(code),
                openrustbooks_core::error::fixed_message(code),
                "code={code}"
            );
        }
        // core の公開定数を共有 mapper が参照していること。
        assert_eq!(
            ipc_message("CONFIG_LOAD_FAILED"),
            openrustbooks_core::error::CONFIG_LOAD_FAILED_MESSAGE
        );
        // desktop 側の既存定数との一致も維持する（E2E 依存文言）。
        assert_eq!(
            openrustbooks_core::error::CONFIG_LOAD_FAILED_MESSAGE,
            crate::state::CONFIG_LOAD_FAILED_MESSAGE
        );
    }

    /// CoreError::to_string() に含まれる secret / path / host / body が
    /// IPC 結果文字列へ一切漏れないこと（R1-ER01 の受け入れ条件）。
    #[test]
    fn ipc_error_never_leaks_secret_path_host_or_body() {
        let secret = "sk-ipc-leak-do-not-expose";
        let path = "/home/koppe/secret/book.epub";
        let host = "https://api.secret.internal";
        let body = "response-body-with-token";

        let errors = [
            CoreError::ai_provider(format!("key={secret} host={host} body={body}")),
            CoreError::from(std::io::Error::other(format!("failed to read {path}"))),
            CoreError::storage(format!("{path} is unreachable with {secret}")),
            CoreError::validation(format!("bad {path} body={body}")),
            CoreError::internal(format!("bug at {host} with {secret}")),
        ];

        for error in errors {
            let rendered = ipc_error(error);
            for needle in [secret, path, host, body] {
                assert!(
                    !rendered.contains(needle),
                    "IPC 結果へ {needle} が漏れています: {rendered}"
                );
            }
        }
    }
}
