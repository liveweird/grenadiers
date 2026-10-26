use std::{
    collections::VecDeque,
    mem::size_of,
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use grenadiers_core::{
    game::{Direction, RoundAssignment, SeatId, Tick},
    protocol::MAX_COUNTER,
    room::{
        CountdownKind, DepartureReason, InterruptReason, MatchPhase, reconnect::ReservationState,
    },
};

use super::super::{
    match_runtime::{MatchRuntime, RuntimeInspection},
    profile::RoomProfile,
};
use super::{
    Header, Journal, Operation, ReplayAction, ReplayError, ReplayStatus,
    record::{MAX_RECORDS, Record},
    replay,
    runner::{apply, identity_for_profile},
};
use crate::round_seed::RoundSeedSource;

struct RecordingRuntime {
    runtime: MatchRuntime,
}

impl RecordingRuntime {
    fn new(seeds: impl IntoIterator<Item = [u8; 32]>) -> Self {
        Self::new_with_profile(RoomProfile::GeneratedClassicV2, seeds)
    }

    fn new_with_profile(profile: RoomProfile, seeds: impl IntoIterator<Item = [u8; 32]>) -> Self {
        let seeds = Arc::new(Mutex::new(seeds.into_iter().collect::<VecDeque<_>>()));
        let source = Arc::clone(&seeds);
        let mut runtime = MatchRuntime::new_recording(
            identity_for_profile(profile),
            RoundSeedSource::with_entropy(move |bytes| {
                let seed = source.lock().map_err(|_| ())?.pop_front().ok_or(())?;
                bytes.copy_from_slice(&seed);
                Ok(())
            }),
        );
        runtime.initial_notice().unwrap();
        Self { runtime }
    }

    fn append(&mut self, operation: Operation) {
        apply(&mut self.runtime, operation).unwrap();
    }

    fn action(&mut self, seat: SeatId, action: ReplayAction) {
        self.append(Operation::Action {
            seat,
            epoch: self.runtime.lifecycle().round_attempt(),
            action,
        });
    }

    fn tick_until(&mut self, predicate: impl Fn(MatchPhase) -> bool) {
        while !predicate(self.runtime.lifecycle().phase()) {
            self.append(Operation::Tick);
        }
    }

    fn start(&mut self) {
        self.action(SeatId::One, ReplayAction::Ready(true));
        self.action(SeatId::Two, ReplayAction::Ready(true));
        self.tick_until(|phase| phase == MatchPhase::Running);
    }

    fn finish(&mut self, bombing: &[SeatId]) {
        for &seat in bombing {
            self.action(seat, ReplayAction::Bomb(1));
        }
        self.tick_until(|phase| phase != MatchPhase::Running);
    }

    fn checkpoint(&self) -> RuntimeInspection {
        self.runtime.replay_inspection()
    }
}

fn occupied(seeds: impl IntoIterator<Item = [u8; 32]>) -> RecordingRuntime {
    let mut recording = RecordingRuntime::new(seeds);
    recording.append(Operation::Occupy(SeatId::One));
    recording.append(Operation::Occupy(SeatId::Two));
    recording
}

fn occupied_with_profile(profile: RoomProfile, seed: [u8; 32]) -> RecordingRuntime {
    let mut recording = RecordingRuntime::new_with_profile(profile, [seed]);
    recording.append(Operation::Occupy(SeatId::One));
    recording.append(Operation::Occupy(SeatId::Two));
    recording
}

#[test]
fn generated_match_replays_cancellation_wins_rematch_draw_and_interruption() {
    let mut recording = occupied((1..=5).map(|value| [value; 32]));
    recording.action(SeatId::One, ReplayAction::Ready(true));
    recording.action(SeatId::Two, ReplayAction::Ready(true));
    for _ in 0..12 {
        recording.append(Operation::Tick);
    }
    recording.action(SeatId::One, ReplayAction::Ready(false));
    recording.start();

    for round in 1..=3 {
        recording.finish(&[SeatId::Two]);
        if round < 3 {
            recording.tick_until(|phase| {
                matches!(
                    phase,
                    MatchPhase::Countdown {
                        kind: CountdownKind::InterRound,
                        ..
                    }
                )
            });
            recording.tick_until(|phase| phase == MatchPhase::Running);
        }
    }
    loop {
        if matches!(recording.runtime.lifecycle().phase(), MatchPhase::MatchResult { rematch_at, .. }
            if recording.runtime.lifecycle().tick() >= rematch_at)
        {
            break;
        }
        recording.append(Operation::Tick);
    }
    recording.action(SeatId::One, ReplayAction::Rematch(true));
    recording.action(SeatId::Two, ReplayAction::Rematch(true));
    recording.start();
    recording.finish(&[SeatId::One, SeatId::Two]);
    recording.append(Operation::Interrupt(InterruptReason::ServerStopped));

    let replayed = replay(recording.runtime.replay_journal()).unwrap();
    assert_eq!(replayed, recording.checkpoint());
    assert!(recording.runtime.replay_journal().len() < MAX_RECORDS);
}

#[test]
fn archived_profile_one_replays_a_complete_physics_result_fixture() {
    let mut recording = occupied([[6; 32]]);
    recording.start();
    recording.action(SeatId::One, ReplayAction::Movement(Some(Direction::East)));
    for _ in 0..12 {
        recording.append(Operation::Tick);
    }
    recording.action(SeatId::One, ReplayAction::Movement(None));
    recording.finish(&[SeatId::One, SeatId::Two]);
    recording.append(Operation::Interrupt(InterruptReason::ServerStopped));

    let mut archived = Journal::new(Header::m5([false; 2]));
    for record in &recording.runtime.replay_journal().records {
        if !matches!(record.operation, Operation::ReportFlush | Operation::Expiry) {
            archived.record(record.tick, record.operation);
        }
    }
    assert!(archived.records.iter().all(|record| !matches!(
        record.operation,
        Operation::GraceDepart(_)
            | Operation::Resume { .. }
            | Operation::Expiry
            | Operation::ReportFlush
    )));
    let replayed = replay(&archived).unwrap();
    let expected = recording.checkpoint();
    assert_eq!(replayed.lifecycle.tick(), expected.lifecycle.tick());
    assert_eq!(
        replayed.lifecycle.round_attempt(),
        expected.lifecycle.round_attempt()
    );
    assert_eq!(replayed.lifecycle.phase(), expected.lifecycle.phase());
    assert_eq!(replayed.lifecycle.seats(), expected.lifecycle.seats());
    assert_eq!(replayed.lifecycle.series(), expected.lifecycle.series());
    assert_eq!(replayed.simulation, expected.simulation);
    assert_eq!(replayed.inputs, expected.inputs);
    assert_eq!(replayed.reports, expected.reports);
    assert_eq!(replayed.lifecycle.reconnect_grace().latest_tick(), Tick(0));
    assert_eq!(
        replayed.lifecycle.reconnect_grace().seats(),
        &[ReservationState::Connected; 2]
    );
}

#[test]
fn replay_selects_the_recorded_turning_rules_for_v2_and_v3() {
    let mut legacy = occupied_with_profile(RoomProfile::GeneratedClassicV2, [31; 32]);
    let mut current = occupied_with_profile(RoomProfile::GeneratedClassicV3, [31; 32]);
    legacy.start();
    current.start();
    let start = legacy.runtime.sim().unwrap().state.actors()[0].position();

    for recording in [&mut legacy, &mut current] {
        recording.action(SeatId::One, ReplayAction::Movement(Some(Direction::East)));
        for _ in 0..4 {
            recording.append(Operation::Tick);
        }
        recording.action(SeatId::One, ReplayAction::Movement(Some(Direction::South)));
        for _ in 0..8 {
            recording.append(Operation::Tick);
        }
    }

    let legacy_position = legacy.runtime.sim().unwrap().state.actors()[0].position();
    let current_position = current.runtime.sim().unwrap().state.actors()[0].position();
    assert_eq!(legacy.runtime.replay_journal().header.rules_version, 2);
    assert_eq!(current.runtime.replay_journal().header.rules_version, 3);
    assert_eq!(legacy_position.y, start.y);
    assert!(current_position.y > start.y);
    assert_ne!(current_position, legacy_position);
    assert_eq!(
        replay(legacy.runtime.replay_journal()).unwrap(),
        legacy.checkpoint()
    );
    assert_eq!(
        replay(current.runtime.replay_journal()).unwrap(),
        current.checkpoint()
    );
}

#[test]
fn movement_refresh_and_predeparture_fence_preserve_input_semantics() {
    let mut recording = occupied([[7; 32]]);
    recording.start();
    recording.action(SeatId::One, ReplayAction::Movement(Some(Direction::East)));
    for _ in 0..10 {
        recording.append(Operation::Tick);
    }
    recording.action(SeatId::One, ReplayAction::Movement(Some(Direction::East)));
    for _ in 0..10 {
        recording.append(Operation::Tick);
    }
    let refreshed = recording.checkpoint();
    assert!(refreshed.inputs.contains("direction: Some(East)"));
    assert!(!refreshed.reports.is_empty());
    assert_eq!(
        replay(recording.runtime.replay_journal()).unwrap(),
        refreshed
    );
    recording.action(SeatId::One, ReplayAction::Bomb(1));
    recording.append(Operation::InputFence(SeatId::One));
    recording.append(Operation::Tick);
    recording.append(Operation::Depart([
        Some(DepartureReason::TransportLoss),
        None,
    ]));

    let expected = recording.checkpoint();
    assert!(
        expected
            .simulation
            .as_ref()
            .unwrap()
            .state
            .world()
            .bombs()
            .is_empty()
    );
    assert_eq!(
        replay(recording.runtime.replay_journal()).unwrap(),
        expected
    );
}

#[test]
fn malformed_versions_order_bounds_and_round_metadata_fail_closed() {
    let mut journal = Journal::new(Header::m5([false; 2]));
    journal.header.format_version += 1;
    assert_eq!(replay(&journal), Err(ReplayError::UnsupportedRules));
    journal.header = Header::m5([false; 2]);
    journal.header.lifecycle_profile += 2;
    assert_eq!(replay(&journal), Err(ReplayError::UnsupportedLifecycle));
    journal.header = Header::m5([false; 2]);
    journal.header.rules_version += 3;
    assert_eq!(replay(&journal), Err(ReplayError::UnsupportedRules));
    journal.header = Header::m5([false; 2]);
    journal.header.generator_version += 1;
    assert_eq!(replay(&journal), Err(ReplayError::UnsupportedGenerator));
    journal.header = Header::m5([false; 2]);
    journal.header.initial_tick = Tick(1);
    assert_eq!(replay(&journal), Err(ReplayError::UnsupportedInitialTick));

    let mut bounds = Journal::new(Header::m5([true, false]));
    bounds.record(
        Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 0,
            action: ReplayAction::Ready(true),
        },
    );
    assert_eq!(replay(&bounds), Err(ReplayError::InvalidBounds));
    bounds.records[0].order = 3;
    assert_eq!(replay(&bounds), Err(ReplayError::InvalidOrder));

    let mut recording = occupied([[9; 32]]);
    recording.start();
    let marker = recording
        .runtime
        .replay_journal_mut()
        .records
        .iter_mut()
        .find(|record| matches!(record.operation, Operation::RoundStarted { .. }))
        .unwrap();
    if let Operation::RoundStarted { assignment, .. } = &mut marker.operation {
        *assignment = RoundAssignment::new(NonZeroU32::new(2).unwrap());
    }
    assert_eq!(
        replay(recording.runtime.replay_journal()),
        Err(ReplayError::RoundStartMismatch)
    );

    let mut action_bound = occupied([[10; 32]]);
    action_bound.start();
    action_bound.action(SeatId::One, ReplayAction::Bomb(MAX_COUNTER));
    assert!(replay(action_bound.runtime.replay_journal()).is_ok());
    if let Operation::Action { action, .. } = &mut action_bound
        .runtime
        .replay_journal_mut()
        .records
        .last_mut()
        .unwrap()
        .operation
    {
        *action = ReplayAction::Bomb(MAX_COUNTER + 1);
    }
    assert_eq!(
        replay(action_bound.runtime.replay_journal()),
        Err(ReplayError::InvalidBounds)
    );
    if let Operation::Action { action, .. } = &mut action_bound
        .runtime
        .replay_journal_mut()
        .records
        .last_mut()
        .unwrap()
        .operation
    {
        *action = ReplayAction::Bomb(u32::MAX);
    }
    assert_eq!(
        replay(action_bound.runtime.replay_journal()),
        Err(ReplayError::InvalidBounds)
    );
}

