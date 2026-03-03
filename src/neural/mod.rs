use crate::config::paths::FlashgrepPaths;
use crate::config::{Config, NeuralProviderConfig};
use crate::db::models::{Chunk, SearchResult, Symbol};
use crate::{FlashgrepError, FlashgrepResult};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use async_openai::Client;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::IsTerminal;
use std::time::Duration;

pub fn ensure_neural_config_prompt(paths: &FlashgrepPaths) -> FlashgrepResult<()> {
    let config_path = paths.config_file();
    if !config_path.exists() {
        return Ok(());
    }

    let mut config = Config::from_file(&config_path)?;
    if config.neural.initialized
        || !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
    {
        return Ok(());
    }

    print!("Enable neural navigation (index-first, optional provider calls)? [y/N]: ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let enable = matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes");

    config.neural.enabled = enable;
    if enable {
        let current_provider = config.neural.provider.provider.clone();
        print!(
            "Neural provider [openrouter/openai/custom] (default: {}): ",
            current_provider
        );
        std::io::stdout().flush()?;
        let mut provider_input = String::new();
        std::io::stdin().read_line(&mut provider_input)?;
        let provider_choice = provider_input.trim().to_ascii_lowercase();
        let provider = if provider_choice.is_empty() {
            current_provider.to_ascii_lowercase()
        } else {
            provider_choice
        };

        if provider == "openrouter" {
            config.neural.provider.provider = "openrouter".to_string();
            config.neural.provider.base_url = "https://openrouter.ai/api/v1".to_string();
            config.neural.provider.model = "arcee-ai/trinity-large-preview:free".to_string();
            config.neural.provider.api_key_env = "OPENROUTER_API_KEY".to_string();
        } else if provider == "openai" {
            config.neural.provider.provider = "openai".to_string();
            config.neural.provider.base_url = "https://api.openai.com/v1".to_string();
            config.neural.provider.model = "gpt-4o-mini".to_string();
            config.neural.provider.api_key_env = "OPENAI_API_KEY".to_string();
        } else {
            config.neural.provider.provider = provider;
        }

        print!("Model (default: {}): ", config.neural.provider.model);
        std::io::stdout().flush()?;
        let mut model_input = String::new();
        std::io::stdin().read_line(&mut model_input)?;
        let model = model_input.trim();
        if !model.is_empty() {
            config.neural.provider.model = model.to_string();
        }

        print!("Base URL (default: {}): ", config.neural.provider.base_url);
        std::io::stdout().flush()?;
        let mut base_input = String::new();
        std::io::stdin().read_line(&mut base_input)?;
        let base = base_input.trim();
        if !base.is_empty() {
            config.neural.provider.base_url = base.to_string();
        }

        print!(
            "API key env var name (default: {}): ",
            config.neural.provider.api_key_env
        );
        std::io::stdout().flush()?;
        let mut env_input = String::new();
        std::io::stdin().read_line(&mut env_input)?;
        let env_name = env_input.trim();
        if !env_name.is_empty() {
            config.neural.provider.api_key_env = env_name.to_string();
        }

        print!(
            "API key (leave blank to use env var {}): ",
            config.neural.provider.api_key_env
        );
        std::io::stdout().flush()?;
        let mut key_input = String::new();
        std::io::stdin().read_line(&mut key_input)?;
        let key = key_input.trim();
        config.neural.provider.api_key = if key.is_empty() {
            None
        } else {
            Some(key.to_string())
        };
    }
    config.neural.initialized = true;
    config.to_file(&config_path)?;
    Ok(())
}

pub fn pseudo_embedding(text: &str, dim: usize) -> Vec<f32> {
    if dim == 0 {
        return Vec::new();
    }
    let mut v = vec![0.0f32; dim];
    for token in text
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
    {
        let mut h = DefaultHasher::new();
        token.to_ascii_lowercase().hash(&mut h);
        let idx = (h.finish() as usize) % dim;
        v[idx] += 1.0;
    }
    normalize(&mut v);
    v
}

