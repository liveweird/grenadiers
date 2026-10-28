use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::{
    game::{Direction, SeatId, SimulationSeed},
    room::{
        BoardSize, CpuDifficulty, DepartureReason, InterruptReason, MatchPhase, RoomConfiguration,
        RoomSettings, SeatControl,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{RosterAction, RosterRuntime, RosterRuntimeError, tests};

fn counted_runtime(count: usize, entropy_calls: Arc<AtomicUsize>) -> RosterRuntime {
    RosterRuntime::new(
        tests::identity(tests::configuration(count)),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(move |bytes| {
            entropy_calls.fetch_add(1, Ordering::SeqCst);
            bytes.fill(9);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

fn ready_until_last_countdown_tick(runtime: &mut RosterRuntime, count: usize) {
    tests::admit_all(runtime, count);
    for &seat in &SeatId::ALL[..count] {
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 1..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
}

#[test]
fn countdown_boundary_applies_departures_before_round_construction() {
    let canceled_entropy = Arc::new(AtomicUsize::new(0));
    let mut canceled = counted_runtime(3, Arc::clone(&canceled_entropy));
    ready_until_last_countdown_tick(&mut canceled, 3);
    let notice = canceled
        .settle_boundary([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap()
        .unwrap();
    assert!(matches!(
        notice.room.phase,
        grenadiers_core::protocol::v14::RoomPhase::Waiting
    ));
    assert_eq!(canceled_entropy.load(Ordering::SeqCst), 0);
    assert!(canceled.sim.is_none());

    let started_entropy = Arc::new(AtomicUsize::new(0));
    let mut started = counted_runtime(3, Arc::clone(&started_entropy));
    ready_until_last_countdown_tick(&mut started, 3);
    let notice = started
        .settle_boundary([None, None, Some(DepartureReason::Explicit), None])
        .unwrap()
        .unwrap();
    assert!(matches!(
        notice.room.phase,
        grenadiers_core::protocol::v14::RoomPhase::Running
    ));
    assert_eq!(started_entropy.load(Ordering::SeqCst), 1);
    assert!(!started.sim.as_ref().unwrap().state.actors()[2].alive());
}

#[test]
fn running_snapshots_and_cpu_publications_share_three_report_cadence() {
    let mut runtime = tests::runtime(4);
    tests::start(&mut runtime, 4);
    assert!(runtime.prepare_cpu_publication().unwrap().is_some());
    assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
    assert!(runtime.prepare_cpu_publication().unwrap().is_none());
    assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
    assert!(runtime.prepare_cpu_publication().unwrap().is_none());
    let notice = runtime.settle_boundary([None; 4]).unwrap().unwrap();
    assert!(notice.cpu_publication.is_some());
    assert_eq!(
        runtime.prepare_cpu_publication().unwrap(),
        notice.cpu_publication
    );
}

#[test]
fn controller_roles_are_checked_on_both_action_paths() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new(
        tests::identity(configuration),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(2);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([5; 32]))),
    )
    .unwrap();
    let creator = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(creator);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(
        runtime.action(
            SeatId::Two,
            1,
            RosterAction::Movement(Some(Direction::East))
        ),
        Err(RosterRuntimeError::InvalidPhase)
    );
    assert_eq!(
        runtime.submit_cpu_intent(SeatId::One, 1, RosterAction::Movement(None)),
        Err(RosterRuntimeError::InvalidPhase)
    );
    runtime
        .submit_cpu_intent(SeatId::Two, 1, RosterAction::Bomb(1))
        .unwrap();
    assert_eq!(runtime.next_cpu_manual_action(SeatId::Two), Some(2));
    assert_eq!(runtime.next_cpu_manual_action(SeatId::One), None);
}

#[test]
fn expiry_consensus_resets_stale_round_state_on_rematch() {
    let mut runtime = tests::runtime(3);
    tests::start(&mut runtime, 3);
    runtime
        .settle_boundary([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap();
    runtime.interrupt(InterruptReason::ServerStopped).unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    for seat in [SeatId::One, SeatId::Two] {
        runtime
            .action(seat, 1, RosterAction::Rematch(true))
            .unwrap();
    }
    while runtime.lifecycle.tick().0 < 780 {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let notice = runtime.settle_boundary([None; 4]).unwrap().unwrap();
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    assert!(matches!(
        notice.room.phase,
        grenadiers_core::protocol::v14::RoomPhase::Waiting
    ));
    assert!(runtime.sim.is_none());
    assert!(runtime.reports.is_empty());
    assert!(runtime.prepare_cpu_publication().unwrap().is_none());
}

#[test]
fn prematch_reservation_is_truthful_resumable_and_does_not_fill_vacancies() {
    let mut runtime = tests::runtime(3);
    let settings = RoomSettings::new(grenadiers_core::game::SpeedLevel::One, 192, 4, 150).unwrap();
    let creator = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(creator);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .unwrap();
    let notice = runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap()
        .unwrap();
    let creator = &notice.room.seats[0];
    assert!(creator.occupied);
    assert!(matches!(
        creator.connection,
        grenadiers_core::protocol::v14::ConnectionStatus::Disconnected { .. }
    ));
    assert_eq!(
        notice.room.settings_editor,
        Some(grenadiers_core::protocol::v14::Seat::One)
    );
    assert_eq!(
        runtime.prepare_join(SeatId::One).err(),
        Some(RosterRuntimeError::InvalidPhase)
    );

    let vacant = runtime.prepare_join(SeatId::Two).unwrap();
    runtime.commit_admission(vacant);
    let resumed = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(resumed.room.settings.wins_to_finish(), 4);
    assert_eq!(resumed.room.round_id.get(), 1);
    runtime.commit_admission(resumed);
    assert_eq!(runtime.lifecycle.settings(), settings);
    assert_eq!(runtime.lifecycle.round_attempt(), 1);
}

#[test]
fn running_resume_commits_the_exact_prepared_cpu_publication() {
    let mut runtime = tests::runtime(3);
    tests::start(&mut runtime, 3);
    assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
    let disconnected = runtime
        .settle_boundary([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap()
        .unwrap();
    let disconnected_publication = disconnected.cpu_publication.unwrap();
    let prepared = runtime.prepare_resume(SeatId::Three).unwrap();
    let prepared_publication = prepared.notice.cpu_publication.clone().unwrap();
    assert_ne!(prepared_publication, disconnected_publication);
    runtime.commit_admission(prepared);
    assert_eq!(
        runtime.prepare_cpu_publication().unwrap(),
        Some(prepared_publication)
    );
}

#[test]
fn closing_final_match_cancels_votes_before_departure_can_start_rematch() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new(
        tests::identity(configuration),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(4);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([6; 32]))),
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Two] {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    runtime.interrupt(InterruptReason::ServerStopped).unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    runtime
        .action(SeatId::One, 1, RosterAction::Rematch(true))
        .unwrap();
    let outcome = runtime.lifecycle.outcome();
    let scores = runtime.lifecycle.scores().to_vec();
    let epoch = runtime.lifecycle.round_attempt();
    let physical_tick = runtime.sim.as_ref().unwrap().state.tick();

    let notice = runtime
        .interrupt_and_depart(
            InterruptReason::ServerStopped,
            [None, Some(DepartureReason::Explicit), None, None],
        )
        .unwrap();
    assert!(matches!(
        runtime.lifecycle.phase(),
        MatchPhase::MatchResult { .. }
    ));
    assert_eq!(runtime.lifecycle.outcome(), outcome);
    assert_eq!(runtime.lifecycle.scores(), scores);
    assert_eq!(runtime.lifecycle.round_attempt(), epoch);
    assert_eq!(runtime.sim.as_ref().unwrap().state.tick(), physical_tick);
    assert!(notice.frames.is_some());
    assert_eq!(runtime.retained_human_count(), 1);
}
