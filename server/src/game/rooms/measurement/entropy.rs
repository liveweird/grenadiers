use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use grenadiers_core::protocol::Counter;
use serde_json::{Value, json};

use super::{metadata, require_release, summary_ns};
use crate::{
    game::{
        rooms::{BindingKey, GameRooms},
        sessions::ConnectionId,
    },
    round_seed::RoundSeedSource,
};

const BATCHES: usize = 128;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "isolated release entropy measurement; coordinate before running"]
async fn real_entropy_requests_by_size_measurement() {
    require_release();
    let timings = Arc::new(Mutex::new(BTreeMap::<usize, Vec<Duration>>::new()));
    let round_seed_requests = Arc::new(AtomicUsize::new(0));
    let mut cleanup_timings = Vec::with_capacity(BATCHES);
    let mut successes = 0;
    let mut errors = BTreeMap::new();

    for batch in 0..BATCHES {
        let room_timings = Arc::clone(&timings);
        let seed_timings = Arc::clone(&timings);
        let seed_requests = Arc::clone(&round_seed_requests);
        let seed_source = RoundSeedSource::with_entropy(move |bytes| {
            seed_requests.fetch_add(1, Ordering::Relaxed);
            timed_fill(&seed_timings, bytes)
        });
        let rooms = GameRooms::with_random_fill_and_round_seed(
            Instant::now(),
            move |bytes| timed_fill(&room_timings, bytes),
            seed_source,
        );
        let result = rooms
            .create_sequenced(binding(batch), Counter::new(1).unwrap(), Instant::now())
            .await;
        let cleanup_started = Instant::now();
        match result {
            Ok(admission) => {
                successes += 1;
                drop(admission);
            }
            Err(error) => {
                *errors
                    .entry(super::registry_support::error_name(error))
                    .or_insert(0) += 1;
            }
        }
        rooms.stop();
        rooms.wait_empty().await;
        cleanup_timings.push(cleanup_started.elapsed());
        assert_finally_empty(&rooms);
    }

    let timings = Arc::try_unwrap(timings)
        .unwrap_or_else(|_| panic!("all entropy timing closures were dropped"))
        .into_inner()
        .unwrap_or_else(|error| error.into_inner());
    assert_eq!(timings.keys().copied().collect::<Vec<_>>(), [5, 8, 32]);
    for values in timings.values() {
        assert_eq!(values.len(), BATCHES);
    }
    let round_seed_requests = round_seed_requests.load(Ordering::Relaxed);
    assert_eq!(round_seed_requests, 0);
    super::emit(
        "real-entropy-requests-by-size",
        json!({
            "metadata": metadata(),
            "workload": "real getrandom::fill calls through GameRooms and RoundSeedSource test seams",
            "batchCount": BATCHES,
            "successes": successes,
            "errorsByEnum": errors,
            "roundSeedRequestsBeforeActualStart": round_seed_requests,
            "principalAdmissionLatencyRecorded": false,
            "requestBytes": summaries(&timings),
            "bytesRecorded": false,
            "cleanupDuration": summary_ns(&cleanup_timings),
            "cleanupCompleted": true,
            "finalRegistrySizes": { "rooms": 0, "codes": 0 },
        }),
    );
    assert_eq!(successes, BATCHES);
}

fn timed_fill(timings: &Mutex<BTreeMap<usize, Vec<Duration>>>, bytes: &mut [u8]) -> Result<(), ()> {
    let size = bytes.len();
    assert!(
        matches!(size, 5 | 8 | 32),
        "unexpected entropy request size"
    );
    let started = Instant::now();
    let result = getrandom::fill(bytes).map_err(|_| ());
    let elapsed = started.elapsed();
    timings
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .entry(size)
        .or_default()
        .push(elapsed);
    result
}

fn summaries(timings: &BTreeMap<usize, Vec<Duration>>) -> Value {
    Value::Object(
        timings
            .iter()
            .map(|(size, values)| (size.to_string(), summary_ns(values)))
            .collect(),
    )
}

fn binding(index: usize) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(u64::try_from(index + 1).unwrap()),
        generation: 1,
    }
}

fn assert_finally_empty(rooms: &GameRooms) {
    let registry = rooms.lock();
    assert!(registry.rooms.is_empty());
    assert!(registry.codes.is_empty());
}
