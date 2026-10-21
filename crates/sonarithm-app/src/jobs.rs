// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The job queue: long work (exports now; imports and analyses later) runs on one worker
//! thread, first in first out, with progress events and cooperative cancellation.

use std::collections::BTreeMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use sonarithm_core::engine::CancelToken;

use crate::codes;
use crate::events::EventSink;
use crate::ipc::{Event, ExportResult, IpcError, JobInfo, JobKind, JobState};

/// The shortest gap between two progress events of one job (ADR-0017 §4: at most 10/s).
const PROGRESS_GAP: Duration = Duration::from_millis(100);

/// What a job produced.
#[derive(Debug)]
pub(crate) enum Output {
    Export(ExportResult),
}

/// Why a job did not produce its output.
#[derive(Debug)]
pub(crate) enum Failure {
    Cancelled,
    Failed(IpcError),
}

/// The work of one job.
pub(crate) type Work = Box<dyn FnOnce(&JobContext) -> Result<Output, Failure> + Send>;

/// What a running job can see: its cancellation and a progress reporter.
pub(crate) struct JobContext {
    id: u64,
    cancel: CancelToken,
    records: Records,
    sink: Arc<dyn EventSink>,
    last: Mutex<Option<Instant>>,
}

impl JobContext {
    /// Whether the user cancelled the job; the work should stop soon.
    pub(crate) fn cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Reports progress from 0 to 1, rate-limited.
    pub(crate) fn progress(&self, done: f32) {
        let done = done.clamp(0.0, 1.0);
        if let Some(job) = lock(&self.records).get_mut(&self.id) {
            job.progress = done;
        }
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        let now = Instant::now();
        if last.is_none_or(|at| now.duration_since(at) >= PROGRESS_GAP) {
            *last = Some(now);
            self.sink.emit(Event::JobProgress {
                id: self.id,
                progress: done,
            });
        }
    }
}

type Records = Arc<Mutex<BTreeMap<u64, Entry>>>;

#[derive(Debug)]
struct Entry {
    info: JobInfo,
    cancel: CancelToken,
}

impl std::ops::Deref for Entry {
    type Target = JobInfo;
    fn deref(&self) -> &JobInfo {
        &self.info
    }
}

impl std::ops::DerefMut for Entry {
    fn deref_mut(&mut self) -> &mut JobInfo {
        &mut self.info
    }
}

fn lock(records: &Records) -> MutexGuard<'_, BTreeMap<u64, Entry>> {
    records.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The queue and its worker thread.
pub(crate) struct JobQueue {
    records: Records,
    sender: Option<Sender<(u64, Work)>>,
    worker: Option<JoinHandle<()>>,
    next: u64,
}

impl JobQueue {
    /// A queue whose worker reports to `sink`.
    pub(crate) fn new(sink: Arc<dyn EventSink>) -> Self {
        let records: Records = Arc::default();
        let (sender, receiver) = mpsc::channel();
        let worker_records = Arc::clone(&records);
        let worker = std::thread::Builder::new()
            .name("sonarithm-jobs".to_owned())
            .spawn(move || work(&receiver, &worker_records, &sink))
            .ok();
        Self {
            records,
            sender: worker.is_some().then_some(sender),
            worker,
            next: 1,
        }
    }

    /// Queues `work` and returns the job as queued.
    pub(crate) fn submit(&mut self, kind: JobKind, work: Work) -> Result<JobInfo, IpcError> {
        let id = self.next;
        self.next += 1;
        let info = JobInfo {
            id,
            kind,
            state: JobState::Queued,
            progress: 0.0,
            export: None,
            error: None,
        };
        lock(&self.records).insert(
            id,
            Entry {
                info: info.clone(),
                cancel: CancelToken::new(),
            },
        );
        let sent = self
            .sender
            .as_ref()
            .is_some_and(|sender| sender.send((id, work)).is_ok());
        if !sent {
            lock(&self.records).remove(&id);
            return Err(IpcError::new(
                &codes::CONTROL_THREAD,
                "The job worker is not running, so the job could not start.",
            ));
        }
        Ok(info)
    }

    /// One job.
    pub(crate) fn status(&self, id: u64) -> Result<JobInfo, IpcError> {
        lock(&self.records)
            .get(&id)
            .map(|entry| entry.info.clone())
            .ok_or_else(|| unknown(id))
    }

    /// Every job, oldest first.
    pub(crate) fn list(&self) -> Vec<JobInfo> {
        lock(&self.records)
            .values()
            .map(|entry| entry.info.clone())
            .collect()
    }

    /// Cancels a job: a queued one at once, a running one when its work next checks.
    pub(crate) fn cancel(&self, id: u64, sink: &dyn EventSink) -> Result<JobInfo, IpcError> {
        let mut records = lock(&self.records);
        let entry = records.get_mut(&id).ok_or_else(|| unknown(id))?;
        entry.cancel.cancel();
        if entry.state == JobState::Queued {
            entry.state = JobState::Cancelled;
            sink.emit(Event::JobDone(entry.info.clone()));
        }
        Ok(entry.info.clone())
    }

