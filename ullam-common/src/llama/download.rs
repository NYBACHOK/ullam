use std::path::Path;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::errors::BetterIoError;
use crate::gh_release_reader::DownloadError;

const GH_RELEASE_LINK: &str = "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest";
const DOWNLOAD_FILENAME: &str = "llama.tar.gz";

#[derive(Debug, thiserror::Error)]
pub enum ArchiveDownloadError {
    #[error(transparent)]
    Download(#[from] DownloadError),
    #[error(transparent)]
    Io(#[from] BetterIoError),
    #[error("failed to find release")]
    ReleaseNotFound,
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Zip(#[from] ::zip::result::ZipError),
}

pub async fn latest_release_url(
    client: &reqwest::Client,
) -> Result<url::Url, ArchiveDownloadError> {
    let is_nvidia = is_nvidia().await;

    // detect gpu vendor
    let gpu_flag = match is_nvidia {
        true if cfg!(target_os = "windows") => "cuda-13.1",
        false if cfg!(target_os = "windows") => "hip",
        _ if cfg!(not(target_arch = "aarch64")) => "vulkan",
        _ => "",
    };

    let arch_flag = {
        #[cfg(target_arch = "x86_64")]
        {
            "x64"
        }
        #[cfg(target_arch = "aarch64")]
        {
            "arm64"
        }
    };

    let os_flag = {
        #[cfg(target_os = "macos")]
        {
            "macos"
        }
        #[cfg(target_os = "linux")]
        {
            "ubuntu"
        }
        #[cfg(target_os = "windows")]
        {
            "win"
        }
    };

    let release_assets = crate::gh_release_reader::release(client, GH_RELEASE_LINK, true)
        .await
        .map_err(|e| ArchiveDownloadError::Download(DownloadError::Other(e)))?
        .assets;

    let release = release_assets
        .into_iter()
        .find(|this| {
            this.name.contains(gpu_flag)
                && this.name.contains(arch_flag)
                && this.name.contains(os_flag)
        })
        .ok_or(ArchiveDownloadError::ReleaseNotFound)?
        .browser_download_url;

    Ok(release)
}

pub async fn download(
    cache_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
) -> Result<(), ArchiveDownloadError> {
    let client = reqwest::Client::new();

    let url = latest_release_url(&client).await?;

    let archive_location = cache_dir.as_ref().join(DOWNLOAD_FILENAME);

    crate::gh_release_reader::download_file(&client, url, &archive_location).await?;

    unpack_archive(&archive_location, target_dir.as_ref())?;

    let dirs = std::fs::read_dir(&target_dir)
        .map_err(|e| BetterIoError::new(target_dir.as_ref(), "open list in result dir", e))?;

    for dir in dirs {
        let dir =
            dir.map_err(|e| BetterIoError::new(target_dir.as_ref(), "open resulting dir", e))?;

        copy_dir_all(dir.path(), target_dir.as_ref()).map_err(|e| {
            BetterIoError::new(
                target_dir.as_ref(),
                "copying content of unpacked archive to localdir",
                e,
            )
        })?;

        break; // there should be single dir
    }

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unpack_archive(tar_location: &Path, target_dir: &Path) -> Result<(), BetterIoError> {
    let file = std::fs::File::open(tar_location).map_err(|error| BetterIoError {
        location: tar_location.to_path_buf(),
        context: "opening archive descriptor",
        error,
    })?;

    tracing::info!("Starting unpacking of archive - {}", tar_location.display());

    let decoder = flate2::read::MultiGzDecoder::new(file);

    let mut archive = tar::Archive::new(decoder);

    archive.unpack(target_dir).map_err(|error| BetterIoError {
        location: target_dir.to_path_buf(),
        context: "unpacking of archive",
        error,
    })?;

    tracing::info!("Finished unpacking of archive - {}", tar_location.display());

    Ok(())
}

#[cfg(target_os = "windows")]
fn unpack_archive(tar_location: &Path, target_dir: &Path) -> Result<(), ArchiveDownloadError> {
    let file = std::fs::File::open(&tar_location).map_err(|error| BetterIoError {
        location: tar_location.to_path_buf(),
        context: "opening archive descriptor",
        error,
    })?;

    tracing::info!("Starting unpacking of archive - {}", tar_location.display());

    let mut archive = zip::ZipArchive::new(file)?;

    archive.extract(target_dir)?;

    tracing::info!("Finished unpacking of archive - {}", tar_location.display());

    Ok(())
}

/// Simple method which check is nvidia drivers present in system. This ignores intel gpu and igpu, but for now should be sufficient
pub async fn is_nvidia() -> bool {
    match tokio::process::Command::new("nvidia-smi").spawn() {
        Ok(child) => child,
        Err(e) => {
            tracing::warn!("Failed to find nvidia drivers. Reason: {e}");

            return false;
        }
    }
    .wait()
    .await
    .is_ok_and(|this| this.success())
}
