use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct Metrics {
    pub total_processed: u64,
    pub total_succeeded: u64,
    pub total_failed: u64,
}

pub struct MetricsCollector {
    metrics: Arc<Mutex<Metrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Metrics::default())),
        }
    }

    #[inline]
    pub fn record_success(&self) {
        let mut metrics = self.metrics.lock();
        metrics.total_processed += 1;
        metrics.total_succeeded += 1;
    }

    #[inline]
    pub fn record_failure(&self) {
        let mut metrics = self.metrics.lock();
        metrics.total_processed += 1;
        metrics.total_failed += 1;
    }

    pub fn get_snapshot(&self) -> Metrics {
        let metrics = self.metrics.lock();
        metrics.clone()
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