    /// Waits until every queued job is done (tests and shutdown).
    pub(crate) fn wait_idle(&self, limit: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < limit {
            if lock(&self.records)
                .values()
                .all(|entry| entry.state.is_done())
            {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    }
}

impl Drop for JobQueue {
    fn drop(&mut self) {
        for entry in lock(&self.records).values() {
            entry.cancel.cancel();
        }
        self.sender = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn unknown(id: u64) -> IpcError {
    IpcError::new(&codes::UNKNOWN_JOB, format!("No job has the ID {id}."))
}

/// The worker: runs jobs in order until the queue is dropped.
fn work(receiver: &Receiver<(u64, Work)>, records: &Records, sink: &Arc<dyn EventSink>) {
    while let Ok((id, job)) = receiver.recv() {
        let cancel = {
            let mut records = lock(records);
            let Some(entry) = records.get_mut(&id) else {
                continue;
            };
            if entry.state != JobState::Queued {
                continue; // cancelled while queued
            }
            entry.state = JobState::Running;
            entry.cancel.clone()
        };
        sink.emit(Event::JobProgress { id, progress: 0.0 });
        let context = JobContext {
            id,
            cancel,
            records: Arc::clone(records),
            sink: Arc::clone(sink),
            last: Mutex::new(None),
        };
        let result = job(&context);
        let done = {
            let mut records = lock(records);
            let Some(entry) = records.get_mut(&id) else {
                continue;
            };
            match result {
                Ok(Output::Export(export)) => {
                    entry.state = JobState::Finished;
                    entry.progress = 1.0;
                    entry.export = Some(export);
                }
                Err(Failure::Cancelled) => entry.state = JobState::Cancelled,
                Err(Failure::Failed(error)) => {
                    entry.state = JobState::Failed;
                    entry.error = Some(error);
                }
            }
            entry.info.clone()
        };
        sink.emit(Event::JobDone(done));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Barrier;

    use super::*;
    use crate::events::CollectingSink;

    fn export() -> ExportResult {
        ExportResult {
            audio: "a.wav".into(),
            manifest: "a.render.json".into(),
            cues: None,
            duration_s: 1.0,
            sample_rate: 48_000,
            integrated_lufs: None,
            true_peak_dbtp: -1.0,
        }
    }

    #[test]
    fn jobs_run_in_order_and_report_done() {
        let sink = CollectingSink::new();
        let mut queue = JobQueue::new(Arc::new(sink.clone()));
        for _ in 0..3 {
            queue
                .submit(JobKind::Export, Box::new(|_| Ok(Output::Export(export()))))
                .unwrap();
        }
        assert!(queue.wait_idle(Duration::from_secs(5)));
        let done: Vec<u64> = sink
            .events()
            .into_iter()
            .filter_map(|event| match event {
                Event::JobDone(info) if info.state == JobState::Finished => Some(info.id),
                _ => None,
            })
            .collect();
        assert_eq!(done, vec![1, 2, 3]);
    }

    #[test]
    fn a_queued_job_cancels_at_once_and_a_running_one_when_it_checks() {
        let sink = CollectingSink::new();
        let mut queue = JobQueue::new(Arc::new(sink.clone()));
        let gate = Arc::new(Barrier::new(2));
        let inner = Arc::clone(&gate);
        let running = queue
            .submit(
                JobKind::Export,
                Box::new(move |context| {
                    inner.wait(); // started
                    inner.wait(); // cancelled by the test
                    if context.cancelled() {
                        Err(Failure::Cancelled)
                    } else {
                        Ok(Output::Export(export()))
                    }
                }),
            )
            .unwrap();
        let queued = queue
            .submit(JobKind::Export, Box::new(|_| Ok(Output::Export(export()))))
            .unwrap();
        gate.wait();
        assert_eq!(
            queue.cancel(queued.id, &sink).unwrap().state,
            JobState::Cancelled
        );
        queue.cancel(running.id, &sink).unwrap();
        gate.wait();
        assert!(queue.wait_idle(Duration::from_secs(5)));
        assert_eq!(queue.status(running.id).unwrap().state, JobState::Cancelled);
        assert_eq!(queue.status(queued.id).unwrap().state, JobState::Cancelled);
        assert_eq!(queue.status(99).unwrap_err().code, "SNR-APP-020");
    }

    #[test]
    fn a_failed_job_keeps_its_error() {
        let sink = CollectingSink::new();
        let mut queue = JobQueue::new(Arc::new(sink));
        let job = queue
            .submit(
                JobKind::Export,
                Box::new(|_| {
                    Err(Failure::Failed(IpcError::new(
                        &codes::EXPORT_FAILED,
                        "disk full",
                    )))
                }),
            )
            .unwrap();
        assert!(queue.wait_idle(Duration::from_secs(5)));
        let info = queue.status(job.id).unwrap();
        assert_eq!(info.state, JobState::Failed);
        assert_eq!(info.error.map(|e| e.code).as_deref(), Some("SNR-APP-022"));
    }
}