#[test]
fn missing_extra_seed_and_truncation_are_never_complete_replays() {
    let mut missing = occupied([[1; 32]]);
    missing.start();
    missing
        .runtime
        .replay_journal_mut()
        .records
        .retain(|record| !matches!(record.operation, Operation::RoundStarted { .. }));
    let mut previous_tick = None;
    let mut order = 0;
    for record in &mut missing.runtime.replay_journal_mut().records {
        if previous_tick != Some(record.tick) {
            order = 0;
        }
        record.order = order;
        order += 1;
        previous_tick = Some(record.tick);
    }
    assert_eq!(
        replay(missing.runtime.replay_journal()),
        Err(ReplayError::MissingSeed)
    );

    let mut extra = Journal::new(Header::m5([false; 2]));
    let generated = grenadiers_core::game::generate_arena([2; 32]).record();
    extra.record(
        Tick(0),
        Operation::RoundStarted {
            epoch: 1,
            assignment: RoundAssignment::first(),
            generation: generated,
        },
    );
    assert_eq!(replay(&extra), Err(ReplayError::ExtraSeed));

    let mut truncated = Journal::new(Header::m5([false; 2]));
    for _ in 0..MAX_RECORDS {
        assert!(truncated.record(Tick(0), Operation::Tick));
    }
    assert!(!truncated.record(Tick(0), Operation::Tick));
    assert!(matches!(truncated.status(), ReplayStatus::Truncated { .. }));
    assert_eq!(replay(&truncated), Err(ReplayError::Truncated));
    assert_eq!(size_of::<Record>(), 64);
    assert!(MAX_RECORDS * size_of::<Record>() < 4 * 1024 * 1024);
    assert!(!format!("{truncated:?}").contains("seed"));
}
