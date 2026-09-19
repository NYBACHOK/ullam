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
