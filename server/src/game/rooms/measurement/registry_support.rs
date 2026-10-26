use std::{
    collections::{BTreeMap, HashSet},
    future::Future,
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    thread,
    time::{Duration, Instant},
};

use futures_util::task::noop_waker_ref;
use grenadiers_core::protocol::Counter;
use serde_json::{Value, json};
use tokio::sync::Barrier as AsyncBarrier;

use super::summary_ns;
use crate::game::{
    rooms::{Admission, BindingKey, GameRooms, RoomError},
    sessions::ConnectionId,
};

pub(super) struct MeasuredAdmission {
    result: Result<Admission, RoomError>,
    first_poll: Duration,
    total: Duration,
    first_ready: bool,
}

#[derive(Default)]
pub(super) struct Observations {
    first_poll: Vec<Duration>,
    total: Vec<Duration>,
    pending: usize,
    ready: usize,
    pub(super) successes: usize,
    errors: BTreeMap<&'static str, usize>,
    pub(super) cleanup: Vec<Duration>,
}

#[derive(Default)]
pub(super) struct ObserverResult {
    pub(super) attempts: usize,
    pub(super) acquired_immediately: usize,
    pub(super) would_block: usize,
    pub(super) acquisition_after_block: Vec<Duration>,
}

pub(super) async fn measured_create(
    rooms: &Arc<GameRooms>,
    binding: BindingKey,
    now: Instant,
) -> MeasuredAdmission {
    let started = Instant::now();
    let mut future = Box::pin(rooms.create_sequenced(binding, Counter::new(1).unwrap(), now));
    let first_started = Instant::now();
    let first = Future::poll(future.as_mut(), &mut Context::from_waker(noop_waker_ref()));
    let first_poll = first_started.elapsed();
    let (result, first_ready) = match first {
        Poll::Ready(result) => (result, true),
        Poll::Pending => (future.await, false),
    };
    MeasuredAdmission {
        result,
        first_poll,
        total: started.elapsed(),
        first_ready,
    }
}

pub(super) async fn concurrent_wave(
    rooms: &Arc<GameRooms>,
    wave: usize,
    concurrency: usize,
) -> Vec<MeasuredAdmission> {
    let barrier = Arc::new(AsyncBarrier::new(concurrency + 1));
    let mut tasks = Vec::with_capacity(concurrency);
    for index in 0..concurrency {
        let rooms = Arc::clone(rooms);
        let barrier = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            measured_create(&rooms, binding(wave, concurrency, index), Instant::now()).await
        }));
    }
    barrier.wait().await;
    let mut results = Vec::with_capacity(concurrency);
    for task in tasks {
        results.push(task.await.expect("measurement worker did not panic"));
    }
    results
}

pub(super) async fn unmeasured_concurrent_wave(
    rooms: &Arc<GameRooms>,
    wave: usize,
    concurrency: usize,
) -> Vec<Result<Admission, RoomError>> {
    let barrier = Arc::new(AsyncBarrier::new(concurrency + 1));
    let mut tasks = Vec::with_capacity(concurrency);
    for index in 0..concurrency {
        let rooms = Arc::clone(rooms);
        let barrier = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            rooms
                .create_sequenced(
                    binding(wave, concurrency, index),
                    Counter::new(1).unwrap(),
                    Instant::now(),
                )
                .await
        }));
    }
    barrier.wait().await;
    let mut results = Vec::with_capacity(concurrency);
    for task in tasks {
        results.push(task.await.expect("contention worker did not panic"));
    }
    results
}

pub(super) fn spawn_observer(
    rooms: Arc<GameRooms>,
    stop: Arc<AtomicBool>,
    start: Arc<Barrier>,
) -> thread::JoinHandle<ObserverResult> {
    thread::spawn(move || {
        let mut result = ObserverResult::default();
        start.wait();
        while !stop.load(Ordering::Acquire) && result.attempts < 100_000 {
            result.attempts += 1;
            match rooms.registry.try_lock() {
                Ok(guard) => {
                    result.acquired_immediately += 1;
                    drop(guard);
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    result.would_block += 1;
                    let started = Instant::now();
                    let guard = rooms.lock();
                    result.acquisition_after_block.push(started.elapsed());
                    drop(guard);
                }
                Err(std::sync::TryLockError::Poisoned(error)) => drop(error.into_inner()),
            }
            thread::yield_now();
        }
        result
    })
}

