mod benchmark;
mod config;
mod error;
mod load_gen;
mod metrics;
mod models;
mod pool;
mod worker;

use benchmark::{print_benchmark_report, BenchmarkCollector};
use config::ConfigManager;
use crossbeam_channel::bounded;
use load_gen::{spawn_load_generator, LoadPattern};
use metrics::MetricsCollector;
use models::{WorkRequest, WorkResponse};
use pool::create_pool;
use worker::spawn_worker_pool_with_metrics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/database".to_string());

    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!("🚀 Starting performance benchmark skeleton");
    println!("📊 Worker count: {}", worker_count);

    let pool = create_pool(&database_url, worker_count)?;

    let queue_capacity = worker_count * 100;
    let (tx, rx) =
        bounded::<(WorkRequest, crossbeam_channel::Sender<WorkResponse>)>(queue_capacity);

    // Create metrics collector
    let metrics = MetricsCollector::new();

    // Create benchmark collector
    let benchmark = BenchmarkCollector::new();

    // Create shared config
    let config = ConfigManager::new();

    // Spawn workers with metrics and benchmarking
    let handles = spawn_worker_pool_with_metrics(
        worker_count,
        pool,
        rx,
        metrics.clone_handle(),
        Some(benchmark.clone_handle()),
    );

    println!("✅ Worker pool started with {} workers\n", worker_count);

    // Choose a load pattern - modify this for different tests
    let load_pattern = LoadPattern::Constant { rps: 100 };
    // let load_pattern = LoadPattern::Burst { rps: 500, duration_secs: 10 };
    // let load_pattern = LoadPattern::Ramp { start_rps: 10, end_rps: 200, duration_secs: 30 };
    // let load_pattern = LoadPattern::Sine { base_rps: 100, amplitude: 50, period_secs: 20 };

    let total_requests = 1000;

    println!("📈 Load pattern: {:?}", load_pattern);
    println!("📦 Total requests: {}\n", total_requests);

    // Spawn load generator
    let load_handle =
        spawn_load_generator(load_pattern, total_requests, tx.clone(), |i| WorkRequest {
            id: format!("req-{}", i),
        });

    // Wait for load generation to complete
    let generation_time = load_handle.join().expect("Load generator panicked");
    println!(
        "⏱️  Load generation completed in {:.2}s\n",
        generation_time.as_secs_f64()
    );

    // Give workers time to process remaining requests
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Print metrics
    let snapshot = metrics.get_snapshot();
    println!("📊 Metrics Summary:");
    println!("  Processed: {}", snapshot.total_processed);
    println!("  Succeeded: {}", snapshot.total_succeeded);
    println!("  Failed:    {}", snapshot.total_failed);

    // Print benchmark report
    let stats = benchmark.get_stats();
    print_benchmark_report(&stats);

    // Example: Update config at runtime
    println!("\n🔧 Runtime config update example:");
    config.update_config(5, 10000, true);
    println!("  Max retries: {}", config.get_max_retries());
    println!("  Timeout:     {}ms", config.get_timeout_ms());
    println!("  Enabled:     {}", config.is_enabled());

    // Graceful shutdown
    println!("\n🛑 Shutting down...");
    drop(tx);
    for handle in handles {
        let _ = handle.join();
    }

    println!("✅ Shutdown complete");

    Ok(())
}
