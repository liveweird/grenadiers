use std::sync::Arc;

use grenadiers_core::{
    game::{RoundResult, SeatId, SimulationSeed, SpeedLevel, StormCloudConfig},
    room::{
        DepartureReason, InterruptReason, LairSettings, LairTuning, MatchPhase, RoomSettings,
        RosterParticipant, series::MatchResult,
    },
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError, tests};
use crate::round_seed::RoundSeedSource;

fn runtime() -> RosterRuntime {
    runtime_with_settings(RoomSettings::DEFAULT)
}

fn runtime_with_settings(settings: RoomSettings) -> RosterRuntime {
    RosterRuntime::new_lairs(
        tests::identity(tests::configuration(3)),
        settings,
        StormCloudConfig::Single,
        LairTuning::new(true, LairSettings::new(7, 3).unwrap()),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(31);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

fn score_runtime() -> RosterRuntime {
    runtime_with_settings(RoomSettings::new(SpeedLevel::Base, 256, 1, 180).unwrap())
}

#[test]
fn score_terminal_boundary_retires_transport_loss_without_replacing_the_winner() {
    let mut runtime = score_runtime();
    tests::start(&mut runtime, 3);

    let notice = runtime
        .settle_boundary([
            Some(DepartureReason::TransportLoss),
            Some(DepartureReason::Explicit),
            Some(DepartureReason::Explicit),
            None,
        ])
        .unwrap()
        .unwrap();

    let outcome = runtime.lifecycle.outcome().unwrap();
    assert_eq!(outcome.result(), MatchResult::ScoreWinner(SeatId::One));
    assert_eq!(
        runtime.sim.as_ref().unwrap().state.result(),
        Some(RoundResult::Winner(SeatId::One))
    );
    assert!(notice.lair_cpu_publication().is_some());
    assert_eq!(
        runtime.lifecycle.participant(SeatId::One),
        Ok(RosterParticipant::Departed)
    );
    assert_eq!(
        runtime.prepare_resume(SeatId::One).err(),
        Some(RosterRuntimeError::StaleEpoch)
    );
}

#[test]
fn delayed_score_terminal_loss_keeps_the_winner_and_frozen_world_publishable() {
    let mut runtime = score_runtime();
    tests::start(&mut runtime, 3);
    let final_notice = runtime
        .settle_boundary([
            None,
            Some(DepartureReason::Explicit),
            Some(DepartureReason::Explicit),
            None,
        ])
        .unwrap()
        .unwrap();
    let outcome = runtime.lifecycle.outcome().unwrap();
    assert_eq!(outcome.result(), MatchResult::ScoreWinner(SeatId::One));
    assert_eq!(
        runtime.sim.as_ref().unwrap().state.result(),
        Some(RoundResult::Winner(SeatId::One))
    );
    assert!(final_notice.lair_cpu_publication().is_some());
    let physical_tick = runtime.sim.as_ref().unwrap().state.tick();
    for _ in 0..3 {
        runtime.settle_boundary([None; 4]).unwrap();
    }

    let disconnected = runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap()
        .unwrap();

    assert!(disconnected.lair_cpu_publication().is_some());
    assert_eq!(runtime.lifecycle.outcome(), Some(outcome));
    assert_eq!(runtime.sim.as_ref().unwrap().state.tick(), physical_tick);
    assert_eq!(
        runtime.sim.as_ref().unwrap().state.result(),
        Some(RoundResult::Winner(SeatId::One))
    );
    assert_eq!(
        runtime.lifecycle.participant(SeatId::One),
        Ok(RosterParticipant::Departed)
    );
    assert_eq!(
        runtime.prepare_resume(SeatId::One).err(),
        Some(RosterRuntimeError::StaleEpoch)
    );
}

#[test]
fn delayed_terminal_transport_loss_publishes_v16_and_retires_the_seat() {
    let mut runtime = runtime();
    tests::start(&mut runtime, 3);
    runtime.interrupt(InterruptReason::ServerStopped).unwrap();
    let outcome = runtime.lifecycle.outcome().unwrap();
    runtime.settle_boundary([None; 4]).unwrap();

    let notice = runtime
        .settle_boundary([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap()
        .unwrap();

    assert!(matches!(
        notice.lair_room().unwrap().phase,
        grenadiers_core::protocol::v16::RoomPhase::MatchResult { .. }
    ));
    assert_eq!(runtime.lifecycle.outcome(), Some(outcome));
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Three),
        Ok(RosterParticipant::Departed)
    );
    assert_eq!(
        runtime.prepare_resume(SeatId::Three).err(),
        Some(RosterRuntimeError::StaleEpoch)
    );

    while let MatchPhase::MatchResult { rematch_at, .. } = runtime.lifecycle.phase() {
        if runtime.lifecycle.tick() >= rematch_at {
            break;
        }
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let first_vote = runtime
        .action(SeatId::One, 1, RosterAction::Rematch(true))
        .unwrap()
        .unwrap();
    assert!(matches!(
        first_vote.lair_room().unwrap().phase,
        grenadiers_core::protocol::v16::RoomPhase::MatchResult { .. }
    ));
    let rematch = runtime
        .action(SeatId::Two, 1, RosterAction::Rematch(true))
        .unwrap()
        .unwrap();
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    assert!(matches!(
        rematch.lair_room().unwrap().phase,
        grenadiers_core::protocol::v16::RoomPhase::Waiting
    ));
}

#[test]
fn preterminal_reservation_can_resume_after_final_outcome() {
    let mut runtime = runtime();
    tests::start(&mut runtime, 3);
    runtime
        .settle_boundary([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let RosterParticipant::Reserved { expires_at } =
        runtime.lifecycle.participant(SeatId::Three).unwrap()
    else {
        panic!("preterminal transport loss must reserve the seat");
    };
    runtime.interrupt(InterruptReason::ServerStopped).unwrap();
    let outcome = runtime.lifecycle.outcome().unwrap();

    let prepared = runtime.prepare_resume(SeatId::Three).unwrap();
    assert_eq!(
        prepared.lair_room().unwrap().tick.get(),
        runtime.lifecycle.tick().0
    );
    runtime.commit_admission(prepared);

    assert_eq!(runtime.lifecycle.outcome(), Some(outcome));
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Three),
        Ok(RosterParticipant::Connected)
    );
    assert!(runtime.lifecycle.tick() < expires_at);
    assert_eq!(
        runtime
            .action(SeatId::Three, 1, RosterAction::Rematch(true))
            .unwrap_err(),
        RosterRuntimeError::InvalidPhase
    );
}
