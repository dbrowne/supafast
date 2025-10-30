use crossbeam_channel::{Receiver, Sender};
use diesel::prelude::*;
use std::thread;
use std::time::Instant;

use crate::benchmark::BenchmarkCollector;
use crate::error::WorkerError;
use crate::metrics::MetricsCollector;
use crate::models::{ResponseStatus, WorkRequest, WorkResponse};
use crate::pool::{DbConnection, DbPool};

pub struct Worker {
    worker_id: usize,
    db_pool: DbPool,
    work_queue: Receiver<(WorkRequest, Sender<WorkResponse>)>,
    cached_connection: Option<DbConnection>,
}

impl Worker {
    pub fn new(
        worker_id: usize,
        db_pool: DbPool,
        queue: Receiver<(WorkRequest, Sender<WorkResponse>)>,
    ) -> Self {
        Self {
            worker_id,
            db_pool,
            work_queue: queue,
            cached_connection: None,
        }
    }

    #[inline]
    fn get_connection(&mut self) -> Result<&mut DbConnection, WorkerError> {
        if self.cached_connection.is_none() {
            self.cached_connection = Some(self.db_pool.get()?);
        }

        Ok(self.cached_connection.as_mut().unwrap())
    }

    #[inline(always)]
    fn validate_request(&self, req: &WorkRequest) -> Result<(), WorkerError> {
        if req.id.is_empty() {
            return Err(WorkerError::ValidationError("ID cannot be empty"));
        }

        Ok(())
    }

    fn process_request_internal(
        &mut self,
        request: &WorkRequest,
    ) -> Result<WorkResponse, WorkerError> {
        self.validate_request(request)?;
        let conn = self.get_connection()?;

        diesel::sql_query("INSERT INTO your_table (id, created_at) VALUES ($1, NOW())")
            .bind::<diesel::sql_types::Text, _>(&request.id)
            .execute(conn)?;

        Ok(WorkResponse::success(request.id.clone()))
    }

    #[inline]
    fn process_request(&mut self, request: &WorkRequest) -> WorkResponse {
        match self.process_request_internal(request) {
            Ok(response) => response,
            Err(e) => {
                if cfg!(debug_assertions) {
                    eprintln!("Worker {} error: {}", self.worker_id, e);
                }

                let status = match e {
                    WorkerError::ValidationError(_) => ResponseStatus::Invalid,
                    WorkerError::ConnectionError(_) => {
                        self.cached_connection = None;
                        ResponseStatus::ConnectionError
                    }
                    WorkerError::DatabaseError(_) => ResponseStatus::Failed,
                    WorkerError::ProcessingError => ResponseStatus::Failed,
                };

                WorkResponse::failure(request.id.clone(), status)
            }
        }
    }

    pub fn run(&mut self) {
        println!("Worker {} started", self.worker_id);

        while let Ok((request, response_tx)) = self.work_queue.recv() {
            let result = self.process_request(&request);
            let _ = response_tx.send(result);
        }

        println!("Worker {} shutting down", self.worker_id);
    }
}

pub struct WorkerWithMetrics {
    worker: Worker,
    metrics: MetricsCollector,
    benchmark: Option<BenchmarkCollector>,
}

impl WorkerWithMetrics {
    pub fn new(
        worker_id: usize,
        db_pool: DbPool,
        queue: Receiver<(WorkRequest, Sender<WorkResponse>)>,
        metrics: MetricsCollector,
        benchmark: Option<BenchmarkCollector>,
    ) -> Self {
        Self {
            worker: Worker::new(worker_id, db_pool, queue),
            metrics,
            benchmark,
        }
    }

    pub fn run(&mut self) {
        println!("Worker {} started", self.worker.worker_id);

        while let Ok((request, response_tx)) = self.worker.work_queue.recv() {
            let start = Instant::now();
            let result = self.worker.process_request(&request);
            let latency = start.elapsed();

            // Track metrics
            if result.success {
                self.metrics.record_success();
            } else {
                self.metrics.record_failure();
            }

            // Track benchmark if enabled
            if let Some(ref benchmark) = self.benchmark {
                benchmark.record_request(latency, result.success);
            }

            let _ = response_tx.send(result);
        }

        println!("Worker {} shutting down", self.worker.worker_id);
    }
}

pub fn spawn_worker_pool(
    worker_count: usize,
    db_pool: DbPool,
    receiver: Receiver<(WorkRequest, Sender<WorkResponse>)>,
) -> Vec<thread::JoinHandle<()>> {
    (0..worker_count)
        .map(|worker_id| {
            let rx = receiver.clone();
            let pool = db_pool.clone();

            thread::Builder::new()
                .name(format!("worker-{}", worker_id))
                .spawn(move || {
                    let mut worker = Worker::new(worker_id, pool, rx);
                    worker.run();
                })
                .expect("Failed to spawn worker thread")
        })
        .collect()
}

pub fn spawn_worker_pool_with_metrics(
    worker_count: usize,
    db_pool: DbPool,
    receiver: Receiver<(WorkRequest, Sender<WorkResponse>)>,
    metrics: MetricsCollector,
    benchmark: Option<BenchmarkCollector>,
) -> Vec<thread::JoinHandle<()>> {
    (0..worker_count)
        .map(|worker_id| {
            let rx = receiver.clone();
            let pool = db_pool.clone();
            let metrics_clone = metrics.clone_handle();
            let benchmark_clone = benchmark.as_ref().map(|b| b.clone_handle());

            thread::Builder::new()
                .name(format!("worker-{}", worker_id))
                .spawn(move || {
                    let mut worker =
                        WorkerWithMetrics::new(worker_id, pool, rx, metrics_clone, benchmark_clone);
                    worker.run();
                })
                .expect("Failed to spawn worker thread")
        })
        .collect()
}
