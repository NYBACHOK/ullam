use std::sync::atomic::{AtomicU64, Ordering};

use hf_hub::progress::{DownloadEvent, ProgressEvent, ProgressHandler};

const TEN_MB: u64 = 10 * 1024 * 1024;

/// Progress logger that outputs a log message every 10 MB transferred.
#[derive(Default)]
pub struct ChunkProgressLogger {
    last_logged_bytes: AtomicU64,
}

impl ChunkProgressLogger {
    fn check_and_log(&self, current_bytes: u64, total_bytes: u64, label: &str) {
        let last = self.last_logged_bytes.load(Ordering::Relaxed);

        // Log if we've crossed a 10MB threshold or reached completion
        if current_bytes >= last + TEN_MB || (total_bytes > 0 && current_bytes >= total_bytes) {
            // Ensure thread-safe updating of the last logged threshold
            if self
                .last_logged_bytes
                .compare_exchange_weak(last, current_bytes, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
                let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

                if total_bytes > 0 {
                    let percent = (current_bytes as f64 / total_bytes as f64) * 100.0;
                    tracing::info!(
                        "[{label}] Progress: {current_mb:.2} MB / {total_mb:.2} MB ({percent:.1}%)"
                    );
                } else {
                    tracing::info!("[{}] Progress: {:.2} MB", label, current_mb);
                }
            }
        }
    }
}

impl ProgressHandler for ChunkProgressLogger {
    fn on_progress(&self, event: &ProgressEvent) {
        match event {
            ProgressEvent::Download(DownloadEvent::AggregateProgress {
                bytes_completed,
                total_bytes,
                ..
            }) => {
                self.check_and_log(*bytes_completed, *total_bytes, "Download");
            }
            _ => {}
        }
    }
}
