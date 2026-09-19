use std::{
    path::{Path, PathBuf},
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::Context;
use async_openai::{
    Client as OpenAIClient,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ReasoningEffort, ResponseFormat, ResponseFormatJsonSchema,
    },
};
use tokio::process::Child;

use crate::{APP_DATA_DIR, hf_progress_logger::ChunkProgressLogger};

pub mod download;
pub mod serve;

pub const LLM_DATA_DIR: &str = "llm";
pub const LLM_MODELS_DIR: &str = "models";

const TIMEOUT: Duration = Duration::from_secs(60);

static IS_LLM_ENGINE_LOADED: AtomicBool = AtomicBool::new(false);

static LLM_CHILD_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

// OpenAI-compatible HTTP client configured for local server (e.g. llama.cpp server / ollama)
static LLM_HTTP_CLIENT: LazyLock<OpenAIClient<OpenAIConfig>> = LazyLock::new(|| {
    let config = OpenAIConfig::new()
        .with_api_base("http://localhost:9931/v1")
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
    let cache_dir = dirs::config_local_dir().unwrap_or_else(std::env::temp_dir);
    let target_dir = APP_DATA_DIR.join(LLM_DATA_DIR);

    create_dir_if_not_exists(&cache_dir)
        .await
        .with_context(|| format!("failed to create cache dir: {cache_dir:?}"))?;

    create_dir_if_not_exists(&target_dir)
        .await
        .with_context(|| format!("failed to create target dir: {target_dir:?}"))?;

    download::download(cache_dir, target_dir).await?;

    Ok(())
}

pub async fn llm_load(model: Model) -> anyhow::Result<()> {
    let model_path = get_or_load(model).await?;

    let mut llm_backend_lock = LLM_CHILD_PROCESS.lock().map_err(|_| {
        anyhow::anyhow!("POISONED LOCK: Failed to acquire LLM_CHILD_PROCESS mutex lock")
    })?;

    if llm_backend_lock.is_none() {
        let child = serve::llama_serve(APP_DATA_DIR.join(LLM_DATA_DIR), model_path)?;
        *llm_backend_lock = Some(child);
    } else {
        tracing::warn!("Tried to init new server instance while old is still running");
    }

    std::mem::drop(llm_backend_lock);

    IS_LLM_ENGINE_LOADED.store(true, Ordering::SeqCst);

    wait_until_ready(TIMEOUT).await?;

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

pub async fn llm_generate<T: schemars::JsonSchema + serde::de::DeserializeOwned>(
    system_prompt: &str,
    user_prompt: &str,
) -> anyhow::Result<T> {
    if !IS_LLM_ENGINE_LOADED.load(Ordering::SeqCst) {
        anyhow::bail!("You need to start llm engine first");
    }

    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            name: "flight_analysis".to_string(),
            description: Some("Evidence-based analysis of an ArduPilot flight log".to_string()),
            schema: serde_json::to_value(schemars::schema_for!(T)).expect("never fails"),
            strict: Some(true),
        },
    };

    let request = CreateChatCompletionRequestArgs::default()
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_prompt)
                .build()?
                .into(),
        ])
        .response_format(response_format)
        .reasoning_effort(ReasoningEffort::Medium)
        .build()?;

    let mut response = LLM_HTTP_CLIENT.chat().create(request).await?;

    let msg = response
        .choices
        .pop()
        .and_then(|c| c.message.content.clone())
        .ok_or_else(|| {
            anyhow::anyhow!("failed to retrieve any choice content from model response")
        })?;

    serde_json::from_str(&msg).context("deserialization of response")
}

async fn create_dir_if_not_exists(path: &Path) -> std::io::Result<()> {
    if !tokio::fs::try_exists(path).await? {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

async fn get_or_load(model: Model) -> anyhow::Result<PathBuf> {
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
                .filename(model)
                .progress(ChunkProgressLogger::default())
                .send()
                .await
                .with_context(|| "unable to download model")
        }
    }
}

async fn wait_until_ready(timeout: Duration) -> anyhow::Result<()> {
    let start = Instant::now();

    tracing::info!("Waiting for LLM server to start");

    while start.elapsed() < timeout {
        if LLM_HTTP_CLIENT.models().list().await.is_ok() {
            tokio::time::sleep(Duration::from_secs(6)).await;

            tracing::info!("LLM engine is fully loaded and accepting requests");
            return Ok(());
        }
    }

    anyhow::bail!("Timed out waiting for LLM server readiness")
}