fn normalize(v: &mut [f32]) {
    let sum_sq: f32 = v.iter().map(|x| x * x).sum();
    if sum_sq <= f32::EPSILON {
        return;
    }
    let norm = sum_sq.sqrt();
    for x in v.iter_mut() {
        *x /= norm;
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let mut dot = 0.0f32;
    for i in 0..len {
        dot += a[i] * b[i];
    }
    dot
}

pub fn build_knowledge_graph_edges(
    file_path: &std::path::Path,
    chunks: &[Chunk],
    symbols: &[Symbol],
) -> Vec<(String, String, String)> {
    let file_node = format!("file:{}", file_path.to_string_lossy());
    let mut edges = Vec::new();
    for chunk in chunks {
        let chunk_node = format!(
            "chunk:{}:{}-{}",
            chunk.file_path.to_string_lossy(),
            chunk.start_line,
            chunk.end_line
        );
        edges.push((
            file_node.clone(),
            chunk_node.clone(),
            "contains".to_string(),
        ));
    }
    for symbol in symbols {
        let symbol_node = format!(
            "symbol:{}:{}:{}",
            symbol.file_path.to_string_lossy(),
            symbol.line_number,
            symbol.symbol_name
        );
        edges.push((file_node.clone(), symbol_node, "defines".to_string()));
    }
    edges
}

pub fn provider_assist_rerank(
    provider: &NeuralProviderConfig,
    api_key: &str,
    query: &str,
    candidates: &[SearchResult],
) -> FlashgrepResult<Vec<usize>> {
    let mut compact = Vec::new();
    for (idx, c) in candidates
        .iter()
        .enumerate()
        .take(provider.max_candidates.max(1))
    {
        compact.push(format!(
            "#{idx} {}:{}-{} {}",
            c.file_path.display(),
            c.start_line,
            c.end_line,
            c.preview.replace('\n', " ")
        ));
    }

    let prompt = format!(
        "You are ranking repository code matches. Query: {query}. Return ONLY a JSON array of candidate IDs that are truly relevant, ordered best-first. Return [] if none are relevant. Candidates:\n{}",
        compact.join("\n")
    );

    let timeout_ms = provider.timeout_ms.max(500);
    let base_url = normalize_openai_api_base(&provider.base_url);
    let model = provider.model.clone();
    let key = api_key.to_string();

    let text = std::thread::spawn(move || -> FlashgrepResult<String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| FlashgrepError::Search(format!("provider runtime init error: {e}")))?;

        rt.block_on(async move {
            let cfg = OpenAIConfig::new()
                .with_api_key(key)
                .with_api_base(base_url);
            let client = Client::with_config(cfg);

            let user_msg = ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .map_err(|e| FlashgrepError::Search(format!("provider request build error: {e}")))?;

            let request = CreateChatCompletionRequestArgs::default()
                .model(model)
                .messages([user_msg.into()])
                .build()
                .map_err(|e| FlashgrepError::Search(format!("provider request build error: {e}")))?;

            let call = tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                client.chat().create(request),
            )
            .await
            .map_err(|_| FlashgrepError::Search("provider request timed out".to_string()))?;

            let response = call
                .map_err(|e| {
                    let msg = e.to_string();
                    if msg.contains("failed to deserialize api response")
                        || msg.contains("Not Found")
                        || msg.contains("404")
                    {
                        FlashgrepError::Search(format!(
                            "provider request failed: {msg}. Check neural.provider.base_url; for OpenRouter use https://openrouter.ai/api/v1"
                        ))
                    } else {
                        FlashgrepError::Search(format!("provider request failed: {msg}"))
                    }
                })?;

            serde_json::to_string(&response)
                .map_err(|e| FlashgrepError::Search(format!("provider response serialization error: {e}")))
        })
    })
    .join()
    .map_err(|_| FlashgrepError::Search("provider worker thread panicked".to_string()))??;

    parse_rerank_ids(&text)
}

fn normalize_openai_api_base(base_url: &str) -> String {
    let mut base = base_url.trim().trim_end_matches('/').to_string();
    for suffix in ["/chat/completions", "/v1/chat/completions"] {
        if base.ends_with(suffix) {
            base.truncate(base.len() - suffix.len());
            break;
        }
    }
    base
}

fn parse_rerank_ids(raw: &str) -> FlashgrepResult<Vec<usize>> {
    let root: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
        FlashgrepError::Search(format!("provider returned invalid JSON payload: {e}"))
    })?;

    let content = root
        .pointer("/choices/0/message/content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(raw);

    if let Ok(ids) = serde_json::from_str::<Vec<usize>>(content.trim()) {
        return Ok(ids);
    }
    if let Ok(ids) = serde_json::from_str::<Vec<String>>(content.trim()) {
        return ids
            .into_iter()
            .map(|s| {
                s.trim().parse::<usize>().map_err(|e| {
                    FlashgrepError::Search(format!("failed to parse provider ID '{}': {e}", s))
                })
            })
            .collect();
    }

    let json_array = extract_first_json_array(content).ok_or_else(|| {
        FlashgrepError::Search("provider response did not include a JSON ID array".to_string())
    })?;
    if let Ok(ids) = serde_json::from_str::<Vec<usize>>(&json_array) {
        return Ok(ids);
    }
    if let Ok(ids) = serde_json::from_str::<Vec<String>>(&json_array) {
        return ids
            .into_iter()
            .map(|s| {
                s.trim().parse::<usize>().map_err(|e| {
                    FlashgrepError::Search(format!("failed to parse provider ID '{}': {e}", s))
                })
            })
            .collect();
    }
    Err(FlashgrepError::Search(
        "failed to parse provider ID array".to_string(),
    ))
}

fn extract_first_json_array(input: &str) -> Option<String> {
    let start = input.find('[')?;
    let mut depth = 0i32;
    for (i, ch) in input[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + i + 1;
                    return Some(input[start..end].to_string());
                }
            }
            _ => {}
        }
    }
    None
}
