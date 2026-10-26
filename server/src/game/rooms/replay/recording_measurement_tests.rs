use std::{
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use grenadiers_core::{
    game::{Direction, SeatId},
    room::MatchPhase,
};
use serde_json::{Value, json};

use super::{
    record::{MAX_RECORDS, Record},
    runner::identity_for_profile,
};
use crate::{
    game::rooms::{
        match_runtime::{MatchAction, MatchRuntime},
        measurement,
        profile::RoomProfile,
    },
    round_seed::RoundSeedSource,
};

const CONSTRUCTION_SAMPLES: usize = 512;
const WORKLOAD_SAMPLES: usize = 128;
const WORKLOAD_TICKS: usize = 220;

#[test]
#[ignore = "isolated release measurement; coordinate before running"]
fn production_journal_allocation_measurement() {
    measurement::require_release();
    let source_identity = required_metadata("GRENADIERS_MEASUREMENT_SOURCE");
    let toolchain = required_metadata("GRENADIERS_MEASUREMENT_TOOLCHAIN");

    let mut disabled_construction = Vec::with_capacity(CONSTRUCTION_SAMPLES);
    let mut recorded_construction = Vec::with_capacity(CONSTRUCTION_SAMPLES);
    let mut observed_capacity = None;
    for _ in 0..CONSTRUCTION_SAMPLES {
        let identity = identity_for_profile(RoomProfile::ExpandedV5);
        let seeds = deterministic_source();
        let started = Instant::now();
        let runtime = MatchRuntime::new(identity, seeds);
        black_box(&runtime);
        disabled_construction.push(started.elapsed());
        assert!(!runtime.recording_enabled_for_test());
        drop(runtime);

        let identity = identity_for_profile(RoomProfile::ExpandedV5);
        let seeds = deterministic_source();
        let started = Instant::now();
        let runtime = MatchRuntime::new_recording(identity, seeds);
        black_box(&runtime);
        recorded_construction.push(started.elapsed());
        observed_capacity = Some(runtime.replay_journal().records.capacity());
        drop(runtime);
    }

    let mut disabled_workload = Vec::with_capacity(WORKLOAD_SAMPLES);
    let mut recorded_workload = Vec::with_capacity(WORKLOAD_SAMPLES);
    let mut final_record_count = 0;
    for sample in 0..WORKLOAD_SAMPLES {
        let mut disabled = MatchRuntime::new(
            identity_for_profile(RoomProfile::ExpandedV5),
            deterministic_source(),
        );
        let mut recorded = MatchRuntime::new_recording(
            identity_for_profile(RoomProfile::ExpandedV5),
            deterministic_source(),
        );
        if sample.is_multiple_of(2) {
            disabled_workload.push(measure_workload(&mut disabled));
            recorded_workload.push(measure_workload(&mut recorded));
        } else {
            recorded_workload.push(measure_workload(&mut recorded));
            disabled_workload.push(measure_workload(&mut disabled));
        }
        assert_eq!(disabled.replay_inspection(), recorded.replay_inspection());
        assert_eq!(
            disabled.publication_revision_for_test(),
            recorded.publication_revision_for_test()
        );
        final_record_count = recorded.replay_journal().len();
        assert!(final_record_count < MAX_RECORDS);
        drop(disabled);
        drop(recorded);
    }

    let vector_capacity = observed_capacity.expect("construction samples exist");
    let record_bytes = size_of::<Record>();
    measurement::emit(
        "production-journal-allocation",
        json!({
            "sourceIdentity": source_identity,
            "toolchain": toolchain,
            "invocationDeclaredCargoProfile": "release",
            "debugAssertions": cfg!(debug_assertions),
            "roomProfile": "ExpandedV5",
            "recordingPolicy": "production-disabled-test-opt-in",
            "construction": {
                "disabled": duration_summary(&disabled_construction),
                "recorded": duration_summary(&recorded_construction),
            },
            "workload": {
                "description": "initial notice, two occupants, two ready actions, 220 ticks, periodic movement",
                "disabledTimings": duration_summary(&disabled_workload),
                "recordedTimings": duration_summary(&recorded_workload),
                "ticksPerSample": WORKLOAD_TICKS,
                "finalRecordCount": final_record_count,
                "stateEquivalentOutsideTiming": true,
            },
            "allocation": {
                "recordBytes": record_bytes,
                "maxRecords": MAX_RECORDS,
                "maxReservedBytes": MAX_RECORDS * record_bytes,
                "disabled": {
                    "journalAllocated": false,
                    "vectorReservedBytes": 0,
                },
                "recorded": {
                    "journalAllocated": true,
                    "vectorCapacityRecords": vector_capacity,
                    "vectorReservedBytes": vector_capacity * record_bytes,
                },
            },
        }),
    );
}

fn measure_workload(runtime: &mut MatchRuntime) -> Duration {
    let started = Instant::now();
    exercise(runtime);
    started.elapsed()
}

fn exercise(runtime: &mut MatchRuntime) {
    runtime.initial_notice().unwrap();
    for seat in SeatId::ALL {
        runtime.occupy(seat).unwrap();
        runtime.action(seat, 1, MatchAction::Ready(true)).unwrap();
    }
    for index in 0..WORKLOAD_TICKS {
        if runtime.lifecycle().phase() == MatchPhase::Running && index.is_multiple_of(11) {
            let epoch = runtime.lifecycle().round_attempt();
            for seat in SeatId::ALL {
                runtime
                    .action(seat, epoch, MatchAction::Movement(Some(Direction::East)))
                    .unwrap();
            }
        }
        runtime.tick().unwrap();
    }
}

fn deterministic_source() -> RoundSeedSource {
    RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(0x5a);
        Ok(())
    })
}

fn required_metadata(name: &str) -> String {
    let value =
        std::env::var(name).unwrap_or_else(|_| panic!("{name} must be explicit for measurements"));
    assert!(
        !value.trim().is_empty(),
        "{name} must not be blank for measurements"
    );
    value
}

fn duration_summary(samples: &[Duration]) -> Value {
    assert!(!samples.is_empty());
    let mut nanos = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    nanos.sort_unstable();
    json!({
        "count": nanos.len(),
        "p50Ns": percentile(&nanos, 50),
        "p95Ns": percentile(&nanos, 95),
        "p99Ns": percentile(&nanos, 99),
        "maxNs": nanos.last().copied().unwrap(),
    })
}

fn percentile(sorted: &[u128], value: usize) -> u128 {
    let rank = (value * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
