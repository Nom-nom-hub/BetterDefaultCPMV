use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use indicatif::{ProgressBar, ProgressStyle};
use humansize::format_size;

/// Track progress of a transfer operation
#[derive(Clone)]
pub struct ProgressTracker {
    inner: Arc<Mutex<ProgressTrackerInner>>,
}

struct ProgressTrackerInner {
    total_bytes: u64,
    transferred_bytes: u64,
    start_time: Instant,
    progress_bar: Option<ProgressBar>,
}

impl ProgressTracker {
    pub fn new(total_bytes: u64, show_progress: bool) -> Self {
        let progress_bar = if show_progress {
            let pb = ProgressBar::new(total_bytes);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({percent}%) | {per_sec} | ⏱ {eta_precise}")
                    .expect("Template valid")
                    .progress_chars("████░░░░")
            );
            Some(pb)
        } else {
            None
        };

        Self {
            inner: Arc::new(Mutex::new(ProgressTrackerInner {
                total_bytes,
                transferred_bytes: 0,
                start_time: Instant::now(),
                progress_bar,
            })),
        }
    }

    pub fn add_bytes(&self, bytes: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.transferred_bytes += bytes;
        if let Some(ref pb) = inner.progress_bar {
            pb.set_position(inner.transferred_bytes);
        }
    }

    pub fn finish(&self) {
        let inner = self.inner.lock().unwrap();
        if let Some(ref pb) = inner.progress_bar {
            pb.finish_with_message("completed");
        }
    }

    pub fn get_stats(&self) -> TransferStats {
        let inner = self.inner.lock().unwrap();
        let elapsed = inner.start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            inner.transferred_bytes as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let remaining = inner.total_bytes.saturating_sub(inner.transferred_bytes);
        let eta = if speed > 0.0 {
            Duration::from_secs_f64(remaining as f64 / speed)
        } else {
            Duration::ZERO
        };

        TransferStats {
            transferred_bytes: inner.transferred_bytes,
            total_bytes: inner.total_bytes,
            elapsed,
            speed_bps: speed as u64,
            eta,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferStats {
    pub transferred_bytes: u64,
    pub total_bytes: u64,
    pub elapsed: Duration,
    pub speed_bps: u64,
    pub eta: Duration,
}

impl TransferStats {
    pub fn percent_complete(&self) -> u8 {
        if self.total_bytes == 0 {
            0
        } else {
            ((self.transferred_bytes as f64 / self.total_bytes as f64) * 100.0).min(100.0) as u8
        }
    }

    pub fn speed_human(&self) -> String {
        format_size(self.speed_bps, humansize::BINARY)
    }

    pub fn transferred_human(&self) -> String {
        format_size(self.transferred_bytes, humansize::BINARY)
    }

    pub fn total_human(&self) -> String {
        format_size(self.total_bytes, humansize::BINARY)
    }
}
