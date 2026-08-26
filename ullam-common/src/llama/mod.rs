use std::{
    path::{Path, PathBuf},
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::Context;
use async_openai::{
    Client as OpenAIClient,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use tokio::process::Child;

use crate::{APP_DATA_DIR, hf_progress_logger::ChunkProgressLogger};

pub mod download;
pub mod serve;

pub const LLM_DATA_DIR: &str = "llm";
pub const LLM_MODELS_DIR: &str = "models";

static IS_LLM_ENGINE_LOADED: AtomicBool = AtomicBool::new(false);

static LLM_CHILD_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

// OpenAI-compatible HTTP client configured for local server (e.g. llama.cpp server / ollama)
static LLM_HTTP_CLIENT: LazyLock<OpenAIClient<OpenAIConfig>> = LazyLock::new(|| {
    let config = OpenAIConfig::new()
        .with_api_base("http://localhost:8080/v1")
        .with_api_key("not-needed-for-local");

    OpenAIClient::with_config(config)
});

#[derive(Debug, Clone)]
pub enum Model {
    Local(PathBuf),
    HuggingFace {
        model: String,
        owner: String,
        repo: String,
    },
}

pub async fn llm_download() -> anyhow::Result<()> {
    let cache_dir = dirs::config_local_dir().unwrap_or_else(|| std::env::temp_dir());
    let target_dir = APP_DATA_DIR.join(LLM_DATA_DIR);

    create_dir_if_not_exists(&cache_dir)
        .await
        .with_context(|| format!("failed to create cache dir: {:?}", cache_dir))?;

    create_dir_if_not_exists(&target_dir)
        .await
        .with_context(|| format!("failed to create target dir: {:?}", target_dir))?;

    download::download(cache_dir, target_dir).await?;

    Ok(())
}

pub async fn llm_load(model: Model) -> anyhow::Result<()> {
    let model_path = get_or_load(model, APP_DATA_DIR.join(LLM_DATA_DIR)).await?;

    let mut llm_backend_lock = LLM_CHILD_PROCESS.lock().map_err(|_| {
        anyhow::anyhow!("POISONED LOCK: Failed to acquire LLM_CHILD_PROCESS mutex lock")
    })?;

    if llm_backend_lock.is_none() {
        let child = serve::ollama_serve(APP_DATA_DIR.join(LLM_DATA_DIR), model_path)?;
        *llm_backend_lock = Some(child);
    } else {
        tracing::warn!("Tried to init new server instance while old is still running");
    }

    IS_LLM_ENGINE_LOADED.store(true, Ordering::SeqCst);

    Ok(())
}

pub async fn llm_unload() -> anyhow::Result<()> {
    let mut llm_backend_lock = LLM_CHILD_PROCESS.lock().map_err(|_| {
        anyhow::anyhow!("POISONED LOCK: Failed to acquire LLM_CHILD_PROCESS mutex lock")
    })?;

    if let Some(mut child) = llm_backend_lock.take() {
        let _ = child.kill().await;
    }

    IS_LLM_ENGINE_LOADED.store(false, Ordering::SeqCst);

    Ok(())
}

pub async fn llm_generate(prompt: &str, max_tokens: u32) -> anyhow::Result<String> {
    if !IS_LLM_ENGINE_LOADED.load(Ordering::SeqCst) {
        anyhow::bail!("You need to start llm engine first");
    }

    let request = CreateChatCompletionRequestArgs::default()
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ])
        .max_tokens(max_tokens)
        .build()?;

    let mut response = LLM_HTTP_CLIENT.chat().create(request).await?;

    let msg = response
        .choices
        .pop()
        .map(|c| c.message.content.clone())
        .flatten()
        .ok_or_else(|| {
            anyhow::anyhow!("failed to retrieve any choice content from model response")
        })?;

    Ok(msg)
}

async fn create_dir_if_not_exists(path: &Path) -> std::io::Result<()> {
    if !tokio::fs::try_exists(path).await? {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

async fn get_or_load(model: Model, dir: PathBuf) -> anyhow::Result<PathBuf> {
    fn check_is_gguf(name: impl AsRef<str>) -> anyhow::Result<()> {
        if !name.as_ref().ends_with(".gguf") {
            return Err(anyhow::anyhow!("Not a GGUF model"));
        }

        Ok(())
    }

    match model {
        Model::Local(path) => {
            check_is_gguf(path.to_string_lossy())?;

            Ok(path)
        }
        Model::HuggingFace { model, owner, repo } => {
            check_is_gguf(&model)?;

            hf_hub::HFClientBuilder::new()
                .build()
                .with_context(|| "unable to create huggingface api")?
                .model(owner, repo)
                .download_file()
                .local_dir(dir)
                .filename(model)
                .progress(ChunkProgressLogger::default())
                .send()
                .await
                .with_context(|| "unable to download model")
        }
    }
}
