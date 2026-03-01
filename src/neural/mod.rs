use crate::config::paths::FlashgrepPaths;
use crate::config::{
    default_global_model_cache_path, Config, ModelCacheScope as ConfigModelCacheScope,
};
use crate::{FlashgrepError, FlashgrepResult};
#[cfg(feature = "neural")]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
#[cfg(feature = "neural")]
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
#[cfg(feature = "neural")]
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const EMBEDDING_MODEL_ID: &str = "BAAI/bge-small-en-v1.5";

const MODEL_SCOPE_OVERRIDE_ENV: &str = "FLASHGREP_MODEL_CACHE_SCOPE";
const MODEL_SCOPE_PROMPT_RESPONSE_ENV: &str = "FLASHGREP_MODEL_SCOPE_RESPONSE";

#[cfg(feature = "neural")]
static EMBEDDER: OnceCell<Mutex<TextEmbedding>> = OnceCell::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelStartupPromptOutcome {
    AlreadyCached,
    Downloaded,
    Declined,
    NonInteractiveSkip,
    DownloadFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelStorageScope {
    Local,
    Global,
}

impl ModelStorageScope {
    fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Global => "global",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCacheManifest {
    pub model_id: String,
    pub created_at: i64,
    pub source_url: String,
    pub files: Vec<String>,
}

pub fn model_cache_root(paths: &FlashgrepPaths) -> PathBuf {
    resolve_embedding_cache_root(paths).unwrap_or_else(|_| paths.root().join("model-cache"))
}

pub fn model_cache_dir(paths: &FlashgrepPaths) -> PathBuf {
    model_cache_root(paths).join("BAAI__bge-small-en-v1.5")
}

fn model_cache_dir_from_root(cache_root: &std::path::Path) -> PathBuf {
    cache_root.join("BAAI__bge-small-en-v1.5")
}

fn manifest_path_from_root(cache_root: &std::path::Path) -> PathBuf {
    model_cache_dir_from_root(cache_root).join("manifest.json")
}

pub fn is_model_cached(paths: &FlashgrepPaths) -> FlashgrepResult<bool> {
    let scope = resolve_default_scope(paths)?;
    Ok(find_cached_model_scope(paths, scope)?.is_some())
}

fn is_model_cached_in_root(cache_root: &std::path::Path) -> FlashgrepResult<bool> {
    let manifest = manifest_path_from_root(cache_root);
    if !manifest.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&manifest)?;
    let parsed: ModelCacheManifest = serde_json::from_str(&content).map_err(|e| {
        FlashgrepError::Config(format!(
            "Corrupted model cache manifest at {}: {}. Remove .flashgrep/model-cache and retry.",
            manifest.display(),
            e
        ))
    })?;
    Ok(parsed.model_id == EMBEDDING_MODEL_ID)
}

fn load_runtime_config(paths: &FlashgrepPaths) -> FlashgrepResult<Config> {
    if paths.config_file().exists() {
        Ok(Config::from_file(&paths.config_file())?)
    } else {
        Ok(Config::default())
    }
}

fn parse_scope(input: &str) -> Option<ModelStorageScope> {
    match input.trim().to_ascii_lowercase().as_str() {
        "g" | "global" => Some(ModelStorageScope::Global),
        "l" | "local" => Some(ModelStorageScope::Local),
        _ => None,
    }
}

fn resolve_default_scope(paths: &FlashgrepPaths) -> FlashgrepResult<ModelStorageScope> {
    if let Ok(override_value) = std::env::var(MODEL_SCOPE_OVERRIDE_ENV) {
        return parse_scope(&override_value).ok_or_else(|| {
            FlashgrepError::Config(format!(
                "Invalid {} value '{}'. Expected 'local' or 'global'.",
                MODEL_SCOPE_OVERRIDE_ENV, override_value
            ))
        });
    }

    let config = load_runtime_config(paths)?;
    Ok(match config.model_cache_scope {
        ConfigModelCacheScope::Local => ModelStorageScope::Local,
        ConfigModelCacheScope::Global => ModelStorageScope::Global,
    })
}

fn resolve_scope_cache_root(
    paths: &FlashgrepPaths,
    scope: Option<ModelStorageScope>,
) -> FlashgrepResult<PathBuf> {
    let resolved_scope = scope.unwrap_or(ModelStorageScope::Local);
    match resolved_scope {
        ModelStorageScope::Local => Ok(paths.root().join("model-cache")),
        ModelStorageScope::Global => {
            let config = load_runtime_config(paths)?;
            Ok(config
                .global_model_cache_path
                .unwrap_or_else(default_global_model_cache_path))
        }
    }
}

fn alternate_scope(scope: ModelStorageScope) -> ModelStorageScope {
    match scope {
        ModelStorageScope::Local => ModelStorageScope::Global,
        ModelStorageScope::Global => ModelStorageScope::Local,
    }
}

fn find_cached_model_scope(
    paths: &FlashgrepPaths,
    preferred_scope: ModelStorageScope,
) -> FlashgrepResult<Option<(ModelStorageScope, PathBuf)>> {
    let preferred_cache_root = resolve_scope_cache_root(paths, Some(preferred_scope))?;
    if is_model_cached_in_root(&preferred_cache_root)? {
        return Ok(Some((preferred_scope, preferred_cache_root)));
    }

    let fallback_scope = alternate_scope(preferred_scope);
    let fallback_cache_root = resolve_scope_cache_root(paths, Some(fallback_scope))?;
    if fallback_cache_root != preferred_cache_root && is_model_cached_in_root(&fallback_cache_root)?
    {
        return Ok(Some((fallback_scope, fallback_cache_root)));
    }

    Ok(None)
}

fn resolve_embedding_cache_root(paths: &FlashgrepPaths) -> FlashgrepResult<PathBuf> {
    let preferred_scope = resolve_default_scope(paths)?;
    if let Some((_, cache_root)) = find_cached_model_scope(paths, preferred_scope)? {
        return Ok(cache_root);
    }

    resolve_scope_cache_root(paths, Some(preferred_scope))
}

fn resolve_embedding_cache_dir(paths: &FlashgrepPaths) -> FlashgrepResult<PathBuf> {
    Ok(model_cache_dir_from_root(&resolve_embedding_cache_root(
        paths,
    )?))
}

fn choose_download_scope(
    _paths: &FlashgrepPaths,
    interactive: bool,
    prompt_override: Option<String>,
    default_scope: ModelStorageScope,
) -> FlashgrepResult<ModelStorageScope> {
    if let Some(value) = prompt_override {
        return parse_scope(&value).ok_or_else(|| {
            FlashgrepError::Config(format!(
                "Invalid {} value '{}'. Expected 'local' or 'global'.",
                MODEL_SCOPE_PROMPT_RESPONSE_ENV, value
            ))
        });
    }

    if !interactive {
        return Ok(default_scope);
    }

    let default_label = default_scope.label();
    print!(
        "Choose model download scope [global/local] (default: {}): ",
        default_label
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(default_scope);
    }

    parse_scope(trimmed).ok_or_else(|| {
        FlashgrepError::Config(format!(
            "Invalid scope '{}'. Expected 'global' or 'local'.",
            trimmed
        ))
    })
}

pub fn ensure_model_for_startup_prompt(
    paths: &FlashgrepPaths,
    startup_context: &str,
) -> FlashgrepResult<ModelStartupPromptOutcome> {
    let response_override = std::env::var("FLASHGREP_MODEL_PROMPT_RESPONSE").ok();
    let scope_prompt_override = std::env::var(MODEL_SCOPE_PROMPT_RESPONSE_ENV).ok();
    let force_non_interactive = std::env::var("FLASHGREP_NONINTERACTIVE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let interactive =
        !force_non_interactive && io::stdin().is_terminal() && io::stdout().is_terminal();

    ensure_model_for_startup_prompt_with(
        paths,
        startup_context,
        interactive,
        response_override,
        scope_prompt_override,
        |p, scope| ensure_model_cached_for_scope(p, scope),
    )
}

fn ensure_model_for_startup_prompt_with<F>(
    paths: &FlashgrepPaths,
    startup_context: &str,
    interactive: bool,
    response_override: Option<String>,
    scope_prompt_override: Option<String>,
    mut downloader: F,
) -> FlashgrepResult<ModelStartupPromptOutcome>
where
    F: FnMut(&FlashgrepPaths, ModelStorageScope) -> FlashgrepResult<()>,
{
    let default_scope = resolve_default_scope(paths)?;
    if let Some((cached_scope, cached_cache_root)) = find_cached_model_scope(paths, default_scope)?
    {
        if cached_scope != default_scope {
            println!(
                "Neural model '{}' is already cached in {} scope at {}.",
                EMBEDDING_MODEL_ID,
                cached_scope.label(),
                model_cache_dir_from_root(&cached_cache_root).display()
            );
        }
        return Ok(ModelStartupPromptOutcome::AlreadyCached);
    }

    let default_cache_root = resolve_scope_cache_root(paths, Some(default_scope))?;

    if !interactive && response_override.is_none() {
        println!(
            "Neural model '{}' not found for {}. Continuing without model download (non-interactive).",
            EMBEDDING_MODEL_ID, startup_context
        );
        println!(
            "To enable neural features later, rerun in interactive mode or pre-populate {}",
            model_cache_dir_from_root(&default_cache_root).display()
        );
        return Ok(ModelStartupPromptOutcome::NonInteractiveSkip);
    }

    let answer = if let Some(value) = response_override {
        value
    } else {
        print!(
            "Neural model '{}' is missing for {}. Download now? [y/N]: ",
            EMBEDDING_MODEL_ID, startup_context
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input
    };

    if !is_affirmative(&answer) {
        println!(
            "Skipping model download. Continuing {} with lexical indexing only.",
            startup_context
        );
        return Ok(ModelStartupPromptOutcome::Declined);
    }

    let selected_scope =
        choose_download_scope(paths, interactive, scope_prompt_override, default_scope)?;
    let selected_cache_root = resolve_scope_cache_root(paths, Some(selected_scope))?;

    if is_model_cached_in_root(&selected_cache_root)? {
        println!(
            "Neural model '{}' is already cached in {} scope at {}.",
            EMBEDDING_MODEL_ID,
            selected_scope.label(),
            model_cache_dir_from_root(&selected_cache_root).display()
        );
        return Ok(ModelStartupPromptOutcome::AlreadyCached);
    }

    println!(
        "Downloading neural model '{}' into {} scope at {} ...",
        EMBEDDING_MODEL_ID,
        selected_scope.label(),
        selected_cache_root.display()
    );
    match downloader(paths, selected_scope) {
        Ok(()) => {
            println!("Model download complete.");
            Ok(ModelStartupPromptOutcome::Downloaded)
        }
        Err(err) => {
            eprintln!(
                "Model download failed: {}. Continuing {} without neural model.",
                err, startup_context
            );
            Ok(ModelStartupPromptOutcome::DownloadFailed)
        }
    }
}

fn is_affirmative(input: &str) -> bool {
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

pub fn ensure_model_cached(paths: &FlashgrepPaths) -> FlashgrepResult<()> {
    let scope = resolve_default_scope(paths)?;
    if find_cached_model_scope(paths, scope)?.is_some() {
        return Ok(());
    }
    ensure_model_cached_for_scope(paths, scope)
}

fn ensure_model_cached_for_scope(
    paths: &FlashgrepPaths,
    scope: ModelStorageScope,
) -> FlashgrepResult<()> {
    let cache_root = resolve_scope_cache_root(paths, Some(scope))?;
    let cache_dir = model_cache_dir_from_root(&cache_root);
    let manifest_path = manifest_path_from_root(&cache_root);
    if is_model_cached_in_root(&cache_root)? {
        return Ok(());
    }

    std::fs::create_dir_all(&cache_dir)?;

    #[cfg(feature = "neural")]
    {
        // Initialize fastembed once to trigger model download into cache.
        let cache_dir_for_thread = cache_dir.clone();
        std::thread::spawn(move || initialize_embedder(&cache_dir_for_thread))
            .join()
            .map_err(|_| {
                FlashgrepError::Config("Model download worker panicked unexpectedly".to_string())
            })??;
    }

    #[cfg(not(feature = "neural"))]
    {
        return Err(FlashgrepError::Config(
            "Neural retrieval is disabled in this build. Rebuild with --features neural"
                .to_string(),
        ));
    }

    let manifest = ModelCacheManifest {
        model_id: EMBEDDING_MODEL_ID.to_string(),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        source_url: "https://huggingface.co/BAAI/bge-small-en-v1.5".to_string(),
        files: vec!["manifest.json".to_string(), "model.onnx".to_string()],
    };
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

#[cfg(feature = "neural")]
fn initialize_embedder(cache_dir: &PathBuf) -> FlashgrepResult<()> {
    if std::env::var("FLASHGREP_OFFLINE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Err(FlashgrepError::Config(
            format!(
                "Offline mode is enabled and model cache is missing. Disable FLASHGREP_OFFLINE or pre-populate {}",
                model_cache_dir_from_root(cache_dir).display()
            ),
        ));
    }

    if EMBEDDER.get().is_some() {
        return Ok(());
    }

    let options = InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_show_download_progress(true)
        .with_cache_dir(cache_dir.clone());

    let embedding = TextEmbedding::try_new(options).map_err(|e| {
        FlashgrepError::Config(format!("Failed to initialize embedding model: {e}"))
    })?;

    let _ = EMBEDDER.set(Mutex::new(embedding));
    Ok(())
}

#[cfg(feature = "neural")]
fn embedding_model(paths: &FlashgrepPaths) -> FlashgrepResult<&'static Mutex<TextEmbedding>> {
    if EMBEDDER.get().is_none() {
        let cache_dir = resolve_embedding_cache_dir(paths)?;
        initialize_embedder(&cache_dir)?;
    }
    EMBEDDER.get().ok_or_else(|| {
        FlashgrepError::Config("Embedding model was not initialized correctly".to_string())
    })
}

pub fn embed_text(paths: &FlashgrepPaths, text: &str) -> FlashgrepResult<Vec<f32>> {
    #[cfg(feature = "neural")]
    {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let model = embedding_model(paths)?;
        let guard = model
            .lock()
            .map_err(|_| FlashgrepError::Config("Embedding model lock poisoned".to_string()))?;
        let results = guard
            .embed(vec![text.to_string()], None)
            .map_err(|e| FlashgrepError::Search(format!("Embedding failed: {e}")))?;

        return Ok(results.into_iter().next().unwrap_or_default());
    }

    #[cfg(not(feature = "neural"))]
    {
        let _ = paths;
        let _ = text;
        Err(FlashgrepError::Config(
            "Neural retrieval is disabled in this build. Rebuild with --features neural"
                .to_string(),
        ))
    }
}

pub fn embed_texts(paths: &FlashgrepPaths, texts: &[String]) -> FlashgrepResult<Vec<Vec<f32>>> {
    #[cfg(feature = "neural")]
    {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let model = embedding_model(paths)?;
        let guard = model
            .lock()
            .map_err(|_| FlashgrepError::Config("Embedding model lock poisoned".to_string()))?;
        let results = guard
            .embed(texts.to_vec(), None)
            .map_err(|e| FlashgrepError::Search(format!("Embedding failed: {e}")))?;

        return Ok(results);
    }

    #[cfg(not(feature = "neural"))]
    {
        let _ = paths;
        let _ = texts;
        Err(FlashgrepError::Config(
            "Neural retrieval is disabled in this build. Rebuild with --features neural"
                .to_string(),
        ))
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_manifest(cache_root: &std::path::Path) -> FlashgrepResult<()> {
        let cache_dir = model_cache_dir_from_root(cache_root);
        std::fs::create_dir_all(&cache_dir)?;
        let manifest = ModelCacheManifest {
            model_id: EMBEDDING_MODEL_ID.to_string(),
            created_at: 0,
            source_url: "https://huggingface.co/BAAI/bge-small-en-v1.5".to_string(),
            files: vec!["manifest.json".to_string(), "model.onnx".to_string()],
        };
        std::fs::write(
            manifest_path_from_root(cache_root),
            serde_json::to_string_pretty(&manifest)?,
        )?;
        Ok(())
    }

    #[test]
    fn embeddings_are_deterministic() {
        assert!(is_affirmative("y"));
        assert!(is_affirmative("YES"));
        assert!(!is_affirmative("n"));
    }

    #[test]
    fn cosine_similarity_prefers_related_text() {
        let query = vec![1.0, 0.0, 0.0];
        let auth = vec![0.8, 0.1, 0.0];
        let unrelated = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&query, &auth) > cosine_similarity(&query, &unrelated));
    }

    #[test]
    fn startup_prompt_decline_continues() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let mut downloader_called = false;
        let outcome = ensure_model_for_startup_prompt_with(
            &paths,
            "index startup",
            true,
            Some("n".to_string()),
            None,
            |_, _| {
                downloader_called = true;
                Ok(())
            },
        )?;

        assert_eq!(outcome, ModelStartupPromptOutcome::Declined);
        assert!(!downloader_called);
        Ok(())
    }

    #[test]
    fn startup_prompt_accept_triggers_download() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let mut downloader_called = false;
        let outcome = ensure_model_for_startup_prompt_with(
            &paths,
            "index startup",
            true,
            Some("y".to_string()),
            Some("local".to_string()),
            |_, _| {
                downloader_called = true;
                Ok(())
            },
        )?;

        assert_eq!(outcome, ModelStartupPromptOutcome::Downloaded);
        assert!(downloader_called);
        Ok(())
    }

    #[test]
    fn startup_prompt_noninteractive_skips() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let mut downloader_called = false;
        let outcome = ensure_model_for_startup_prompt_with(
            &paths,
            "watcher startup",
            false,
            None,
            None,
            |_, _| {
                downloader_called = true;
                Ok(())
            },
        )?;

        assert_eq!(outcome, ModelStartupPromptOutcome::NonInteractiveSkip);
        assert!(!downloader_called);
        Ok(())
    }

    #[test]
    fn startup_prompt_accepts_scope_override() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let mut config = crate::config::Config::default();
        config.global_model_cache_path = Some(temp.path().join("shared-model-cache"));
        config.to_file(&paths.config_file())?;

        let mut selected_scope = None;
        let outcome = ensure_model_for_startup_prompt_with(
            &paths,
            "index startup",
            true,
            Some("y".to_string()),
            Some("global".to_string()),
            |_, scope| {
                selected_scope = Some(scope);
                Ok(())
            },
        )?;

        assert_eq!(outcome, ModelStartupPromptOutcome::Downloaded);
        assert_eq!(selected_scope, Some(ModelStorageScope::Global));
        Ok(())
    }

    #[test]
    fn global_scope_without_config_falls_back_to_default_path() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let mut selected_scope = None;
        let result = ensure_model_for_startup_prompt_with(
            &paths,
            "index startup",
            true,
            Some("y".to_string()),
            Some("global".to_string()),
            |_, scope| {
                selected_scope = Some(scope);
                Ok(())
            },
        );

        assert!(result.is_ok());
        assert_eq!(selected_scope, Some(ModelStorageScope::Global));
        Ok(())
    }

    #[test]
    fn startup_prompt_skips_when_model_is_cached_in_other_scope() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let global_cache_root = temp.path().join("shared-model-cache");
        write_manifest(&global_cache_root)?;

        let mut config = crate::config::Config::default();
        config.global_model_cache_path = Some(global_cache_root);
        config.to_file(&paths.config_file())?;

        let mut downloader_called = false;
        let outcome = ensure_model_for_startup_prompt_with(
            &paths,
            "index startup",
            true,
            None,
            None,
            |_, _| {
                downloader_called = true;
                Ok(())
            },
        )?;

        assert_eq!(outcome, ModelStartupPromptOutcome::AlreadyCached);
        assert!(!downloader_called);
        Ok(())
    }

    #[test]
    fn is_model_cached_checks_non_default_scope() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let global_cache_root = temp.path().join("shared-model-cache");
        write_manifest(&global_cache_root)?;

        let mut config = crate::config::Config::default();
        config.global_model_cache_path = Some(global_cache_root);
        config.to_file(&paths.config_file())?;

        assert!(is_model_cached(&paths)?);
        Ok(())
    }

    #[test]
    fn embedding_cache_root_prefers_cached_non_default_scope() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let global_cache_root = temp.path().join("shared-model-cache");
        write_manifest(&global_cache_root)?;

        let mut config = crate::config::Config::default();
        config.global_model_cache_path = Some(global_cache_root.clone());
        config.to_file(&paths.config_file())?;

        assert_eq!(resolve_embedding_cache_root(&paths)?, global_cache_root);
        Ok(())
    }

    #[test]
    fn embedding_cache_dir_uses_model_subdir_for_non_default_scope() -> FlashgrepResult<()> {
        let temp = TempDir::new()?;
        let repo_root = temp.path().to_path_buf();
        let paths = FlashgrepPaths::new(&repo_root);
        paths.create()?;

        let global_cache_root = temp.path().join("shared-model-cache");
        write_manifest(&global_cache_root)?;

        let mut config = crate::config::Config::default();
        config.global_model_cache_path = Some(global_cache_root.clone());
        config.to_file(&paths.config_file())?;

        assert_eq!(
            resolve_embedding_cache_dir(&paths)?,
            global_cache_root.join("BAAI__bge-small-en-v1.5")
        );
        Ok(())
    }
}
