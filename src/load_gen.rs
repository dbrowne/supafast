use crossbeam_channel::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::models::{WorkRequest, WorkResponse};

#[derive(Debug, Clone)]
pub enum LoadPattern {
    Constant {
        rps: u64,
    },
    Burst {
        rps: u64,
        duration_secs: u64,
    },
    Ramp {
        start_rps: u64,
        end_rps: u64,
        duration_secs: u64,
    },
    Sine {
        base_rps: u64,
        amplitude: u64,
        period_secs: u64,
    },
}

pub struct LoadGenerator {
    pattern: LoadPattern,
    total_requests: u64,
}

impl LoadGenerator {
    pub fn new(pattern: LoadPattern, total_requests: u64) -> Self {
        Self {
            pattern,
            total_requests,
        }
    }

    pub fn generate<F>(
        &self,
        work_sender: Sender<(WorkRequest, Sender<WorkResponse>)>,
        request_factory: F,
    ) -> Duration
    where
        F: FnMut(u64) -> WorkRequest,
    {
        let start = Instant::now();

        match self.pattern {
            LoadPattern::Constant { rps } => {
                self.generate_constant(rps, work_sender, request_factory)
            }
            LoadPattern::Burst { rps, duration_secs } => {
                self.generate_burst(rps, duration_secs, work_sender, request_factory)
            }
            LoadPattern::Ramp {
                start_rps,
                end_rps,
                duration_secs,
            } => self.generate_ramp(
                start_rps,
                end_rps,
                duration_secs,
                work_sender,
                request_factory,
            ),
            LoadPattern::Sine {
                base_rps,
                amplitude,
                period_secs,
            } => self.generate_sine(
                base_rps,
                amplitude,
                period_secs,
                work_sender,
                request_factory,
            ),
        }

        start.elapsed()
    }

    fn generate_constant<F>(
        &self,
        rps: u64,
        work_sender: Sender<(WorkRequest, Sender<WorkResponse>)>,
        mut request_factory: F,
    ) where
        F: FnMut(u64) -> WorkRequest,
    {
        let interval = Duration::from_secs_f64(1.0 / rps as f64);

        for i in 0..self.total_requests {
            let request = request_factory(i);
            let (response_tx, _response_rx) = crossbeam_channel::bounded(1);

            if work_sender.send((request, response_tx)).is_err() {
                break;
            }

            thread::sleep(interval);
        }
    }

    fn generate_burst<F>(
        &self,
        rps: u64,
        duration_secs: u64,
        work_sender: Sender<(WorkRequest, Sender<WorkResponse>)>,
        mut request_factory: F,
    ) where
        F: FnMut(u64) -> WorkRequest,
    {
        let interval = Duration::from_secs_f64(1.0 / rps as f64);
        let start = Instant::now();
        let burst_duration = Duration::from_secs(duration_secs);
        let mut sent = 0;

        while sent < self.total_requests && start.elapsed() < burst_duration {
            let request = request_factory(sent);
            let (response_tx, _response_rx) = crossbeam_channel::bounded(1);

            if work_sender.send((request, response_tx)).is_err() {
                break;
            }

            sent += 1;
            thread::sleep(interval);
        }
    }

    fn generate_ramp<F>(
        &self,
        start_rps: u64,
        end_rps: u64,
        duration_secs: u64,
        work_sender: Sender<(WorkRequest, Sender<WorkResponse>)>,
        mut request_factory: F,
    ) where
        F: FnMut(u64) -> WorkRequest,
    {
        let start = Instant::now();
        let total_duration = Duration::from_secs(duration_secs);
        let mut sent = 0;

        while sent < self.total_requests {
            let elapsed = start.elapsed().as_secs_f64();
            let progress = (elapsed / total_duration.as_secs_f64()).min(1.0);

            let current_rps = start_rps as f64 + (end_rps as f64 - start_rps as f64) * progress;
            let interval = Duration::from_secs_f64(1.0 / current_rps);

            let request = request_factory(sent);
            let (response_tx, _response_rx) = crossbeam_channel::bounded(1);

            if work_sender.send((request, response_tx)).is_err() {
                break;
            }

            sent += 1;
            thread::sleep(interval);

            if start.elapsed() >= total_duration {
                break;
            }
        }
    }

    fn generate_sine<F>(
        &self,
        base_rps: u64,
        amplitude: u64,
        period_secs: u64,
        work_sender: Sender<(WorkRequest, Sender<WorkResponse>)>,
        mut request_factory: F,
    ) where
        F: FnMut(u64) -> WorkRequest,
    {
        let start = Instant::now();
        let mut sent = 0;

        while sent < self.total_requests {
            let elapsed = start.elapsed().as_secs_f64();
            let phase = (elapsed / period_secs as f64) * 2.0 * std::f64::consts::PI;
            let current_rps = base_rps as f64 + amplitude as f64 * phase.sin();
            let current_rps = current_rps.max(1.0);

            let interval = Duration::from_secs_f64(1.0 / current_rps);

            let request = request_factory(sent);
            let (response_tx, _response_rx) = crossbeam_channel::bounded(1);

            if work_sender.send((request, response_tx)).is_err() {
                break;
            }

            sent += 1;
            thread::sleep(interval);
        }
    }
}

pub fn spawn_load_generator<F>(
    pattern: LoadPattern,
    total_requests: u64,
    work_sender: Sender<(WorkRequest, Sender<WorkResponse>)>,
    request_factory: F,
) -> thread::JoinHandle<Duration>
where
    F: FnMut(u64) -> WorkRequest + Send + 'static,
{
    thread::Builder::new()
        .name("load-generator".to_string())
        .spawn(move || {
            let generator = LoadGenerator::new(pattern, total_requests);
            generator.generate(work_sender, request_factory)
        })
        .expect("Failed to spawn load generator thread")
}
