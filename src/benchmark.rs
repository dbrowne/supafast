use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_duration: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub avg_latency: Duration,
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub throughput_rps: f64,
}

pub struct BenchmarkCollector {
    start_time: Instant,
    latencies: Arc<Mutex<Vec<Duration>>>,
    total_requests: Arc<Mutex<u64>>,
    successful_requests: Arc<Mutex<u64>>,
    failed_requests: Arc<Mutex<u64>>,
}

impl BenchmarkCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            latencies: Arc::new(Mutex::new(Vec::with_capacity(10000))),
            total_requests: Arc::new(Mutex::new(0)),
            successful_requests: Arc::new(Mutex::new(0)),
            failed_requests: Arc::new(Mutex::new(0)),
        }
    }

    pub fn record_request(&self, latency: Duration, success: bool) {
        let mut latencies = self.latencies.lock();
        latencies.push(latency);

        *self.total_requests.lock() += 1;
        if success {
            *self.successful_requests.lock() += 1;
        } else {
            *self.failed_requests.lock() += 1;
        }
    }

    pub fn get_stats(&self) -> BenchmarkStats {
        let mut latencies = self.latencies.lock().clone();
        let total_duration = self.start_time.elapsed();
        let total_requests = *self.total_requests.lock();
        let successful_requests = *self.successful_requests.lock();
        let failed_requests = *self.failed_requests.lock();

        if latencies.is_empty() {
            return BenchmarkStats {
                total_requests,
                successful_requests,
                failed_requests,
                total_duration,
                min_latency: Duration::ZERO,
                max_latency: Duration::ZERO,
                avg_latency: Duration::ZERO,
                p50_latency: Duration::ZERO,
                p95_latency: Duration::ZERO,
                p99_latency: Duration::ZERO,
                throughput_rps: 0.0,
            };
        }

        latencies.sort();

        let min_latency = *latencies.first().unwrap();
        let max_latency = *latencies.last().unwrap();
        let avg_latency = Duration::from_nanos(
            latencies.iter().map(|d| d.as_nanos() as u64).sum::<u64>() / latencies.len() as u64,
        );

        let p50_idx = (latencies.len() as f64 * 0.50) as usize;
        let p95_idx = (latencies.len() as f64 * 0.95) as usize;
        let p99_idx = (latencies.len() as f64 * 0.99) as usize;

        let p50_latency = latencies[p50_idx.min(latencies.len() - 1)];
        let p95_latency = latencies[p95_idx.min(latencies.len() - 1)];
        let p99_latency = latencies[p99_idx.min(latencies.len() - 1)];

        let throughput_rps = if total_duration.as_secs_f64() > 0.0 {
            total_requests as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        BenchmarkStats {
            total_requests,
            successful_requests,
            failed_requests,
            total_duration,
            min_latency,
            max_latency,
            avg_latency,
            p50_latency,
            p95_latency,
            p99_latency,
            throughput_rps,
        }
    }

    pub fn reset(&self) {
        self.latencies.lock().clear();
        *self.total_requests.lock() = 0;
        *self.successful_requests.lock() = 0;
        *self.failed_requests.lock() = 0;
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            start_time: self.start_time,
            latencies: Arc::clone(&self.latencies),
            total_requests: Arc::clone(&self.total_requests),
            successful_requests: Arc::clone(&self.successful_requests),
            failed_requests: Arc::clone(&self.failed_requests),
        }
    }
}

impl Default for BenchmarkCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_benchmark_report(stats: &BenchmarkStats) {
    println!("\n{}", "=".repeat(60));
    println!("PERFORMANCE BENCHMARK REPORT");
    println!("{}", "=".repeat(60));

    println!("\n📊 Request Statistics:");
    println!("  Total Requests:      {:>10}", stats.total_requests);
    println!("  Successful:          {:>10}", stats.successful_requests);
    println!("  Failed:              {:>10}", stats.failed_requests);
    println!(
        "  Success Rate:        {:>9.2}%",
        if stats.total_requests > 0 {
            (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
        } else {
            0.0
        }
    );

    println!("\n⏱️  Latency Statistics:");
    println!(
        "  Min Latency:         {:>10.3} ms",
        stats.min_latency.as_secs_f64() * 1000.0
    );
    println!(
        "  Max Latency:         {:>10.3} ms",
        stats.max_latency.as_secs_f64() * 1000.0
    );
    println!(
        "  Avg Latency:         {:>10.3} ms",
        stats.avg_latency.as_secs_f64() * 1000.0
    );
    println!(
        "  P50 Latency:         {:>10.3} ms",
        stats.p50_latency.as_secs_f64() * 1000.0
    );
    println!(
        "  P95 Latency:         {:>10.3} ms",
        stats.p95_latency.as_secs_f64() * 1000.0
    );
    println!(
        "  P99 Latency:         {:>10.3} ms",
        stats.p99_latency.as_secs_f64() * 1000.0
    );

    println!("\n🚀 Throughput:");
    println!("  Requests/sec:        {:>10.2}", stats.throughput_rps);
    println!(
        "  Total Duration:      {:>10.3} s",
        stats.total_duration.as_secs_f64()
    );

    println!("\n{}", "=".repeat(60));
}
