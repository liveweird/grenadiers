use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
};
use std::thread::{self, JoinHandle};

use grenadiers_core::cpu::{
    CpuRng, Observation, WorkBudget,
    forecast::ObservedHazardMemory,
    policy::UtilityMemory,
    routing::ContinuationCertificate,
    survival::{PlantEscapeOrigin, SurvivalPlan, WORK_ALLOWANCE},
};
use grenadiers_core::game::Tick;
use grenadiers_core::room::CpuPolicy;
const WORKERS: usize = 2;
const QUEUE_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JobStatus {
    Delivered,
    Cancelled,
    Failed,
    Dropped,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::game::rooms) struct PumpReport {
    pub(in crate::game::rooms) dequeued: usize,
    pub(in crate::game::rooms) delivered: usize,
    pub(in crate::game::rooms) cancelled: usize,
    pub(in crate::game::rooms) failed: usize,
    pub(in crate::game::rooms) dropped: usize,
    pub(in crate::game::rooms) work: Vec<u32>,
    pub(in crate::game::rooms) planned_intents: Vec<grenadiers_core::cpu::Intent>,
    pub(in crate::game::rooms) planner_times: Vec<std::time::Duration>,
}

pub(super) enum WorkerPlan {
    Ordinary {
        plan: Box<SurvivalPlan>,
        contact: Option<grenadiers_core::cpu::MobContactLease>,
    },
    Continuation(Box<ContinuationCertificate>),
    TeleporterStep(Box<grenadiers_core::cpu::survival::TeleporterStepPlan>),
}

pub(super) struct PlanResult {
    pub(super) outcome: WorkerPlan,
    pub(super) basis: Observation,
    pub(super) rng: CpuRng,
    pub(super) escape_origin: Option<PlantEscapeOrigin>,
}

#[cfg(test)]
impl PlanResult {
    pub(super) fn ordinary(&self) -> &SurvivalPlan {
        match &self.outcome {
            WorkerPlan::Ordinary { plan, .. } => plan.as_ref(),
            WorkerPlan::Continuation(_) => panic!("expected ordinary worker result"),
            WorkerPlan::TeleporterStep(_) => panic!("expected ordinary worker result"),
        }
    }

    pub(super) fn ordinary_mut(&mut self) -> &mut SurvivalPlan {
        match &mut self.outcome {
            WorkerPlan::Ordinary { plan, .. } => plan.as_mut(),
            WorkerPlan::Continuation(_) => panic!("expected ordinary worker result"),
            WorkerPlan::TeleporterStep(_) => panic!("expected ordinary worker result"),
        }
    }
}

pub(super) struct PlanJob {
    pub(super) policy: CpuPolicy,
    pub(super) observation: Observation,
    pub(super) memory: UtilityMemory,
    pub(super) hazard_memory: ObservedHazardMemory,
    pub(super) settled_stop_tick: Tick,
    pub(super) rng: CpuRng,
    pub(super) escape_origin: Option<PlantEscapeOrigin>,
    pub(super) live: Arc<AtomicBool>,
    pub(super) result: SyncSender<PlanResult>,
}

struct PlannerInner {
    sender: Mutex<Option<SyncSender<PlanJob>>>,
    workers: Mutex<Vec<JoinHandle<()>>>,
    _receiver: Arc<Mutex<Receiver<PlanJob>>>,
    #[cfg(test)]
    trace: Arc<Mutex<PumpReport>>,
}

impl Drop for PlannerInner {
    fn drop(&mut self) {
        self.sender.get_mut().ok().and_then(Option::take);
        if let Ok(workers) = self.workers.get_mut() {
            for worker in workers.drain(..) {
                let _ = worker.join();
            }
        }
    }
}

#[derive(Clone)]
pub(in crate::game::rooms) struct CpuPlanner {
    inner: Arc<PlannerInner>,
}

impl CpuPlanner {
    pub(in crate::game::rooms) fn new() -> Result<Self, ()> {
        Self::with_worker_count(WORKERS)
    }

