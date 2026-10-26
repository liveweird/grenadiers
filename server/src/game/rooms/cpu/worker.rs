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
    survival::{SurvivalPlan, WORK_ALLOWANCE, plan},
};
use grenadiers_core::game::Tick;

const WORKERS: usize = 2;
const QUEUE_CAPACITY: usize = 32;
pub(super) struct PlanResult {
    pub(super) plan: SurvivalPlan,
    pub(super) basis: Observation,
    pub(super) rng: CpuRng,
}

pub(super) struct PlanJob {
    pub(super) observation: Observation,
    pub(super) memory: UtilityMemory,
    pub(super) hazard_memory: ObservedHazardMemory,
    pub(super) settled_stop_tick: Tick,
    pub(super) rng: CpuRng,
    pub(super) live: Arc<AtomicBool>,
    pub(super) result: SyncSender<PlanResult>,
}

struct PlannerInner {
    sender: Mutex<Option<SyncSender<PlanJob>>>,
    workers: Mutex<Vec<JoinHandle<()>>>,
    _receiver: Arc<Mutex<Receiver<PlanJob>>>,
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
        let mut workers = Vec::with_capacity(worker_count);
        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let worker = thread::Builder::new()
                .name(format!("grenadiers-cpu-{index}"))
                .spawn(move || worker_loop(&receiver))
                .map_err(|_| ())?;
            workers.push(worker);
        }
        Ok(Self {
            inner: Arc::new(PlannerInner {
                sender: Mutex::new(Some(sender)),
                workers: Mutex::new(workers),
                _receiver: receiver,
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
}

fn worker_loop(receiver: &Arc<Mutex<Receiver<PlanJob>>>) {
    loop {
        let job = receiver
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .recv();
        let Ok(mut job) = job else {
            return;
        };
        if !job.live.load(Ordering::Acquire) {
            continue;
        }
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let Ok(plan) = plan(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            &mut job.rng,
            &mut work,
        ) else {
            continue;
        };
        if job.live.load(Ordering::Acquire) {
            let _ = job.result.try_send(PlanResult {
                plan,
                basis: job.observation,
                rng: job.rng,
            });
        }
    }
}
