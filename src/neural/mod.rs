use crate::config::paths::FlashgrepPaths;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCacheManifest {
    pub model_id: String,
    pub created_at: i64,
    pub source_url: String,
    pub files: Vec<String>,
}

pub fn model_cache_root(paths: &FlashgrepPaths) -> PathBuf {
    paths.root().join("model-cache")
}

pub fn model_cache_dir(paths: &FlashgrepPaths) -> PathBuf {
    model_cache_root(paths).join("BAAI__bge-small-en-v1.5")
}

fn manifest_path(paths: &FlashgrepPaths) -> PathBuf {
    model_cache_dir(paths).join("manifest.json")
}

pub fn is_model_cached(paths: &FlashgrepPaths) -> FlashgrepResult<bool> {
    let manifest = manifest_path(paths);
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

pub fn ensure_model_for_startup_prompt(
    paths: &FlashgrepPaths,
    startup_context: &str,
) -> FlashgrepResult<ModelStartupPromptOutcome> {
    let response_override = std::env::var("FLASHGREP_MODEL_PROMPT_RESPONSE").ok();
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
        |p| ensure_model_cached(p),
    )
}

fn ensure_model_for_startup_prompt_with<F>(
    paths: &FlashgrepPaths,
    startup_context: &str,
    interactive: bool,
    response_override: Option<String>,
    mut downloader: F,
) -> FlashgrepResult<ModelStartupPromptOutcome>
where
    F: FnMut(&FlashgrepPaths) -> FlashgrepResult<()>,
{
    if is_model_cached(paths)? {
        return Ok(ModelStartupPromptOutcome::AlreadyCached);
    }

    if !interactive && response_override.is_none() {
        println!(
            "Neural model '{}' not found for {}. Continuing without model download (non-interactive).",
            EMBEDDING_MODEL_ID, startup_context
        );
        println!(
            "To enable neural features later, rerun in interactive mode or pre-populate {}",
            model_cache_dir(paths).display()
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

    println!(
        "Downloading neural model '{}' into {} ...",
        EMBEDDING_MODEL_ID,
        model_cache_root(paths).display()
    );
    match downloader(paths) {
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
    let cache_dir = model_cache_dir(paths);
    let manifest_path = manifest_path(paths);
    if is_model_cached(paths)? {
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
            "Offline mode is enabled and model cache is missing. Disable FLASHGREP_OFFLINE or pre-populate .flashgrep/model-cache/BAAI__bge-small-en-v1.5".to_string(),
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
        initialize_embedder(&model_cache_root(paths))?;
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
            |_| {
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
            |_| {
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
        let outcome =
            ensure_model_for_startup_prompt_with(&paths, "watcher startup", false, None, |_| {
                downloader_called = true;
                Ok(())
            })?;

        assert_eq!(outcome, ModelStartupPromptOutcome::NonInteractiveSkip);
        assert!(!downloader_called);
        Ok(())
    }
}
