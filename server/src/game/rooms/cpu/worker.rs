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
    survival::{
        PlantEscapeOrigin, SurvivalPlan, WORK_ALLOWANCE, plan_lairs_with_origin,
        plan_respawn_v22_with_origin, plan_respawn_v23_with_origin, plan_respawn_v24_with_origin,
        plan_respawn_v25_with_origin, plan_respawn_with_origin, plan_with_origin,
    },
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

pub(super) struct PlanResult {
    pub(super) plan: SurvivalPlan,
    pub(super) contact: Option<grenadiers_core::cpu::MobContactLease>,
    pub(super) basis: Observation,
    pub(super) rng: CpuRng,
    pub(super) escape_origin: Option<PlantEscapeOrigin>,
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
            report.dequeued += 1;
            let (status, spent, intent, planner_time) = execute_job(job, true);
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
        report
    }
}

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
    let planned = if job.observation.parts().rules.version == 9 {
        let plan = if job.policy == CpuPolicy::UtilityV25 {
            plan_respawn_v25_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                &mut work,
            )
        } else if job.policy == CpuPolicy::UtilityV24 {
            plan_respawn_v24_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                &mut work,
            )
        } else if job.policy == CpuPolicy::UtilityV23 {
            plan_respawn_v23_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                &mut work,
            )
        } else if job.policy == CpuPolicy::UtilityV22 {
            plan_respawn_v22_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                &mut work,
            )
        } else {
            plan_respawn_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                &mut work,
            )
        };
        plan.map(|plan| (plan.survival, plan.contact))
    } else if job.observation.parts().rules.version == 8 {
        plan_lairs_with_origin(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut job.rng,
            &mut work,
        )
        .map(|plan| (plan.survival, plan.contact))
    } else {
        plan_with_origin(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut job.rng,
            &mut work,
        )
        .map(|plan| (plan, None))
    };
    let Ok((plan, contact)) = planned else {
        return (
            JobStatus::Failed,
            work.spent(),
            None,
            started.map(|value| value.elapsed()),
        );
    };
    let planner_time = started.map(|value| value.elapsed());
    let intent = plan.control.intent;
    let escape_origin = job.escape_origin;
    if !job.live.load(Ordering::Acquire) {
        return (
            JobStatus::Cancelled,
            work.spent(),
            Some(intent),
            planner_time,
        );
    }
    let status = if job
        .result
        .try_send(PlanResult {
            plan,
            contact,
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
    (status, work.spent(), Some(intent), planner_time)
}
