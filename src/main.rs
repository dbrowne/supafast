mod config;
mod error;
mod metrics;
mod models;
mod pool;
mod worker;

use config::ConfigManager;
use crossbeam_channel::bounded;
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

    let pool = create_pool(&database_url, worker_count)?;

    let queue_capacity = worker_count * 10;
    let (tx, rx) =
        bounded::<(WorkRequest, crossbeam_channel::Sender<WorkResponse>)>(queue_capacity);

    // Create metrics collector
    let metrics = MetricsCollector::new();

    // Create shared config
    let config = ConfigManager::new();

    // Spawn workers with metrics
    let handles = spawn_worker_pool_with_metrics(worker_count, pool, rx, metrics.clone_handle());

    println!("Worker pool started with {} workers", worker_count);

    // Example: Send work to the pool
    let (response_tx, response_rx) = bounded(1);
    let request = WorkRequest {
        id: "123".to_string(),
    };

    tx.send((request, response_tx))?;

    match response_rx.recv()? {
        response if response.success => {
            println!("Success: {:?}", response.status);
        }
        response => {
            eprintln!("Failed: {:?}", response.status);
        }
    }

    // Print metrics
    let snapshot = metrics.get_snapshot();
    println!(
        "Metrics - Processed: {}, Succeeded: {}, Failed: {}",
        snapshot.total_processed, snapshot.total_succeeded, snapshot.total_failed
    );

    // Example: Update config at runtime
    config.update_config(5, 10000, true);
    println!("Config updated - Max retries: {}", config.get_max_retries());

    // Graceful shutdown
    drop(tx);
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
