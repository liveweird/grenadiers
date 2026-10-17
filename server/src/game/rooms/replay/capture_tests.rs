use grenadiers_core::{
    game::{Direction, SeatId},
    protocol::{MAX_COUNTER, RoomCode, RoomId},
    room::{DepartureReason, InterruptReason, MatchPhase},
};

use super::super::{
    match_runtime::{MatchAction, MatchRuntime, MatchRuntimeError},
    profile::RoomProfile,
    types::RoomIdentity,
};
use super::{Operation, ReplayError, ReplayStatus, record::MAX_RECORDS, replay, runner::identity};
use crate::round_seed::RoundSeedSource;

fn generated(source: RoundSeedSource) -> MatchRuntime {
    let mut runtime = MatchRuntime::new(identity(), source);
    runtime.initial_notice().unwrap();
    runtime
}

fn occupy(runtime: &mut MatchRuntime) {
    runtime.occupy(SeatId::One).unwrap();
    runtime.occupy(SeatId::Two).unwrap();
}

fn start(runtime: &mut MatchRuntime) {
    occupy(runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    while runtime.lifecycle().phase() != MatchPhase::Running {
        runtime.tick().unwrap();
    }
}

#[test]
fn capture_keeps_accepted_noops_and_excludes_every_rejected_command() {
    let mut runtime = generated(RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(3);
        Ok(())
    }));
    occupy(&mut runtime);
    let initial = runtime.replay_journal().len();
    assert_eq!(
        runtime.action(SeatId::One, 0, MatchAction::Ready(true)),
        Err(MatchRuntimeError::StaleEpoch)
    );
    assert_eq!(
        runtime.action(SeatId::One, 1, MatchAction::Movement(None)),
        Err(MatchRuntimeError::InvalidPhase)
    );
    assert_eq!(runtime.replay_journal().len(), initial);

    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    assert_eq!(runtime.replay_journal().len(), initial + 2);
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    while runtime.lifecycle().phase() != MatchPhase::Running {
        runtime.tick().unwrap();
    }
    let epoch = runtime.lifecycle().round_attempt();
    let before_controls = runtime.replay_journal().len();
    for _ in 0..8 {
        runtime
            .action(
                SeatId::One,
                epoch,
                MatchAction::Movement(Some(Direction::East)),
            )
            .unwrap();
    }
    assert_eq!(
        runtime.action(
            SeatId::One,
            epoch,
            MatchAction::Movement(Some(Direction::East))
        ),
        Err(MatchRuntimeError::RateLimited)
    );
    assert_eq!(runtime.replay_journal().len(), before_controls + 8);
    runtime.tick().unwrap();
    runtime
        .action(SeatId::One, epoch, MatchAction::Bomb(1))
        .unwrap();
    let after_bomb = runtime.replay_journal().len();
    assert_eq!(
        runtime.action(SeatId::One, epoch, MatchAction::Bomb(1)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    assert_eq!(
        runtime.action(SeatId::One, epoch, MatchAction::Bomb(MAX_COUNTER + 1)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    assert_eq!(runtime.replay_journal().len(), after_bomb);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn atomic_interrupt_departure_replays_and_construction_failure_is_incomplete() {
    let mut atomic = generated(RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(4);
        Ok(())
    }));
    start(&mut atomic);
    atomic.stop_seat_inputs(SeatId::One);
    atomic
        .interrupt_and_depart(
            InterruptReason::ServerStopped,
            [Some(DepartureReason::Explicit), None],
        )
        .unwrap();
    assert!(matches!(
        atomic.replay_journal().records.last().unwrap().operation,
        Operation::InterruptAndDepart { .. }
    ));
    assert_eq!(
        replay(atomic.replay_journal()).unwrap(),
        atomic.replay_inspection()
    );

    let mut failed = generated(RoundSeedSource::with_entropy(|_| Err(())));
    start_until_terminal(&mut failed);
    assert!(matches!(
        failed.replay_journal().status(),
        ReplayStatus::Incomplete { .. }
    ));
    assert_eq!(
        replay(failed.replay_journal()),
        Err(ReplayError::Incomplete)
    );
}

fn start_until_terminal(runtime: &mut MatchRuntime) {
    occupy(runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    while matches!(runtime.lifecycle().phase(), MatchPhase::Countdown { .. }) {
        runtime.tick().unwrap();
    }
}

#[test]
fn truncation_is_bounded_and_cannot_change_runtime_or_fixed_test_profile() {
    let mut runtime = generated(RoundSeedSource::with_entropy(|_| panic!("no round start")));
    runtime.interrupt(InterruptReason::ServerStopped).unwrap();
    while runtime.replay_journal().len() < MAX_RECORDS {
        runtime.tick().unwrap();
    }
    let before = runtime.replay_inspection();
    let capacity = runtime.replay_journal().records.capacity();
    runtime.tick().unwrap();
    runtime.tick().unwrap();
    assert_eq!(runtime.replay_inspection(), before);
    assert_eq!(runtime.replay_journal().len(), MAX_RECORDS);
    assert_eq!(runtime.replay_journal().records.capacity(), capacity);
    assert!(runtime.replay_journal().allocated_bytes() <= 4 * 1024 * 1024);
    assert!(matches!(
        runtime.replay_journal().status(),
        ReplayStatus::Truncated { .. }
    ));
    assert_eq!(
        replay(runtime.replay_journal()),
        Err(ReplayError::Truncated)
    );

    let mut fixed = MatchRuntime::new(
        RoomIdentity {
            id: RoomId::new("00000000000000000000000000000002").unwrap(),
            code: RoomCode::new("BCDEFGHJ").unwrap(),
            profile: RoomProfile::FixedClassicV1,
        },
        RoundSeedSource::with_entropy(|_| panic!("fixed profile entropy")),
    );
    fixed.occupy(SeatId::One).unwrap();
    assert_eq!(
        fixed.replay_journal().status(),
        ReplayStatus::UnsupportedProfile
    );
    assert_eq!(fixed.replay_journal().len(), 0);
}
