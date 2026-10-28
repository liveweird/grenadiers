use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::{
    game::SeatId,
    room::{
        CpuDifficulty, DepartureReason, InterruptReason, MatchPhase, RoomConfiguration,
        RoomSettings, RosterParticipant, SeatControl,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{
    CpuProvenance, Operation, RecordingRosterRuntime, ReplayAction, ReplayError, ReplayStatus,
    replay, verify_exact,
};

#[test]
fn reserved_expiry_completes_rematch_consensus_without_a_round_marker_or_panic() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Standard),
        grenadiers_core::room::BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut recorder = RecordingRosterRuntime::new(
        configuration,
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(41);
            Ok(())
        }),
        vec![CpuProvenance::new(
            SeatId::Two,
            CpuDifficulty::Standard,
            [19; 32],
        )],
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Three] {
        recorder.admit(seat, false).unwrap();
        recorder
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    recorder
        .settle([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap();
    recorder.publish_flush().unwrap();
    recorder.interrupt(InterruptReason::ServerStopped).unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    recorder
        .human_action(SeatId::One, 1, ReplayAction::Rematch(true))
        .unwrap();
    for _ in 0..grenadiers_core::room::reconnect::RECONNECT_GRACE_TICKS {
        recorder.settle([None; 4]).unwrap();
    }

    let inspection = recorder.runtime().replay_inspection();
    assert_eq!(inspection.lifecycle.phase(), MatchPhase::Waiting);
    assert_eq!(
        inspection.lifecycle.participant(SeatId::Three).unwrap(),
        RosterParticipant::Departed
    );
    assert!(inspection.simulation.is_none());
    let (_, journal) = recorder.into_parts();
    assert_eq!(
        journal
            .records
            .iter()
            .filter(|record| matches!(record.operation, Operation::RoundStarted { .. }))
            .count(),
        1
    );
    verify_exact(&journal, &inspection).unwrap();
}

#[test]
fn failed_round_construction_marks_incomplete_once_and_replay_rejects_it() {
    let entropy_calls = Arc::new(AtomicUsize::new(0));
    let source_calls = Arc::clone(&entropy_calls);
    let mut recorder = RecordingRosterRuntime::new(
        RoomConfiguration::human_duel(),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(move |_| {
            source_calls.fetch_add(1, Ordering::SeqCst);
            Err(())
        }),
        vec![],
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Two] {
        recorder.admit(seat, false).unwrap();
        recorder
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS - 1 {
        recorder.settle([None; 4]).unwrap();
    }
    assert_eq!(recorder.settle([None; 4]), Err(ReplayError::Runtime));
    assert_eq!(entropy_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        recorder.journal().status(),
        ReplayStatus::IncompleteRoundConstruction
    );
    assert_eq!(replay(recorder.journal()), Err(ReplayError::Incomplete));
    assert!(matches!(
        recorder.runtime().replay_inspection().lifecycle.phase(),
        MatchPhase::Countdown { .. }
    ));
}
