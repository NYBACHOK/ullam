use std::{path::Path, process::Stdio};

use tokio::process::Child;

use crate::errors::BetterIoError;

#[inline]
#[must_use]
pub fn binary_location(dir: &Path) -> std::path::PathBuf {
    #[cfg(target_family = "unix")]
    {
        dir.join("llama-server")
    }
    #[cfg(target_family = "windows")]
    {
        dir.join("llama-server.exe")
    }
}

pub fn llama_serve(
    ollama_dir: impl AsRef<Path>,
    model_path: impl AsRef<Path>,
) -> Result<Child, BetterIoError> {
    let bin = binary_location(ollama_dir.as_ref());

    tracing::info!("starting llama binary in - {}", bin.display());

    tokio::process::Command::new(&bin)
        .kill_on_drop(true)
        .arg("-m")
        .arg(model_path.as_ref())
        .args(["--port", "9931"])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .map_err(|e| BetterIoError::new(bin, "start of llama binary", e))
}
