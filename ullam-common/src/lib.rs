use std::{path::PathBuf, sync::LazyLock};

pub mod errors;
mod gh_release_reader;
mod hf_progress_logger;
pub mod llama;

pub static APP_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    const BUNDLE_ID: &str = "ullam";

    #[inline]
    fn data_dir() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            PathBuf::from("/data/data")
        }
        #[cfg(not(target_os = "android"))]
        {
            dirs::data_dir().unwrap_or_else(|| {
                let dir = std::env::current_dir().unwrap_or_default();

                tracing::error!(data_dir = %dir.display(), "failed to get data OS dir will use");

                dir
            })
        }
    }

    data_dir().join(if cfg!(debug_assertions) {
        "ullam/debug"
    } else {
        BUNDLE_ID
    })
});

pub fn get_pre_processing_config() -> anyhow::Result<ullam_preprocessor::PreprocessorConfig> {
    let config_path = APP_DATA_DIR.join("config.json");
    if config_path.exists() {
        let config = serde_json::from_slice(&std::fs::read(config_path)?)?;
        return Ok(config);
    }

    let config = ullam_preprocessor::PreprocessorConfig::default();

    std::fs::write(
        config_path,
        serde_json::to_string_pretty(&config).expect("never fails"),
    )?;

    Ok(config)
}