pub(super) fn observe(
    observations: &mut Observations,
    sample: MeasuredAdmission,
    admissions: &mut Vec<Admission>,
) {
    observations.first_poll.push(sample.first_poll);
    observations.total.push(sample.total);
    if sample.first_ready {
        observations.ready += 1;
    } else {
        observations.pending += 1;
    }
    match sample.result {
        Ok(admission) => {
            observations.successes += 1;
            admissions.push(admission);
        }
        Err(error) => *observations.errors.entry(error_name(error)).or_insert(0) += 1,
    }
}

pub(super) fn observation_json(observations: &Observations) -> Value {
    json!({
        "firstPoll": optional_summary(&observations.first_poll),
        "entireAwait": optional_summary(&observations.total),
        "firstPollOutcomes": { "pending": observations.pending, "ready": observations.ready },
        "successes": observations.successes,
        "errorsByEnum": observations.errors,
        "cleanupDuration": optional_summary(&observations.cleanup),
    })
}

pub(super) fn optional_summary(samples: &[Duration]) -> Value {
    if samples.is_empty() {
        json!({ "count": 0, "p50Ns": null, "p95Ns": null, "p99Ns": null, "maxNs": null })
    } else {
        summary_ns(samples)
    }
}

pub(super) fn assert_unique_and_mapped(rooms: &GameRooms, admissions: &[Admission]) {
    let ids = admissions
        .iter()
        .map(|admission| admission.room_id.clone())
        .collect::<HashSet<_>>();
    let codes = admissions
        .iter()
        .map(|admission| admission.room.room_code.clone())
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), admissions.len());
    assert_eq!(codes.len(), admissions.len());
    let registry = rooms.lock();
    assert_eq!(registry.rooms.len(), admissions.len());
    assert_eq!(registry.codes.len(), admissions.len());
    for admission in admissions {
        assert_eq!(
            registry.codes.get(&admission.room.room_code),
            Some(&admission.room_id)
        );
    }
}

pub(super) async fn cleanup(rooms: Arc<GameRooms>, admissions: Vec<Admission>) -> Duration {
    let started = Instant::now();
    drop(admissions);
    rooms.stop();
    rooms.wait_empty().await;
    let elapsed = started.elapsed();
    let registry = rooms.lock();
    assert!(registry.rooms.is_empty());
    assert!(registry.codes.is_empty());
    elapsed
}

pub(super) fn binding(wave: usize, concurrency: usize, index: usize) -> BindingKey {
    let connection = wave
        .checked_mul(64)
        .and_then(|value| value.checked_add(concurrency))
        .and_then(|value| value.checked_add(index + 1))
        .unwrap();
    BindingKey {
        connection_id: ConnectionId(u64::try_from(connection).unwrap()),
        generation: 1,
    }
}

pub(super) fn os_seed_source() -> crate::round_seed::RoundSeedSource {
    crate::round_seed::RoundSeedSource::from_optional_path(None)
        .expect("OS entropy source construction cannot fail")
}

pub(super) fn merge_observer(target: &mut ObserverResult, observed: ObserverResult) {
    target.attempts += observed.attempts;
    target.acquired_immediately += observed.acquired_immediately;
    target.would_block += observed.would_block;
    target
        .acquisition_after_block
        .extend(observed.acquisition_after_block);
}

pub(super) const fn error_name(error: RoomError) -> &'static str {
    match error {
        RoomError::RateLimited => "RateLimited",
        RoomError::CapacityReached => "CapacityReached",
        RoomError::RoomNotFound => "RoomNotFound",
        RoomError::RoomFull => "RoomFull",
        RoomError::StaleRoom => "StaleRoom",
        RoomError::StaleSettings => "StaleSettings",
        RoomError::SettingsLocked => "SettingsLocked",
        RoomError::InvalidPhase => "InvalidPhase",
        RoomError::DuplicateAction => "DuplicateAction",
        RoomError::ResumeRejected => "ResumeRejected",
    }
}
