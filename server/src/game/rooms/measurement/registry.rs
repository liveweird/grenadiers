use std::{
    collections::BTreeMap,
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use serde_json::json;

use super::{
    metadata,
    registry_support::{
        Observations, ObserverResult, assert_unique_and_mapped, binding, cleanup, concurrent_wave,
        error_name, measured_create, merge_observer, observation_json, observe, optional_summary,
        os_seed_source, spawn_observer, unmeasured_concurrent_wave,
    },
    require_release, summary_ns,
};
use crate::game::rooms::GameRooms;

const SEQUENTIAL_REPETITIONS: usize = 16;
const CONCURRENT_WAVES: usize = 8;
const CONTENTION_WAVES: usize = 16;
const BATCH_SIZES: [usize; 3] = [1, 8, 32];
const CONCURRENCY_LEVELS: [usize; 5] = [2, 4, 8, 16, 32];

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "isolated release registry latency measurement; coordinate before running"]
async fn sequential_registry_admission_measurement() {
    require_release();
    let mut batches = Vec::new();
    for size in BATCH_SIZES {
        let mut first = Observations::default();
        let mut remaining = Observations::default();
        for repetition in 0..SEQUENTIAL_REPETITIONS {
            let rooms = GameRooms::new(Instant::now(), os_seed_source());
            let mut admissions = Vec::with_capacity(size);
            for index in 0..size {
                let measured =
                    measured_create(&rooms, binding(repetition, size, index), Instant::now()).await;
                let target = if index == 0 {
                    &mut first
                } else {
                    &mut remaining
                };
                observe(target, measured, &mut admissions);
            }
            assert_unique_and_mapped(&rooms, &admissions);
            first.cleanup.push(cleanup(rooms, admissions).await);
        }
        assert_eq!(
            first.successes + remaining.successes,
            size * SEQUENTIAL_REPETITIONS
        );
        batches.push(json!({
            "batchSize": size,
            "repetitions": SEQUENTIAL_REPETITIONS,
            "firstCreation": observation_json(&first),
            "remainingCreations": observation_json(&remaining),
        }));
    }
    super::emit(
        "sequential-registry-admission",
        json!({
            "metadata": metadata(),
            "workload": "real GameRooms::create_sequenced admissions; each batch has a fresh registry",
            "firstPollInterpretation": "upper bound including registry lock acquisition, guarded identity and owner construction, insertion, owner spawn, and admission command submission",
            "firstPollIsExactMutexHoldTime": false,
            "batches": batches,
            "cleanupCompleted": true,
            "finalRegistrySizes": { "rooms": 0, "codes": 0 },
        }),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "isolated release barrier-released registry measurement; coordinate before running"]
async fn barrier_released_registry_admission_measurement() {
    require_release();
    let mut levels = Vec::new();
    for concurrency in CONCURRENCY_LEVELS {
        let mut observations = Observations::default();
        for wave in 0..CONCURRENT_WAVES {
            let rooms = GameRooms::new(Instant::now(), os_seed_source());
            let measured = concurrent_wave(&rooms, wave, concurrency).await;
            let mut admissions = Vec::with_capacity(concurrency);
            for sample in measured {
                observe(&mut observations, sample, &mut admissions);
            }
            assert_unique_and_mapped(&rooms, &admissions);
            observations.cleanup.push(cleanup(rooms, admissions).await);
        }
        assert_eq!(observations.successes, concurrency * CONCURRENT_WAVES);
        levels.push(json!({
            "executorWorkerThreads": 4,
            "barrierReleasedTaskCount": concurrency,
            "firstPollExecutionConcurrencyUpperBound": concurrency.min(4),
            "waves": CONCURRENT_WAVES,
            "actual": observation_json(&observations),
        }));
    }
    super::emit(
        "barrier-released-registry-admission",
        json!({
            "metadata": metadata(),
            "workload": "barrier-released create_sequenced task waves within the 32-room capacity and 32-create burst on a fixed four-thread Tokio executor",
            "firstPollInterpretation": "upper bound including registry lock acquisition, guarded identity and owner construction, insertion, owner spawn, and admission command submission",
            "firstPollIsExactMutexHoldTime": false,
            "perTaskTimingStartsWhenTaskIsScheduledAfterBarrier": true,
            "perTaskTimingExcludesBarrierToSchedulingDelay": true,
            "barrierReleaseProvesSimultaneousMutexContention": false,
            "observedMutexContentionReportedSeparately": true,
            "levels": levels,
            "cleanupCompleted": true,
            "finalRegistrySizes": { "rooms": 0, "codes": 0 },
        }),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "isolated release registry contention observation; coordinate before running"]
async fn actual_registry_mutex_contention_observation() {
    require_release();
    let mut combined = ObserverResult::default();
    let mut successes = 0;
    let mut errors = BTreeMap::new();
    let mut cleanup_timings = Vec::with_capacity(CONTENTION_WAVES);
    for wave in 0..CONTENTION_WAVES {
        let rooms = GameRooms::new(Instant::now(), os_seed_source());
        let stop = Arc::new(AtomicBool::new(false));
        let start = Arc::new(Barrier::new(2));
        let observer = spawn_observer(Arc::clone(&rooms), Arc::clone(&stop), Arc::clone(&start));
        start.wait();
        let results = unmeasured_concurrent_wave(&rooms, wave, 32).await;
        stop.store(true, Ordering::Release);
        let observed = observer.join().expect("registry observer did not panic");
        merge_observer(&mut combined, observed);
        let mut admissions = Vec::with_capacity(32);
        for result in results {
            match result {
                Ok(admission) => {
                    successes += 1;
                    admissions.push(admission);
                }
                Err(error) => *errors.entry(error_name(error)).or_insert(0) += 1,
            }
        }
        assert_unique_and_mapped(&rooms, &admissions);
        cleanup_timings.push(cleanup(rooms, admissions).await);
    }
    assert_eq!(successes, CONTENTION_WAVES * 32);
    super::emit(
        "actual-registry-mutex-contention-observation",
        json!({
            "metadata": metadata(),
            "workload": "bounded observer probes the actual registry mutex around barrier-released 32-task waves on a four-thread executor",
            "observationWindow": "before task allocation/spawn through completion of all full admissions, including work outside the registry prefixes",
            "attemptCountsAreWaveContentionRate": false,
            "observerPerturbsWorkload": true,
            "notProductionWaitSamples": true,
            "principalAdmissionLatencyRecorded": false,
            "executorWorkerThreads": 4,
            "barrierReleasedTaskCount": 32,
            "firstPollExecutionConcurrencyUpperBound": 4,
            "barrierReleaseProvesSimultaneousMutexContention": false,
            "waves": CONTENTION_WAVES,
            "successes": successes,
            "errorsByEnum": errors,
            "observer": {
                "tryLockAttempts": combined.attempts,
                "acquiredImmediately": combined.acquired_immediately,
                "wouldBlock": combined.would_block,
                "subsequentLockAcquisition": optional_summary(&combined.acquisition_after_block),
            },
            "cleanupDuration": summary_ns(&cleanup_timings),
            "cleanupCompleted": true,
            "finalRegistrySizes": { "rooms": 0, "codes": 0 },
        }),
    );
}