    fn with_worker_count(worker_count: usize) -> Result<Self, ()> {
        let (sender, receiver) = sync_channel(QUEUE_CAPACITY);
        let receiver = Arc::new(Mutex::new(receiver));
        #[cfg(test)]
        let trace = Arc::new(Mutex::new(PumpReport::default()));
        let mut workers = Vec::with_capacity(worker_count);
        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            #[cfg(test)]
            let trace = Arc::clone(&trace);
            let worker = thread::Builder::new()
                .name(format!("grenadiers-cpu-{index}"))
                .spawn(move || {
                    #[cfg(test)]
                    worker_loop(&receiver, &trace);
                    #[cfg(not(test))]
                    worker_loop(&receiver);
                })
                .map_err(|_| ())?;
            workers.push(worker);
        }
        Ok(Self {
            inner: Arc::new(PlannerInner {
                sender: Mutex::new(Some(sender)),
                workers: Mutex::new(workers),
                _receiver: receiver,
                #[cfg(test)]
                trace,
            }),
        })
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn without_workers() -> Self {
        Self::with_worker_count(0).expect("zero workers do not spawn")
    }

    pub(super) fn try_submit(&self, job: PlanJob) -> bool {
        let sender = self
            .inner
            .sender
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(sender) = sender.as_ref() else {
            return false;
        };
        match sender.try_send(job) {
            Ok(()) => true,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => false,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn pump_for_test(&self, limit: usize) -> PumpReport {
        assert!(
            self.inner
                .workers
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .is_empty(),
            "only a zero-worker planner may be pumped"
        );
        let mut report = PumpReport::default();
        for _ in 0..limit.min(QUEUE_CAPACITY) {
            let job = self
                .inner
                ._receiver
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .try_recv();
            let job = match job {
                Ok(job) => job,
                Err(
                    std::sync::mpsc::TryRecvError::Empty
                    | std::sync::mpsc::TryRecvError::Disconnected,
                ) => break,
            };
            record(&mut report, execute_job(job, true));
        }
        report
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn trace_for_test(&self) -> PumpReport {
        self.inner
            .trace
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

#[cfg(not(test))]
fn worker_loop(receiver: &Arc<Mutex<Receiver<PlanJob>>>) {
    loop {
        let job = receiver
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .recv();
        let Ok(job) = job else {
            return;
        };
        let _ = execute_job(job, false);
    }
}

#[cfg(test)]
fn worker_loop(receiver: &Arc<Mutex<Receiver<PlanJob>>>, trace: &Arc<Mutex<PumpReport>>) {
    loop {
        let job = receiver
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .recv();
        let Ok(job) = job else {
            return;
        };
        let result = execute_job(job, true);
        record(
            &mut trace.lock().unwrap_or_else(|error| error.into_inner()),
            result,
        );
    }
}

#[cfg(test)]
fn record(
    report: &mut PumpReport,
    (status, spent, intent, planner_time): (
        JobStatus,
        u32,
        Option<grenadiers_core::cpu::Intent>,
        Option<std::time::Duration>,
    ),
) {
    report.dequeued += 1;
    report.planner_times.extend(planner_time);
    match status {
        JobStatus::Delivered => report.delivered += 1,
        JobStatus::Cancelled => report.cancelled += 1,
        JobStatus::Failed => report.failed += 1,
        JobStatus::Dropped => report.dropped += 1,
    }
    report.work.push(spent);
    report.planned_intents.extend(intent);
}

fn execute_job(
    mut job: PlanJob,
    measure: bool,
) -> (
    JobStatus,
    u32,
    Option<grenadiers_core::cpu::Intent>,
    Option<std::time::Duration>,
) {
    if !job.live.load(Ordering::Acquire) {
        return (JobStatus::Cancelled, 0, None, None);
    }
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let started = measure.then(std::time::Instant::now);
    let planned = plan::plan(&mut job, &mut work);
    let Ok(outcome) = planned else {
        return (
            JobStatus::Failed,
            work.spent(),
            None,
            started.map(|value| value.elapsed()),
        );
    };
    let planner_time = started.map(|value| value.elapsed());
    let intent = match &outcome {
        WorkerPlan::Ordinary { plan, .. } => Some(plan.control.intent),
        WorkerPlan::Continuation(_) => None,
        WorkerPlan::TeleporterStep(_) => None,
    };
    let escape_origin = job.escape_origin;
    if !job.live.load(Ordering::Acquire) {
        return (JobStatus::Cancelled, work.spent(), intent, planner_time);
    }
    let status = if job
        .result
        .try_send(PlanResult {
            outcome,
            basis: job.observation,
            rng: job.rng,
            escape_origin,
        })
        .is_ok()
    {
        JobStatus::Delivered
    } else {
        JobStatus::Dropped
    };
    (status, work.spent(), intent, planner_time)
}

#[path = "worker_plan.rs"]
mod plan;
