//! Application state management.

use openrustbooks_core::{
    AppConfig, CoreError, CoreResult, LintConfig,
    ai_markdown::AiMarkdownService,
    document::{Document, DocumentId},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 起動時の config.toml 読み込み失敗時に保存を拒否する固定文言。
///
/// 設定値・パス・生エラー文字列は一切含めない（R2-ER01 / fail-closed）。
pub const CONFIG_LOAD_FAILED_MESSAGE: &str = "Configuration load failed; saving is disabled";

/// 未保存文書を non-force で閉じようとした場合の固定文言。
///
/// 文書 id / path は一切含めない。IPC へは `ipc_error` が固定 code 経由で
/// 変換するため、この文言自体は内部エラーとしてのみ使う（ADR-0287）。
pub const DOCUMENT_UNSAVED_MESSAGE: &str = "Document has unsaved changes";

/// 保存開始後に本文が変更された場合の固定文言。
///
/// 文書 id / path / 本文は一切含めない。IPC へは `ipc_error` が固定 code
/// 経由で変換するため、この文言自体は内部エラーとしてのみ使う。
pub const DOCUMENT_SAVE_CONFLICT_MESSAGE: &str = "Document changed since it was read";

/// 編集対象の文書が存在しない場合の固定文言。
///
/// 文書 id / path は一切含めない（IPC へは `ipc_error` が固定 code 経由で
/// 変換するため、この文言自体は内部エラーとしてのみ使う）。
pub const DOCUMENT_NOT_FOUND_MESSAGE: &str = "Document not found";

/// 設定とその revision を同じロックで保持する（CAS の比較単位）。
struct ConfigState {
    config: AppConfig,
    revision: u64,
    /// 保存中のコミットがあるか。write guard を保存の外で解放するため、
    /// 同一 expected の二重通過を防ぐ予約フラグとして使う。
    pending: bool,
    /// 起動時に config.toml を読み込めなかったか。true の間は保存を拒否する。
    config_load_failed: bool,
}

/// Application state shared across commands
pub struct AppState {
    config: RwLock<ConfigState>,
    documents: RwLock<HashMap<DocumentId, Document>>,
    ai_service: RwLock<Option<Arc<AiMarkdownService>>>,
    unsaved_changes: RwLock<HashMap<DocumentId, bool>>,
    /// 文書 id 単位の世代カウンタ。本文を更新するたびに前進し、保存 CAS の
    /// 比較単位として使う。`documents` と同じ順（documents -> generations）
    /// でロックを取得すること。文書を閉じても削除しない（再オープン時に
    /// 古い expected が一致するのを防ぐため、世代は単調に増やし続ける）。
    document_generations: RwLock<HashMap<DocumentId, u64>>,
    /// `prepare_ai_service` の完了直後に呼ぶテスト専用フック。
    /// await を跨いで `commit_config` が入った状況を外部 AI API なしで再現する。
    /// インスタンスごとに持つため、並列テスト間で干渉しない。
    #[cfg(test)]
    test_after_prepare: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
    /// `close_document_atomic` が documents の write guard を取得した直後に呼ぶ
    /// テスト専用フック。unsaved チェックの直前に `mark_unsaved` が割り込む
    /// 状況を疑似的に再現する（実運用では documents guard がブロックする）。
    #[cfg(test)]
    test_after_documents_lock: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
    /// `save_document_impl` が `atomic_write().await` を終えた直後に呼ぶ
    /// テスト専用フック。await 中に別編集が割り込んだ状況を外部ファイル
    /// イベントなしで再現する。インスタンスごとに持つため並列テスト間で
    /// 干渉しない。
    #[cfg(test)]
    test_after_atomic_write: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
    /// `save_document_impl` が保存コミット直前に呼ぶテスト専用フック。
    /// 再確認後〜コミット前に force close が割り込む状況を再現する。
    #[cfg(test)]
    test_before_commit_saved: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
    /// `update_document_content_atomic` が本文更新後・unsaved 設定前に呼ぶ
    /// テスト専用フック。全 guard を保持している間に close が割り込めない
    /// ことを再現する。インスタンスごとに持つため並列テスト間で干渉しない。
    #[cfg(test)]
    test_after_content_update: std::sync::Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
}

impl AppState {
    /// Create new application state（正常起動用）
    pub fn new(config: AppConfig) -> Self {
        Self::with_config_load_failed(config, false)
    }

    /// 設定読み込み失敗状態で状態を生成する。
    ///
    /// 既定値をメモリ上に保持するが、読み込み失敗状態が続く限り
    /// `commit_config` は fail-closed で保存を拒否する（R2-ER01）。
    pub fn new_with_load_failure(config: AppConfig) -> Self {
        Self::with_config_load_failed(config, true)
    }

    fn with_config_load_failed(config: AppConfig, config_load_failed: bool) -> Self {
        Self {
            config: RwLock::new(ConfigState {
                config,
                revision: 0,
                pending: false,
                config_load_failed,
            }),
            documents: RwLock::new(HashMap::new()),
            ai_service: RwLock::new(None),
            unsaved_changes: RwLock::new(HashMap::new()),
            document_generations: RwLock::new(HashMap::new()),
            #[cfg(test)]
            test_after_prepare: std::sync::Mutex::new(None),
            #[cfg(test)]
            test_after_documents_lock: std::sync::Mutex::new(None),
            #[cfg(test)]
            test_after_atomic_write: std::sync::Mutex::new(None),
            #[cfg(test)]
            test_before_commit_saved: std::sync::Mutex::new(None),
            #[cfg(test)]
            test_after_content_update: std::sync::Mutex::new(None),
        }
    }

    /// 起動時に config.toml の読み込みに失敗したか。
    pub fn config_load_failed(&self) -> bool {
        self.config.read().unwrap().config_load_failed
    }

    /// Get current configuration
    pub fn config(&self) -> AppConfig {
        self.config.read().unwrap().config.clone()
    }

    /// 現在の `LintConfig` のクローンを返す。
    ///
    /// Lint はローカル処理（`MarkdownLinter`）であり AI service を必要としないため、
    /// AI 無効時でも設定を反映して実行できるようにする（返り値は clone なので
    /// 呼び出し側が書き換えても state は変化しない）。
    pub fn lint_config(&self) -> LintConfig {
        self.config.read().unwrap().config.lint.clone()
    }

    /// 現在の設定と revision を同一ロック・同一時点で取得する。
    ///
    /// 設定コマンドはここで得た `expected` を後続の `commit_config` へ渡す。
    pub fn config_with_revision(&self) -> (AppConfig, u64) {
        let state = self.config.read().unwrap();
        (state.config.clone(), state.revision)
    }

    /// 低レベル更新（revision を進めない）。既存のテスト用に残す。
    ///
    /// 設定コマンドの保存経路には使わず、CAS を含む `commit_config` を使うこと。
    #[cfg(test)]
    pub fn update_config(&self, config: AppConfig) -> CoreResult<()> {
        self.config.write().unwrap().config = config;
        Ok(())
    }

    /// 候補設定から AI service を準備する（状態を一切変更しない）。
    ///
    /// 無効設定なら `Ok(None)`、有効設定なら生成結果を `Ok(Some(...))` で返す。
    /// 生成失敗時は `Err` を返すだけなので、config / ディスク / service は不変。
    /// ロックは保持せず、生成は呼び出し側のコミット前に完了する。
    pub async fn prepare_ai_service(
        &self,
        config: &AppConfig,
    ) -> CoreResult<Option<Arc<AiMarkdownService>>> {
        if !config.ai.enabled {
            return Ok(None);
        }
        let service = AiMarkdownService::new(config).await?;
        Ok(Some(Arc::new(service)))
    }

    /// 設定と AI service を revision CAS でまとめてコミットする。
    ///
    /// `expected` が現在の revision と不一致なら `CoreError::Conflict` を返し、
    /// config / disk / service を一切変更しない。write guard は比較と予約だけの
    /// 短いクリティカルセクションで解放し、ディスク保存はロックの外で行う
    /// （`save_config()` は read guard を取るため、ここでは候補を直接保存する）。
    pub fn commit_config(
        &self,
        expected: u64,
        config: AppConfig,
        service: Option<Arc<AiMarkdownService>>,
    ) -> CoreResult<()> {
        // --- 短い write クリティカルセクション: 失敗状態確認・比較・予約のみ ---
        let next_revision = {
            let mut state = self.config.write().unwrap();
            // 起動時に config.toml を読み込めていない場合は、既定値での上書きを
            // 防ぐため一切保存しない（固定文言のみ返し、設定値は出さない）。
            if state.config_load_failed {
                return Err(CoreError::storage(CONFIG_LOAD_FAILED_MESSAGE));
            }
            // overflow は保存前に失敗させる（ADR 未決定のため fail-closed）。
            let next = expected
                .checked_add(1)
                .ok_or_else(|| CoreError::internal("configuration revision overflow"))?;
            if state.revision != expected || state.pending {
                return Err(CoreError::Conflict(
                    "Configuration changed since it was read".to_string(),
                ));
            }
            state.pending = true;
            next
        };

        // --- ロックを解放してから保存（await も guard 保持もしない） ---
        if let Err(save_error) = config.save() {
            // メモリはまだ変更していないため、旧 config / revision はそのまま。
            // 予約を保持したまま旧設定を読み、ディスクを再保存する（他コミットの
            // 割込みを防ぐ）。2 次失敗あり得る。
            let previous = self.config.read().unwrap().config.clone();
            if let Err(restore_error) = previous.save() {
                self.config.write().unwrap().pending = false;
                // エラーの表示文字列だけを連結し、設定値（シークレット含む）は出さない。
                return Err(CoreError::internal(format!(
                    "config save failed and rollback re-save also failed: {save_error}; {restore_error}"
                )));
            }
            self.config.write().unwrap().pending = false;
            return Err(save_error);
        }

        // --- 成功時のみ、短い guard で確定（revision 前進・service 置換） ---
        // ロックは config -> ai_service の順で取得し（逆順の箇所は無い）、
        // 設定と service を不可分に反映する。
        let mut state = self.config.write().unwrap();
        state.config = config;
        state.revision = next_revision;
        state.pending = false;
        *self.ai_service.write().unwrap() = service;
        Ok(())
    }

    /// Initialize AI service（起動時）。fail-closed の refresh 処理を共用する。
    pub async fn init_ai_service(&self) -> CoreResult<()> {
        self.refresh_ai_service().await
    }

    /// Get AI service if available
    ///
    /// 設定で無効な場合は service の有無に関わらず None を返す。
    /// ロックは逐次取得して重ねない（デッドロック回避）。
    pub fn ai_service(&self) -> Option<Arc<AiMarkdownService>> {
        if !self.is_ai_enabled() {
            return None;
        }
        self.ai_service.read().unwrap().clone()
    }

    /// Check if AI features are enabled
    pub fn is_ai_enabled(&self) -> bool {
        self.config.read().unwrap().config.ai.enabled
    }

    /// Clear the in-memory AI service (設定無効化時に呼ぶ)
    pub fn disable_ai_service(&self) {
        *self.ai_service.write().unwrap() = None;
    }

    /// 設定変更後に AI service を同期する（fail-closed）。
    ///
    /// 生成中は旧 service を残し、完了時に開始時点の revision を再確認する。
    /// await 中に `commit_config` が新しい config / service を確定していた場合は
    /// 生成結果を捨てて何もしない。生成失敗時も競合が無ければ service をクリアし、
    /// 「古い service は残らない」という fail-closed の契約を維持する。
    pub async fn refresh_ai_service(&self) -> CoreResult<()> {
        // 設定と revision を同じロック・同じ時点で読み、比較基準を固定する。
        let (config, expected) = self.config_with_revision();
        let prepared = self.prepare_ai_service(&config).await;

        #[cfg(test)]
        {
            // 1 回限りで解除し、callback を呼ぶ前に MutexGuard を drop する。
            let hook = self.test_after_prepare.lock().unwrap().take();
            if let Some(hook) = hook {
                hook();
            }
        }

        match prepared {
            Ok(service) => {
                // read() で確認してから write() を取り直すと、その間に
                // commit_config が入り得る（TOCTOU）。write guard を保持した
                // まま revision を比較し、service 置換まで解放しない。
                // ロック順は commit_config と同じ config -> ai_service。
                // clippy は revision を読むだけと判定するが、書き込まないこと
                // 自体が CAS を壊さないための要件（意図的な排他）。
                #[allow(clippy::readonly_write_lock)]
                let state = self.config.write().unwrap();
                if state.revision != expected {
                    // 古い config から生成した service は捨て、commit 側を守る。
                    return Ok(());
                }
                // state（config の write guard）はこのスコープ末尾まで保持される。
                match service {
                    Some(service) => *self.ai_service.write().unwrap() = Some(service),
                    // 無効設定なら disable と同義。生成前ではなく確認後に呼ぶ。
                    None => self.disable_ai_service(),
                }
                Ok(())
            }
            Err(error) => {
                // 上と同じく、確認から disable までを commit_config と排他する。
                #[allow(clippy::readonly_write_lock)]
                let state = self.config.write().unwrap();
                if state.revision == expected {
                    // 競合が無ければ旧 service を残さない（fail-closed）。
                    // 競合していれば commit_config の service を守る。
                    self.disable_ai_service();
                }
                Err(error)
            }
        }
    }

    /// Add a document
    ///
    /// 文書を登録・差し替える経路（open / set_document_content 等）はここを
    /// 通るため、登録ごとに世代を前進させる。成功した保存コミットは
    /// `commit_saved_document` が世代を進めるため、ここは通らない。
    /// ロック順は `documents` -> `document_generations` で、他の経路と同じにする。
    pub fn add_document(&self, document: Document) {
        let id = document.metadata.id.clone();
        let mut documents = self.documents.write().unwrap();
        let mut generations = self.document_generations.write().unwrap();
        documents.insert(id.clone(), document);
        // 世代は単調に増やす。overflow しても値が戻って古い expected と
        // 一致しないよう saturating で留める（実用上到達しないが fail-closed）。
        let next = generations.get(&id).copied().unwrap_or(0).saturating_add(1);
        generations.insert(id, next);
    }

    /// 文書本文の更新・世代前進・unsaved=true を単一クリティカルセクションで行う。
    ///
    /// `documents` -> `document_generations` -> `unsaved_changes` の順で
    /// write guard を同時に保持し、本文の更新から unsaved 設定までを排他する
    /// （逆順の箇所は作らない）。旧来の `get_document` -> `add_document` ->
    /// `mark_unsaved` では `add_document` が documents guard を解放した隙間に
    /// close が走り、編集が失われる／孤立フラグが残る問題があった。
    ///
    /// 文書が存在しない場合は固定文言の `DocumentNotFound` を返し、何も変更
    /// しない（既に閉じられた文書を復活させない）。本文が編集不能な場合も
    /// 固定文言の Validation を返し、世代・unsaved を一切変更しない。
    /// 成功時は更新後の文書を返す（emit は guard 解放後に呼び出し側が行う）。
    pub fn update_document_content_atomic(
        &self,
        id: &DocumentId,
        content: String,
    ) -> CoreResult<Document> {
        let mut documents = self.documents.write().unwrap();
        let mut generations = self.document_generations.write().unwrap();
        let mut unsaved = self.unsaved_changes.write().unwrap();

        let document = documents
            .get_mut(id)
            .ok_or_else(|| CoreError::DocumentNotFound(DOCUMENT_NOT_FOUND_MESSAGE.to_string()))?;
        let text = document
            .as_text_mut()
            .ok_or_else(|| CoreError::validation("Document is not text-editable"))?;
        *text = content;

        #[cfg(test)]
        {
            // guard を保持したまま close の割込みを試すための 1 回限りフック。
            // callback を呼ぶ前に MutexGuard を drop する（再取得で死結しない）。
            let hook = self.test_after_content_update.lock().unwrap().take();
            if let Some(hook) = hook {
                hook();
            }
        }

        // 世代は単調に増やす。overflow しても値が戻って古い expected と
        // 一致しないよう saturating で留める（add_document と同じ契約）。
        let next = generations.get(id).copied().unwrap_or(0).saturating_add(1);
        generations.insert(id.clone(), next);
        unsaved.insert(id.clone(), true);

        Ok(document.clone())
    }

    /// Get a document
    pub fn get_document(&self, id: &DocumentId) -> Option<Document> {
        self.documents.read().unwrap().get(id).cloned()
    }

    /// 現在の文書と generation を同じロック・同じ時点で取得する。
    ///
    /// 保存コマンドはここで得た `expected` を後続の `commit_saved_document` へ
    /// 渡す。文書と世代を別々に読むとその間に編集が入り得るため、必ず両方を
    /// 同じクリティカルセクションで読む。
    pub fn document_with_generation(&self, id: &DocumentId) -> Option<(Document, u64)> {
        let documents = self.documents.read().unwrap();
        let generations = self.document_generations.read().unwrap();
        let document = documents.get(id)?.clone();
        let generation = generations.get(id).copied().unwrap_or(0);
        Some((document, generation))
    }

    /// 現在の generation を返す。文書が未登録なら `None`。
    #[cfg(test)]
    pub fn document_generation(&self, id: &DocumentId) -> Option<u64> {
        self.document_generations.read().unwrap().get(id).copied()
    }

    /// 保存コミット（CAS）。
    ///
    /// `expected` が現在の generation と一致する場合のみ、文書を `document`
    /// で置き換えて generation を進め、unsaved をクリアする。不一致なら
    /// 固定文言の `CoreError::Conflict` を返し、文書・世代・unsaved の
    /// いずれも変更しない。比較から確定まで `documents` の write guard を
    /// 保持するため、await 後の再確認とコミットの間に編集が入っても
    /// 古い snapshot が書き戻されることはない。
    ///
    /// 文書が既に閉じられている場合も Conflict とし、古い保存が
    /// 削除済み文書を復活させない（close 側も世代を前進させる）。
    pub fn commit_saved_document(
        &self,
        id: &DocumentId,
        expected: u64,
        document: Document,
    ) -> CoreResult<()> {
        // documents -> document_generations -> unsaved_changes の順で取得する。
        let mut documents = self.documents.write().unwrap();
        let mut generations = self.document_generations.write().unwrap();
        // 閉じられた文書は復活させない。generation が一致していても
        // documents に無ければ競合として扱う（force close の隔挿入対策）。
        if !documents.contains_key(id) {
            return Err(CoreError::Conflict(
                DOCUMENT_SAVE_CONFLICT_MESSAGE.to_string(),
            ));
        }
        let current = generations.get(id).copied().unwrap_or(0);
        if current != expected {
            return Err(CoreError::Conflict(
                DOCUMENT_SAVE_CONFLICT_MESSAGE.to_string(),
            ));
        }
        let next = expected
            .checked_add(1)
            .ok_or_else(|| CoreError::internal("document generation overflow"))?;
        documents.insert(id.clone(), document);
        generations.insert(id.clone(), next);
        // unsaved も同じクリティカルセクションで更新し、コミット直後の
        // 編集との取り残しを防ぐ。
        self.unsaved_changes
            .write()
            .unwrap()
            .insert(id.clone(), false);
        Ok(())
    }

    /// 本文更新直後に任意の処理を挟む（テスト専用フック）。
    ///
    /// 全 guard を保持したまま close コマンドを割り込ませ、unsaved 拒否を
    /// 検証するコマンド側テストからも使う。
    #[cfg(test)]
    pub(crate) fn set_after_content_update_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.test_after_content_update.lock().unwrap() = Some(Box::new(hook));
    }

    /// atomic_write 完了直後に任意の処理を挟む（テスト専用フック）。
    #[cfg(test)]
    pub fn set_after_atomic_write_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.test_after_atomic_write.lock().unwrap() = Some(Box::new(hook));
    }

    /// atomic_write 完了直後のテストフックを 1 回だけ実行する。
    #[cfg(test)]
    pub(crate) fn invoke_after_atomic_write_hook(&self) {
        let hook = self.test_after_atomic_write.lock().unwrap().take();
        if let Some(hook) = hook {
            hook();
        }
    }

    /// 保存コミット直前に任意の処理を挟む（テスト専用フック）。
    #[cfg(test)]
    pub fn set_before_commit_saved_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.test_before_commit_saved.lock().unwrap() = Some(Box::new(hook));
    }

    /// 保存コミット直前のテストフックを 1 回だけ実行する。
    #[cfg(test)]
    pub(crate) fn invoke_before_commit_saved_hook(&self) {
        let hook = self.test_before_commit_saved.lock().unwrap().take();
        if let Some(hook) = hook {
            hook();
        }
    }

    /// Remove a document
    ///
    /// documents と unsaved_changes のロックは順番に取得・解放する（同時保持しない）。
    /// 文書が存在しない場合でも unsaved キーを削除し、孤立フラグを掃除する。
    ///
    /// `close_document_atomic` の導入で production の呼び出しは無くなったが、
    /// ADR-0208 の削除契約を残すため維持する（unit test から使用）。
    #[allow(dead_code)]
    pub fn remove_document(&self, id: &DocumentId) -> Option<Document> {
        let removed = self.documents.write().unwrap().remove(id);
        self.unsaved_changes.write().unwrap().remove(id);
        removed
    }

    /// 未保存チェックと文書削除を単一クリティカルセクションで行う。
    ///
    /// `documents` -> `document_generations` -> `unsaved_changes` の順で
    /// write guard を同時に保持し、チェックから削除までを排他する
    /// （逆順の箇所は作らない）。文書を削除できた場合は世代も前進させ、
    /// 削除前に開始した保存が古い expected で復活しないようにする。
    /// `force=false` で unsaved なら固定文言の Validation を返し、何も変更しない。
    /// `force=true` なら削除する。文書が存在しない場合も `unsaved_changes` の
    /// キーを削除し、孤立フラグを掃除する既存契約を維持する（ADR-0208）。
    pub fn close_document_atomic(
        &self,
        id: &DocumentId,
        force: bool,
    ) -> CoreResult<Option<Document>> {
        // 既存の remove_document と同じく documents を先に取得する。
        let mut documents = self.documents.write().unwrap();

        #[cfg(test)]
        {
            // 疑似的な隔挿入: documents guard を保持したまま mark_unsaved を
            // 割り込ませ、チェック時点の最新フラグで拒否されることを確認する。
            // 実運用の set_document_content は documents を経由するためここで
            // ブロックされるが、unsaved_changes 側からの直接割込みを模する。
            // guard は take() で 1 回限りにし、callback を呼ぶ前に drop する
            // （callback が Mutex を再取得してもデッドロックしないように）。
            let hook = self.test_after_documents_lock.lock().unwrap().take();
            if let Some(hook) = hook {
                hook();
            }
        }

        // 世代も同じクリティカルセクションで保持する。
        // documents -> document_generations -> unsaved_changes の順を維持し、
        // 逆順の箇所は作らない。
        let mut generations = self.document_generations.write().unwrap();
        let mut unsaved = self.unsaved_changes.write().unwrap();

        if !force && unsaved.get(id).copied().unwrap_or(false) {
            return Err(CoreError::validation(DOCUMENT_UNSAVED_MESSAGE));
        }

        let removed = documents.remove(id);
        if removed.is_some() {
            // 削除前に開始した保存の CAS が一致しないよう世代を前進させる。
            let next = generations.get(id).copied().unwrap_or(0).saturating_add(1);
            generations.insert(id.clone(), next);
        }
        // 文書が存在しない場合も孤立フラグを掃除する（既存契約）。
        unsaved.remove(id);
        Ok(removed)
    }

    /// List all documents
    pub fn list_documents(&self) -> Vec<Document> {
        self.documents.read().unwrap().values().cloned().collect()
    }

    /// Mark document as having unsaved changes
    ///
    /// 編集コマンドは `update_document_content_atomic` 経由で unsaved を立てる
    /// ため production からは呼ばれないが、状態確認 API として維持する
    /// （unit test および close の割込み再現から使用）。
    #[allow(dead_code)]
    pub fn mark_unsaved(&self, id: &DocumentId) {
        self.unsaved_changes
            .write()
            .unwrap()
            .insert(id.clone(), true);
    }

    /// Mark document as saved
    pub fn mark_saved(&self, id: &DocumentId) {
        self.unsaved_changes
            .write()
            .unwrap()
            .insert(id.clone(), false);
    }

    /// Check if document has unsaved changes
    ///
    /// `close_document_atomic` が両 guard 保持下で同じ判定を行うため production
    /// では未使用だが、状態確認 API として維持する（unit test から使用）。
    #[allow(dead_code)]
    pub fn is_unsaved(&self, id: &DocumentId) -> bool {
        self.unsaved_changes
            .read()
            .unwrap()
            .get(id)
            .copied()
            .unwrap_or(false)
    }

    /// Check if any document has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.unsaved_changes.read().unwrap().values().any(|&v| v)
    }

    /// Save configuration to disk（テスト用。commit は再ロック回避のため直接 save する）
    #[cfg(test)]
    pub fn save_config(&self) -> CoreResult<()> {
        self.config.read().unwrap().config.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openrustbooks_core::document::{DocumentContent, DocumentFormat, DocumentMetadata};
    use std::path::PathBuf;

    /// config.toml を触るテストは同じパスを共有するため直列化する。
    /// async テストは guard を await を跨いで保持するため tokio の Mutex を使う。
    static CONFIG_FILE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    /// 同期テスト用（tokio runtime 外で呼ぶこと）。
    fn lock_config_file_blocking() -> tokio::sync::MutexGuard<'static, ()> {
        CONFIG_FILE_TEST_LOCK.blocking_lock()
    }

    fn saved_config_toml() -> Option<Vec<u8>> {
        directories::ProjectDirs::from("com", "openrustbooks", "openrustbooks")
            .and_then(|dirs| std::fs::read(dirs.config_dir().join("config.toml")).ok())
    }

    /// commit_config が触れる config.toml をテスト前後で保護する。
    /// 既存ファイルは内容を復元し、無ければ削除して環境を汚さない。
    struct ConfigFileGuard {
        path: Option<PathBuf>,
        original: Option<Vec<u8>>,
    }

    impl ConfigFileGuard {
        fn new() -> Self {
            let path = directories::ProjectDirs::from("com", "openrustbooks", "openrustbooks")
                .map(|dirs| dirs.config_dir().join("config.toml"));
            let original = path.as_ref().and_then(|p| std::fs::read(p).ok());
            Self { path, original }
        }
    }

    impl Drop for ConfigFileGuard {
        fn drop(&mut self) {
            let Some(path) = &self.path else {
                return;
            };
            match &self.original {
                Some(bytes) => {
                    let _ = std::fs::write(path, bytes);
                }
                None => {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }

    /// AppConfig は PartialEq 未 derive のため JSON 表現で比較する。
    fn assert_config_eq(actual: &AppConfig, expected: &AppConfig) {
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }

    fn sample_document(id: &str) -> Document {
        Document {
            metadata: DocumentMetadata::new(
                DocumentId::from_string(id.to_string()),
                DocumentFormat::Txt,
            ),
            content: DocumentContent::Text("sample".to_string()),
        }
    }

    impl AppState {
        /// prepare 完了直後に任意の処理を挟む（テスト専用フック）。
        fn set_after_prepare_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
            *self.test_after_prepare.lock().unwrap() = Some(Box::new(hook));
        }

        /// close が documents guard を取得した直後に任意の処理を挟む（テスト専用フック）。
        fn set_after_documents_lock_hook(&self, hook: impl Fn() + Send + Sync + 'static) {
            *self.test_after_documents_lock.lock().unwrap() = Some(Box::new(hook));
        }
    }

    /// 有効設定と ollama の service を revision を進めてコミットする。
    ///
    /// ollama は初期化時に HTTP 要求を出さないため、外部 AI API を使わない。
    async fn commit_enabled_service(
        state: &AppState,
        theme: &str,
    ) -> (AppConfig, Arc<AiMarkdownService>) {
        let (mut config, expected) = state.config_with_revision();
        config.ai.enabled = true;
        config.ai.default_provider = "ollama".to_string();
        config.general.theme = theme.to_string();
        let service = state
            .prepare_ai_service(&config)
            .await
            .unwrap()
            .expect("ollama 設定なら service が生成される");
        state
            .commit_config(expected, config.clone(), Some(service.clone()))
            .unwrap();
        (config, service)
    }

    #[test]
    fn unsaved_changes_state_transitions() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-1");
        let id = document.metadata.id.clone();

        assert!(!state.has_unsaved_changes());

        state.add_document(document);
        assert!(state.get_document(&id).is_some());
        assert!(!state.is_unsaved(&id));

        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));
        assert!(state.has_unsaved_changes());

        state.mark_saved(&id);
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());

        assert!(state.remove_document(&id).is_some());
        assert!(state.get_document(&id).is_none());
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn remove_document_clears_unsaved_flag() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-remove-unsaved");
        let id = document.metadata.id.clone();

        state.add_document(document);
        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));
        assert!(state.has_unsaved_changes());

        // force close 相当: mark_saved を挟まずに削除してもフラグは消える。
        assert!(state.remove_document(&id).is_some());
        assert!(state.get_document(&id).is_none());
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn remove_document_clears_orphaned_unsaved_flag() {
        let state = AppState::new(AppConfig::default());
        let orphan = sample_document("doc-orphan-unsaved");
        let id = orphan.metadata.id.clone();

        // documents に登録せずに unsaved フラグだけ立てる（孤立フラグ）。
        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));
        assert!(state.has_unsaved_changes());

        // 文書が無くても remove_document は孤立フラグを掃除する。
        assert!(state.remove_document(&id).is_none());
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    /// force=false の unsaved 拒否は固定文言の Validation で、状態は一切変化しない。
    #[test]
    fn close_document_atomic_rejects_unsaved_without_force() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-atomic-unsaved");
        let id = document.metadata.id.clone();
        state.add_document(document);
        state.mark_unsaved(&id);

        let error = state
            .close_document_atomic(&id, false)
            .expect_err("unsaved 文書は force なしで拒否される");
        match error {
            CoreError::Validation(message) => assert_eq!(message, DOCUMENT_UNSAVED_MESSAGE),
            other => panic!("expected Validation, got {other:?}"),
        }
        assert!(state.get_document(&id).is_some(), "文書は削除されない");
        assert!(state.is_unsaved(&id), "フラグも残る");
        assert!(state.has_unsaved_changes());
    }

    /// add_document は文書ごとに世代を前進させ、文書と世代を同時に読めること。
    #[test]
    fn add_document_advances_generation_and_exposes_consistent_pair() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-generation-basic".to_string());
        assert_eq!(state.document_generation(&id), None);
        assert!(state.document_with_generation(&id).is_none());

        state.add_document(sample_document("doc-generation-basic"));
        let (document, first) = state.document_with_generation(&id).unwrap();
        assert_eq!(document.as_text(), Some("sample"));
        assert_eq!(first, 1);

        let mut edited = document;
        *edited.as_text_mut().unwrap() = "second".to_string();
        state.add_document(edited);
        let (document, second) = state.document_with_generation(&id).unwrap();
        assert_eq!(document.as_text(), Some("second"));
        assert_eq!(second, first + 1);
        assert_eq!(state.document_generation(&id), Some(second));
    }

    /// 成功時のみ文書・世代・unsaved が更新されること。
    #[test]
    fn commit_saved_document_success_updates_document_and_generation() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-save-cas-ok".to_string());
        state.add_document(sample_document("doc-save-cas-ok"));
        state.mark_unsaved(&id);
        let (_, expected) = state.document_with_generation(&id).unwrap();

        let mut saved = sample_document("doc-save-cas-ok");
        *saved.as_text_mut().unwrap() = "saved".to_string();
        saved.metadata.size_bytes = 5;
        state
            .commit_saved_document(&id, expected, saved)
            .expect("期待世代が一致すればコミットできる");

        let (current, generation) = state.document_with_generation(&id).unwrap();
        assert_eq!(current.as_text(), Some("saved"));
        assert_eq!(current.metadata.size_bytes, 5);
        assert_eq!(generation, expected + 1);
        assert!(!state.is_unsaved(&id));
    }

    /// 古い expected でのコミットは固定文言の Conflict になり、変更がないこと。
    #[test]
    fn commit_saved_document_stale_generation_conflicts_without_changes() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-save-cas-conflict".to_string());
        state.add_document(sample_document("doc-save-cas-conflict"));
        let (snapshot, expected) = state.document_with_generation(&id).unwrap();

        // 保存中の並行編集を再現（add_document が世代を前進させる）。
        let mut edited = snapshot.clone();
        *edited.as_text_mut().unwrap() = "edited".to_string();
        state.add_document(edited);
        state.mark_unsaved(&id);
        let (_, after_edit) = state.document_with_generation(&id).unwrap();

        let mut stale = snapshot;
        *stale.as_text_mut().unwrap() = "stale".to_string();
        let error = state
            .commit_saved_document(&id, expected, stale)
            .expect_err("古い世代は競合する");
        match error {
            CoreError::Conflict(message) => {
                assert_eq!(message, DOCUMENT_SAVE_CONFLICT_MESSAGE);
            }
            other => panic!("expected Conflict, got {other:?}"),
        }

        let (current, generation) = state.document_with_generation(&id).unwrap();
        assert_eq!(current.as_text(), Some("edited"), "最新本文が残る");
        assert_eq!(generation, after_edit, "競合で世代は進まない");
        assert!(state.is_unsaved(&id), "競合で unsaved は残る");
    }

    /// close 後に古い expected でコミットしても文書が復活しないこと。
    #[test]
    fn commit_saved_document_after_close_conflicts_without_resurrection() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-save-cas-closed".to_string());
        state.add_document(sample_document("doc-save-cas-closed"));
        let (snapshot, expected) = state.document_with_generation(&id).unwrap();

        state.close_document_atomic(&id, true).unwrap();
        assert!(state.get_document(&id).is_none());
        let closed_generation = state.document_generation(&id).unwrap();
        assert_eq!(closed_generation, expected + 1, "close で世代が前進する");

        let error = state
            .commit_saved_document(&id, expected, snapshot)
            .expect_err("閉じた文書への古いコミットは競合する");
        assert!(matches!(error, CoreError::Conflict(_)));
        assert!(state.get_document(&id).is_none(), "文書は復活しない");
        assert_eq!(state.document_generation(&id), Some(closed_generation));
    }

    /// 保存済み文書は force=false で削除され、unsaved キーも掃除される（通常系の維持）。
    #[test]
    fn close_document_atomic_removes_saved_document() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-atomic-saved");
        let id = document.metadata.id.clone();
        state.add_document(document);
        state.mark_saved(&id);

        let removed = state
            .close_document_atomic(&id, false)
            .expect("保存済みは閉じられる");
        assert!(removed.is_some());
        assert!(state.get_document(&id).is_none());
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    /// force=true なら unsaved でも削除され、フラグも消える（既存契約の維持）。
    #[test]
    fn close_document_atomic_force_removes_unsaved_document() {
        let state = AppState::new(AppConfig::default());
        let document = sample_document("doc-atomic-force");
        let id = document.metadata.id.clone();
        state.add_document(document);
        state.mark_unsaved(&id);

        let removed = state
            .close_document_atomic(&id, true)
            .expect("force なら削除される");
        assert!(removed.is_some());
        assert!(state.get_document(&id).is_none());
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    /// 疑似注入 test: documents guard 取得直後に mark_unsaved を直接呼び
    /// （別スレッド・barrier を使わず同スレッドで隔挿入を模する）、
    /// チェックが最新フラグを読んで拒否し文書が削除されないことを確認する。
    /// 実スレッドによる回帰 test は
    /// `close_document_atomic_does_not_lose_edit_injected_between_legacy_check_and_remove` を参照。
    #[test]
    fn close_document_atomic_rejects_when_mark_unsaved_interleaves_after_documents_lock() {
        let state = Arc::new(AppState::new(AppConfig::default()));
        let document = sample_document("doc-atomic-interleave");
        let id = document.metadata.id.clone();
        state.add_document(document);
        assert!(!state.is_unsaved(&id));

        let state_for_hook = state.clone();
        let id_for_hook = id.clone();
        state.set_after_documents_lock_hook(move || {
            state_for_hook.mark_unsaved(&id_for_hook);
        });

        let error = state
            .close_document_atomic(&id, false)
            .expect_err("割り込んだ未保存編集は拒否される");
        match error {
            CoreError::Validation(message) => assert_eq!(message, DOCUMENT_UNSAVED_MESSAGE),
            other => panic!("expected Validation, got {other:?}"),
        }
        assert!(state.get_document(&id).is_some(), "文書は削除されない");
        assert!(state.is_unsaved(&id), "割り込んだフラグは残る");
    }

    /// 旧 2 段階手順（is_unsaved() → remove_document()）の
    /// 「unsaved 検査後・削除前」と同じ隔挿入点に別スレッドの mark_unsaved を
    /// 入れても、force=false で未保存編集が失われないことの回帰 test。
    ///
    /// barrier で両者を同じ時点に揃え、隔挿入スレッドの完了を待ってから
    /// atomic なチェックが走るようにする（決定論的）。旧実装では検査を
    /// 通過した直後にフラグが立ち、その後の削除で編集が黙って失われた。
    #[test]
    fn close_document_atomic_does_not_lose_edit_injected_between_legacy_check_and_remove() {
        let state = Arc::new(AppState::new(AppConfig::default()));
        let document = sample_document("doc-legacy-race");
        let id = document.metadata.id.clone();
        state.add_document(document);

        // 旧手順の is_unsaved() に相当: 検査時点では未保存ではない。
        assert!(!state.is_unsaved(&id));

        let barrier = Arc::new(std::sync::Barrier::new(2));
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        // Receiver は Sync ではないため、フックの Sync 要件を満たす Mutex で包む。
        let done_rx = Arc::new(std::sync::Mutex::new(done_rx));

        // set_document_content 相当の別スレッド: close が documents guard を
        // 取得した直後（旧手順の検査後・削除前と同じ隔挿入点）で unsaved を立てる。
        let writer_state = state.clone();
        let writer_id = id.clone();
        let writer_barrier = barrier.clone();
        let writer = std::thread::spawn(move || {
            writer_barrier.wait();
            writer_state.mark_unsaved(&writer_id);
            done_tx.send(()).unwrap();
        });

        let hook_barrier = barrier.clone();
        state.set_after_documents_lock_hook(move || {
            hook_barrier.wait();
            // 隔挿入が完了してからチェックが走るように待つ。
            done_rx
                .lock()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("隔挿入スレッドが mark_unsaved を完了すること");
        });

        let error = state
            .close_document_atomic(&id, false)
            .expect_err("割り込んだ未保存編集は拒否される");
        assert!(matches!(error, CoreError::Validation(_)));
        writer.join().unwrap();

        // 未保存編集は失われず、文書もフラグも残る（旧実装ではここで削除された）。
        assert!(state.get_document(&id).is_some());
        assert!(state.is_unsaved(&id));
    }

    /// 文書が存在しなくても force=true なら孤立フラグを掃除する（既存契約）。
    #[test]
    fn close_document_atomic_cleans_orphaned_unsaved_flag() {
        let state = AppState::new(AppConfig::default());
        let orphan = sample_document("doc-atomic-orphan");
        let id = orphan.metadata.id.clone();
        state.mark_unsaved(&id);
        assert!(state.is_unsaved(&id));

        assert!(
            state
                .close_document_atomic(&id, true)
                .expect("force なら孤立フラグを掃除できる")
                .is_none()
        );
        assert!(!state.is_unsaved(&id));
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn config_round_trip_in_memory() {
        let state = AppState::new(AppConfig::default());
        assert!(!state.is_ai_enabled(), "AI must be off by default");
        let mut config = state.config();
        config.ai.enabled = true;
        state.update_config(config).unwrap();
        assert!(state.is_ai_enabled());
        assert!(state.config().ai.enabled);
    }

    /// lint_config は現在の lint 設定を clone で返し、変更は state へ漏れないこと。
    #[test]
    fn lint_config_returns_current_settings_as_clone() {
        let state = AppState::new(AppConfig::default());
        let expected = state.config().lint;
        assert!(expected.enabled);
        assert!(expected.disable_rules.iter().any(|r| r == "MD013"));
        assert!(!expected.warnings_as_errors);

        let mut config = state.config();
        config.lint.warnings_as_errors = true;
        config.lint.enable_rules.push("MD001".to_string());
        state.update_config(config).unwrap();

        let actual = state.lint_config();
        assert!(actual.warnings_as_errors);
        assert_eq!(actual.enable_rules, vec!["MD001".to_string()]);
        // 返り値を書き換えても state の設定は不変（clone であることの確認）。
        let mut mutated = actual;
        mutated.warnings_as_errors = false;
        assert!(state.lint_config().warnings_as_errors);
    }

    #[test]
    fn save_config_writes_current_config() {
        let _test_lock = lock_config_file_blocking();
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        let mut config = state.config();
        config.general.theme = "solarized".to_string();
        state.update_config(config).unwrap();

        state.save_config().unwrap();

        let bytes = saved_config_toml().expect("save_config が config.toml を書くこと");
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("solarized"));
    }

    #[test]
    fn ai_service_is_none_before_enabling() {
        let state = AppState::new(AppConfig::default());
        assert!(!state.is_ai_enabled());
        assert!(state.ai_service().is_none());
    }

    #[test]
    fn ai_service_is_none_when_update_config_disables_ai() {
        let state = AppState::new(AppConfig::default());
        let mut config = state.config();
        config.ai.enabled = true;
        state.update_config(config).unwrap();
        assert!(state.is_ai_enabled());

        // service が未生成でも、無効設定ならガードが None を返すことを確認。
        let mut config = state.config();
        config.ai.enabled = false;
        state.update_config(config).unwrap();
        assert!(!state.is_ai_enabled());
        assert!(state.ai_service().is_none());
    }

    #[test]
    fn disable_ai_service_clears_service() {
        let state = AppState::new(AppConfig::default());
        state.disable_ai_service();
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn refresh_ai_service_disabled_keeps_service_none() {
        let state = AppState::new(AppConfig::default());
        state.refresh_ai_service().await.unwrap();
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn refresh_ai_service_propagates_init_error_and_keeps_service_none() {
        let state = AppState::new(AppConfig::default());
        let mut config = state.config();
        config.ai.enabled = true;
        config.ai.default_provider = "unknown".to_string(); // create_client が即エラー
        state.update_config(config).unwrap();

        // 生成失敗は呼び出し側へ伝播し、service は None のまま（fail-closed）。
        assert!(state.refresh_ai_service().await.is_err());
        assert!(state.ai_service().is_none());
    }

    /// prepare 完了直後に commit_config が入っても、古い service で上書きしない
    /// こと（R2-BV01 の競合修正）。フックにより外部 AI API なしで再現する。
    #[tokio::test]
    async fn refresh_ai_service_discards_stale_service_when_config_is_committed_during_prepare() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = Arc::new(AppState::new(AppConfig::default()));

        let (_, old_service) = commit_enabled_service(&state, "dark").await;
        assert!(Arc::ptr_eq(&state.ai_service().unwrap(), &old_service));

        // refresh が読むのは revision 1 の config。その prepare 完了直後に
        // revision 2 をコミットするフックを仕込む（await 中の割込み相当）。
        let (mut replacement_config, commit_expected) = state.config_with_revision();
        replacement_config.general.theme = "light".to_string();
        let replacement_service = state
            .prepare_ai_service(&replacement_config)
            .await
            .unwrap()
            .expect("置換用 service が生成される");
        let state_for_hook = state.clone();
        let config_for_hook = replacement_config.clone();
        let service_for_hook = replacement_service.clone();
        state.set_after_prepare_hook(move || {
            state_for_hook
                .commit_config(
                    commit_expected,
                    config_for_hook.clone(),
                    Some(service_for_hook.clone()),
                )
                .unwrap();
        });

        // 古い config から生成した service を持っていても、競合を検出して
        // commit 側の config / service を守る。
        state.refresh_ai_service().await.unwrap();

        assert_eq!(state.config_with_revision().1, 2);
        assert_config_eq(&state.config(), &replacement_config);
        let current = state
            .ai_service()
            .expect("競合後も有効設定の service が残る");
        assert!(Arc::ptr_eq(&current, &replacement_service));
        assert!(!Arc::ptr_eq(&current, &old_service));
    }

    /// 生成失敗時も、await 中に commit_config が成功していればその service を
    /// 守ること（fail-closed 契約と競合安全性の両立）。
    #[tokio::test]
    async fn refresh_ai_service_keeps_committed_service_when_prepare_fails_after_commit() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = Arc::new(AppState::new(AppConfig::default()));

        commit_enabled_service(&state, "dark").await;

        // revision 2: 生成失敗する provider をコミット（service は None）。
        let (mut failing, expected) = state.config_with_revision();
        failing.ai.default_provider = "unknown".to_string();
        state.commit_config(expected, failing, None).unwrap();
        assert!(state.is_ai_enabled());
        assert!(state.ai_service().is_none());

        // revision 3: prepare 失敗後に別の commit_config が成功した状況を作る。
        let (mut replacement_config, commit_expected) = state.config_with_revision();
        replacement_config.ai.default_provider = "ollama".to_string();
        let replacement_service = state
            .prepare_ai_service(&replacement_config)
            .await
            .unwrap()
            .expect("置換用 service が生成される");
        let state_for_hook = state.clone();
        let config_for_hook = replacement_config.clone();
        let service_for_hook = replacement_service.clone();
        state.set_after_prepare_hook(move || {
            state_for_hook
                .commit_config(
                    commit_expected,
                    config_for_hook.clone(),
                    Some(service_for_hook.clone()),
                )
                .unwrap();
        });

        // 生成失敗は Err で伝播するが、commit 済み service は破壊しない。
        assert!(state.refresh_ai_service().await.is_err());
        assert_eq!(state.config_with_revision().1, 3);
        assert_config_eq(&state.config(), &replacement_config);
        let current = state
            .ai_service()
            .expect("競合後も commit 済み service が残る");
        assert!(Arc::ptr_eq(&current, &replacement_service));
    }

    /// revision が変わらなければ生成した service で置換される（通常系の維持）。
    #[tokio::test]
    async fn refresh_ai_service_replaces_service_when_revision_is_unchanged() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());

        let (_, old_service) = commit_enabled_service(&state, "dark").await;

        state.refresh_ai_service().await.unwrap();

        let current = state.ai_service().expect("有効設定なら service がある");
        assert!(!Arc::ptr_eq(&current, &old_service));
        assert_eq!(state.config_with_revision().1, 1);
    }

    #[tokio::test]
    async fn prepare_ai_service_failure_keeps_config_and_service_unchanged() {
        let state = AppState::new(AppConfig::default());
        let mut valid = AppConfig::default();
        valid.ai.enabled = true;
        valid.ai.default_provider = "ollama".to_string(); // 初期化時に HTTP 要求なし
        state.update_config(valid.clone()).unwrap();
        state.init_ai_service().await.unwrap();
        let before = state.ai_service();
        assert!(before.is_some());

        let mut invalid = valid.clone();
        invalid.ai.default_provider = "unknown".to_string();

        // 生成失敗は Err のみ。コミット（＝保存）に進まないため disk も不変。
        assert!(state.prepare_ai_service(&invalid).await.is_err());

        assert_config_eq(&state.config(), &valid);
        let after = state.ai_service();
        assert!(after.is_some());
        // 古い service は廃棄も復活もされず、同じ Arc が残る。
        assert!(Arc::ptr_eq(
            before.as_ref().unwrap(),
            after.as_ref().unwrap()
        ));
    }

    #[tokio::test]
    async fn prepare_ai_service_disabled_returns_none() {
        let state = AppState::new(AppConfig::default());
        assert!(
            state
                .prepare_ai_service(&AppConfig::default())
                .await
                .unwrap()
                .is_none()
        );
        assert!(state.ai_service().is_none());
    }

    #[test]
    fn normal_state_is_not_in_load_failure_mode() {
        let state = AppState::new(AppConfig::default());
        assert!(!state.config_load_failed());
    }

    /// 読み込み失敗状態では `commit_config` が保存前に拒否され、
    /// メモリ・ディスク・revision のいずれも変化しないこと（R2-ER01）。
    #[tokio::test]
    async fn commit_config_rejected_when_config_load_failed() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new_with_load_failure(AppConfig::default());
        assert!(state.config_load_failed());

        let (_, expected) = state.config_with_revision();
        let disk_before = saved_config_toml();
        let mut candidate = AppConfig::default();
        candidate.general.theme = "dark".to_string();

        let error = state
            .commit_config(expected, candidate, None)
            .expect_err("読み込み失敗状態では保存が拒否されること");
        match &error {
            CoreError::Storage(message) => {
                assert_eq!(message, CONFIG_LOAD_FAILED_MESSAGE);
            }
            other => panic!("expected Storage, got {other:?}"),
        }
        // 固定文言のみで、設定値やパスを含めないこと。
        assert!(!error.to_string().contains("config.toml"));
        assert!(!error.to_string().contains("dark"));

        assert_config_eq(&state.config(), &AppConfig::default());
        assert_eq!(state.config_with_revision().1, 0);
        assert_eq!(saved_config_toml(), disk_before, "ディスクは不変");
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn commit_config_success_replaces_config_and_service_and_advances_revision() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        assert!(state.ai_service().is_none());
        let (_, expected) = state.config_with_revision();
        assert_eq!(expected, 0);

        let mut candidate = AppConfig::default();
        candidate.ai.enabled = true;
        candidate.ai.default_provider = "ollama".to_string();
        candidate.general.theme = "dark".to_string();
        let prepared = state.prepare_ai_service(&candidate).await.unwrap();
        assert!(prepared.is_some());

        state
            .commit_config(expected, candidate.clone(), prepared.clone())
            .unwrap();

        assert_config_eq(&state.config(), &candidate);
        assert_eq!(
            state.config_with_revision().1,
            1,
            "成功時のみ revision が進む"
        );
        let committed = state.ai_service().unwrap();
        assert!(Arc::ptr_eq(prepared.as_ref().unwrap(), &committed));

        // 無効設定をコミットすると service は None に置換される。
        let (_, expected) = state.config_with_revision();
        let disabled = AppConfig::default();
        let none_service = state.prepare_ai_service(&disabled).await.unwrap();
        assert!(none_service.is_none());
        state
            .commit_config(expected, disabled.clone(), none_service)
            .unwrap();
        assert_config_eq(&state.config(), &disabled);
        assert_eq!(state.config_with_revision().1, 2);
        assert!(state.ai_service().is_none());
    }

    #[tokio::test]
    async fn commit_config_stale_expected_conflicts_and_keeps_everything() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = AppState::new(AppConfig::default());
        let (_, expected) = state.config_with_revision();

        let mut accepted = AppConfig::default();
        accepted.ai.enabled = true;
        accepted.ai.default_provider = "ollama".to_string();
        accepted.general.theme = "dark".to_string();
        let prepared = state.prepare_ai_service(&accepted).await.unwrap();
        state
            .commit_config(expected, accepted.clone(), prepared.clone())
            .unwrap();
        let service_before = state.ai_service().unwrap();
        let disk_before = saved_config_toml();
        assert!(disk_before.is_some());

        // 読み取ったままの古い expected で別設定をコミットしようとする。
        let mut stale = accepted.clone();
        stale.general.theme = "light".to_string();
        let err = state.commit_config(expected, stale, None).unwrap_err();
        match &err {
            CoreError::Conflict(message) => {
                assert_eq!(message, "Configuration changed since it was read");
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
        assert!(
            err.to_string().starts_with("Configuration conflict:"),
            "Conflict の表示は固定文言のみ（設定値を含まない）"
        );

        assert_config_eq(&state.config(), &accepted);
        assert_eq!(
            state.config_with_revision().1,
            1,
            "Conflict で revision は不変"
        );
        assert_eq!(
            saved_config_toml(),
            disk_before,
            "Conflict でディスクは不変"
        );
        let service_after = state.ai_service().unwrap();
        assert!(
            Arc::ptr_eq(&service_before, &service_after),
            "Conflict で service は不変"
        );
    }

    #[tokio::test]
    async fn two_commits_with_same_expected_only_one_succeeds() {
        let _test_lock = CONFIG_FILE_TEST_LOCK.lock().await;
        let _guard = ConfigFileGuard::new();
        let state = Arc::new(AppState::new(AppConfig::default()));
        let (_, expected) = state.config_with_revision();

        async fn commit(
            state: Arc<AppState>,
            expected: u64,
            theme: &'static str,
        ) -> CoreResult<()> {
            let mut candidate = AppConfig::default();
            candidate.general.theme = theme.to_string();
            // 両者が同じ expected を保持したまま await を跨ぐ状況を明示する。
            tokio::task::yield_now().await;
            state.commit_config(expected, candidate, None)
        }

        let (first, second) = tokio::join!(
            commit(state.clone(), expected, "dark"),
            commit(state.clone(), expected, "light"),
        );

        let conflicts = [&first, &second]
            .iter()
            .filter(|result| matches!(result, Err(CoreError::Conflict(_))))
            .count();
        assert_eq!(first.is_ok() as usize + second.is_ok() as usize, 1);
        assert_eq!(conflicts, 1, "古い expected の片方は必ず Conflict");
        assert_eq!(state.config_with_revision().1, 1);
        let winner = if first.is_ok() { "dark" } else { "light" };
        assert_eq!(state.config().general.theme, winner);
    }

    /// 通常更新で本文・世代・unsaved が不可分に更新され、更新後文書が返ること。
    #[test]
    fn update_document_content_atomic_updates_content_generation_and_unsaved() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-atomic-content-ok".to_string());
        state.add_document(sample_document("doc-atomic-content-ok"));
        state.mark_saved(&id);
        let (snapshot, before) = state.document_with_generation(&id).unwrap();

        let updated = state
            .update_document_content_atomic(&id, "edited".to_string())
            .expect("更新は成功する");
        assert_eq!(updated.as_text(), Some("edited"));
        assert_eq!(state.get_document(&id).unwrap().as_text(), Some("edited"));
        assert_eq!(state.document_generation(&id), Some(before + 1));
        assert!(state.is_unsaved(&id));

        // さらに 1 回更新しても世代は単調に前進し、snapshot は変わらない。
        let updated_again = state
            .update_document_content_atomic(&id, "edited twice".to_string())
            .unwrap();
        assert_eq!(updated_again.as_text(), Some("edited twice"));
        assert_eq!(state.document_generation(&id), Some(before + 2));
        assert_eq!(snapshot.as_text(), Some("sample"));
    }

    /// 存在しない文書は固定文言の DocumentNotFound で、状態を一切変えないこと。
    #[test]
    fn update_document_content_atomic_missing_document_returns_fixed_error() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-atomic-content-missing".to_string());

        let error = state
            .update_document_content_atomic(&id, "edited".to_string())
            .expect_err("未登録文書は更新できない");
        match error {
            CoreError::DocumentNotFound(message) => {
                assert_eq!(message, DOCUMENT_NOT_FOUND_MESSAGE);
            }
            other => panic!("expected DocumentNotFound, got {other:?}"),
        }
        assert!(state.get_document(&id).is_none(), "文書を復活させない");
        assert_eq!(state.document_generation(&id), None, "世代も作らない");
        assert!(!state.is_unsaved(&id), "孤立フラグも作らない");
    }

    /// 本文編集不能な文書は固定文言の Validation で、状態を一切変えないこと。
    #[test]
    fn update_document_content_atomic_rejects_non_text_without_changes() {
        let state = AppState::new(AppConfig::default());
        let id = DocumentId::from_string("doc-atomic-content-binary".to_string());
        let document = Document {
            metadata: DocumentMetadata::new(id.clone(), DocumentFormat::Txt),
            content: DocumentContent::Binary(vec![1, 2, 3]),
        };
        state.add_document(document);
        let generation_before = state.document_generation(&id);

        let error = state
            .update_document_content_atomic(&id, "edited".to_string())
            .expect_err("バイナリ文書は本文編集できない");
        match error {
            CoreError::Validation(message) => {
                assert_eq!(message, "Document is not text-editable");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
        assert_eq!(state.document_generation(&id), generation_before);
        assert!(!state.is_unsaved(&id));
    }

    /// 本文更新中（全 guard 保持中）に close が割り込めないこと。
    ///
    /// フックで別スレッドの close を開始させ、guard が解放されるまで完了
    /// しないことを確認してから更新を終える。解放後に再開した close は
    /// unsaved を見て拒否され、編集もフラグも残る。
    #[test]
    fn update_document_content_atomic_does_not_allow_close_to_interleave() {
        let state = Arc::new(AppState::new(AppConfig::default()));
        let id = DocumentId::from_string("doc-atomic-content-close".to_string());
        state.add_document(sample_document("doc-atomic-content-close"));
        state.mark_saved(&id);
        let generation_before = state.document_generation(&id).unwrap();

        let barrier = Arc::new(std::sync::Barrier::new(2));
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let started_rx = Arc::new(std::sync::Mutex::new(started_rx));
        let result_rx = Arc::new(std::sync::Mutex::new(result_rx));

        let closer_state = state.clone();
        let closer_id = id.clone();
        let closer_barrier = barrier.clone();
        let closer = std::thread::spawn(move || {
            closer_barrier.wait();
            started_tx.send(()).unwrap();
            let result = closer_state.close_document_atomic(&closer_id, false);
            result_tx.send(result).unwrap();
        });

        let hook_barrier = barrier.clone();
        let hook_result_rx = result_rx.clone();
        state.set_after_content_update_hook(move || {
            hook_barrier.wait();
            started_rx
                .lock()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("close スレッドが開始すること");
            // guard を保持している間は close が完了しないこと。
            assert!(
                hook_result_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(std::time::Duration::from_millis(100))
                    .is_err(),
                "close は更新が終わるまでブロックされる"
            );
        });

        let document = state
            .update_document_content_atomic(&id, "edited".to_string())
            .expect("更新は成功する");
        assert_eq!(document.as_text(), Some("edited"));

        let close_result = result_rx
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("更新後に close が完了する");
        match close_result {
            Err(CoreError::Validation(message)) => assert_eq!(message, DOCUMENT_UNSAVED_MESSAGE),
            other => panic!("expected Validation, got {other:?}"),
        }
        closer.join().unwrap();

        let current = state.get_document(&id).expect("文書は残る");
        assert_eq!(current.as_text(), Some("edited"));
        assert!(state.is_unsaved(&id));
        assert_eq!(
            state.document_generation(&id),
            Some(generation_before + 1),
            "編集のみが世代を進め、拒否された close は進めない"
        );
    }
}
