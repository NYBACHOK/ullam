use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Failed IO for: {location} during {context}. Reason: {error}")]
    Io {
        location: PathBuf,
        context: &'static str,
        error: std::io::Error,
    },
    #[error("Network error. Reason: {0}")]
    Network(#[from] reqwest::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("Network error. Failed to request file")]
    FailedRequest,
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Zip(#[from] ::zip::result::ZipError),
}

impl DownloadError {
    #[inline]
    pub fn new_io(
        location: impl Into<PathBuf>,
        context: &'static str,
        error: std::io::Error,
    ) -> Self {
        Self::Io {
            location: location.into(),
            context,
            error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub url: Url,
    pub html_url: Url,
    pub assets_url: Url,
    pub upload_url: Url,
    pub tarball_url: Option<Url>,
    pub zipball_url: Option<Url>,
    pub id: i64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub author: SimpleUser,
    pub assets: Vec<ReleaseAsset>,
    pub reactions: Option<Reactions>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub url: Url,
    pub browser_download_url: Url,
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub state: String,
    pub content_type: String,
    pub size: i64,
    pub download_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub uploader: Option<SimpleUser>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleUser {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: Url,
    pub html_url: Url,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reactions {
    pub url: Url,
    pub total_count: i32,
    #[serde(rename = "+1")]
    pub plus_one: i32,
    #[serde(rename = "-1")]
    pub minus_one: i32,
    pub laugh: i32,
    pub heart: i32,
    pub rocket: i32,
}

pub async fn release(
    client: &reqwest::Client,
    base_repo_url: impl IntoUrl,
    prerelease: bool,
) -> Result<Release, anyhow::Error> {
    let mut url = base_repo_url.into_url()?;

    let mut segments: Vec<String> = url
        .path_segments()
        .map(|s| s.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    while let Some(last) = segments.last() {
        if last == "latest" || last == "releases" || last.is_empty() {
            segments.pop();
        } else {
            break;
        }
    }

    if prerelease {
        segments.push("releases".into());
    } else {
        segments.push("releases".into());
        segments.push("latest".into());
    }

    url.set_path(&segments.join("/"));

    if prerelease {
        url.set_query(Some("per_page=1"));
    } else {
        url.set_query(None);
    }

    let req = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:147.0) Gecko/20100101 Firefox/147.0",
        );

    if prerelease {
        let releases: Vec<Release> = req.send().await?.error_for_status()?.json().await?;

        releases
            .into_iter()
            .find(|r| r.prerelease)
            .ok_or_else(|| anyhow::anyhow!("no pre-release found for repository"))
    } else {
        let rel = req
            .send()
            .await?
            .error_for_status()?
            .json::<Release>()
            .await?;

        Ok(rel)
    }
}

pub async fn download_file(
    client: &reqwest::Client,
    url: impl IntoUrl,
    location: impl AsRef<Path>,
) -> Result<(), DownloadError> {
    let url = url.into_url()?;

    // Send HEAD request to get file size
    let head_response = client.head(url.clone()).send().await?;

    let total_size = head_response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or(DownloadError::FailedRequest)?;

    let is_exists = tokio::fs::try_exists(&location).await.map_err(|error| {
        DownloadError::new_io(
            location.as_ref(),
            "checking is file exists before download starts",
            error,
        )
    })?;

    if is_exists {
        match tokio::fs::metadata(&location).await {
            Ok(meta) => {
                const LOG_MESSAGE: &str =
                    "Existing download with matching size found. Ignoring download";

                #[cfg(target_family = "unix")]
                {
                    use std::os::unix::fs::MetadataExt;

                    if meta.size() == total_size {
                        tracing::warn!("{LOG_MESSAGE}");
                    }
                }

                #[cfg(target_family = "windows")]
                if meta.len() == total_size {
                    tracing::warn!("{LOG_MESSAGE}");
                }

                return Ok(());
            }
            _ => (),
        }
    }

    // Start the actual download
    let response = client.get(url).send().await?.error_for_status()?;

    let mut file = tokio::fs::File::create(&location).await.map_err(|error| {
        DownloadError::new_io(location.as_ref(), "creation of file descriptor", error)
    })?;

    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;
    let start_time = tokio::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await.map_err(|error| {
            DownloadError::new_io(location.as_ref(), "saving of downloaded chunk", error)
        })?;

        downloaded += chunk.len() as u64;

        // Print progress every 1MB
        if downloaded % (1024 * 1024) == 0 || downloaded == total_size {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = downloaded as f64 / elapsed / 1024.0 / 1024.0; // MB/s
            let progress = if total_size > 0 {
                (downloaded as f64 / total_size as f64 * 100.0) as u32
            } else {
                0
            };

            tracing::debug!(
                "Downloaded: {:.1} MB | Progress: {}% | Speed: {:.1} MB/s",
                downloaded as f64 / 1024.0 / 1024.0,
                progress,
                speed
            );
        }
    }

    file.flush().await.map_err(|error| {
        DownloadError::new_io(location.as_ref(), "flushing file descriptor", error)
    })?;

    tracing::info!(
        "Download completed in {:.2} seconds",
        start_time.elapsed().as_secs_f64()
    );

    Ok(())
}
