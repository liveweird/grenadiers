use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, RoomCode, RoomId, v14},
    room::{BoardSize, DepartureReason, MatchPhase, RoomConfiguration, RoomSettings, SeatControl},
};

use crate::round_seed::RoundSeedSource;

use super::{RosterAction, RosterIdentity, RosterRuntime, RosterRuntimeError};

pub(super) fn identity(configuration: RoomConfiguration) -> RosterIdentity {
    RosterIdentity {
        id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        registry_serial: 7,
        configuration,
    }
}

pub(super) fn configuration(count: usize) -> RoomConfiguration {
    configuration_for(count, BoardSize::FifteenByThirteen)
}

pub(super) fn configuration_for(count: usize, board: BoardSize) -> RoomConfiguration {
    RoomConfiguration::new(&vec![SeatControl::Human; count], None, board).unwrap()
}

pub(super) fn runtime(count: usize) -> RosterRuntime {
    runtime_for(count, BoardSize::FifteenByThirteen)
}

pub(super) fn runtime_for(count: usize, board: BoardSize) -> RosterRuntime {
    RosterRuntime::new(
        identity(configuration_for(count, board)),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(3);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

pub(super) fn admit_all(runtime: &mut RosterRuntime, count: usize) {
    for &seat in &SeatId::ALL[..count] {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
}

pub(super) fn start(runtime: &mut RosterRuntime, count: usize) {
    admit_all(runtime, count);
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Running);
}

#[test]
fn two_three_and_four_seat_rooms_start_with_checked_recipient_frames() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in 2..=4 {
            let mut runtime = runtime_for(count, board);
            start(&mut runtime, count);
            let sim = runtime.sim.as_ref().unwrap();
            assert_eq!(sim.state.world().arena().board(), board);
            assert_eq!(
                sim.generation.version(),
                if board == BoardSize::FifteenByThirteen {
                    grenadiers_core::game::MULTIPLAYER_GENERATOR_VERSION
                } else {
                    grenadiers_core::game::VARIABLE_GENERATOR_VERSION
                }
            );
            assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
            assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
            let notice = runtime.settle_boundary([None; 4]).unwrap().unwrap();
            assert_eq!(
                notice.legacy_room().unwrap().configuration.domain().board(),
                board
            );
            for &seat in &SeatId::ALL[..count] {
                assert!(notice.frame(seat).is_some());
            }
            assert!(notice.frame(SeatId::ALL[count.min(3)]).is_none() || count == 4);
        }
    }
}

#[test]
fn fourth_seat_admission_is_personal_and_commit_is_explicit() {
    let mut runtime = runtime(4);
    for &seat in &SeatId::ALL[..3] {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
    let prepared = runtime.prepare_join(SeatId::Four).unwrap();
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Four).unwrap(),
        grenadiers_core::room::RosterParticipant::NeverAdmitted
    );
    let encoded = prepared
        .encode(
            Counter::new(1).unwrap(),
            v14::ResumeToken::new(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
        )
        .unwrap();
    assert!(encoded.contains("\"youSeat\":\"four\""));
    runtime.commit_admission(prepared);
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Four).unwrap(),
        grenadiers_core::room::RosterParticipant::Connected
    );
}

#[test]
fn departure_elimination_uses_the_physics_tick_without_advancing_twice() {
    let mut runtime = runtime(4);
    start(&mut runtime, 4);
    let before = runtime.lifecycle.tick();
    let notice = runtime
        .settle_boundary([
            None,
            None,
            Some(DepartureReason::Explicit),
            Some(DepartureReason::Explicit),
        ])
        .unwrap()
        .unwrap();
    assert_eq!(runtime.lifecycle.tick().0, before.0 + 1);
    let sim = runtime.sim.as_ref().unwrap();
    assert!(!sim.state.actors()[2].alive());
    assert!(!sim.state.actors()[3].alive());
    assert!(notice.frame(SeatId::One).is_some());
}

#[test]
fn creator_authority_does_not_transfer_after_departure() {
    let mut runtime = runtime(3);
    admit_all(&mut runtime, 3);
    runtime
        .settle_boundary([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();
    assert_eq!(runtime.lifecycle.settings_editor(), None);
    assert_eq!(
        runtime.action(
            SeatId::Two,
            1,
            RosterAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT,
            },
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
}

#[test]
fn checked_boards_are_accepted_and_cpu_human_admission_is_rejected() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        assert!(
            RosterRuntime::new(
                identity(configuration_for(3, board)),
                RoomSettings::DEFAULT,
                RoundSeedSource::with_entropy(|_| Ok(())),
                Arc::new(|_| Ok(SimulationSeed::new([0; 32]))),
            )
            .is_ok()
        );
    }
    let cpu = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(grenadiers_core::room::CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let runtime = RosterRuntime::new(
        identity(cpu),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|_| Ok(())),
        Arc::new(|_| Ok(SimulationSeed::new([0; 32]))),
    )
    .unwrap();
    assert_eq!(
        runtime.prepare_join(SeatId::Two).err(),
        Some(RosterRuntimeError::InvalidPhase)
    );
}

#[test]
fn failed_admission_encoding_and_failed_publication_leave_runtime_unchanged() {
    let mut runtime = runtime(4);
    let mut prepared = runtime.prepare_join(SeatId::Four).unwrap();
    prepared.legacy_room_mut().unwrap().configuration =
        v14::RoomConfiguration::from_domain(configuration(3));
    assert_eq!(
        prepared
            .encode(
                Counter::new(1).unwrap(),
                v14::ResumeToken::new(
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                )
                .unwrap(),
            )
            .err(),
        Some(RosterRuntimeError::Publication)
    );
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Four).unwrap(),
        grenadiers_core::room::RosterParticipant::NeverAdmitted
    );

    admit_all(&mut runtime, 4);
    runtime.revision = grenadiers_core::protocol::MAX_COUNTER;
    assert_eq!(
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .err(),
        Some(RosterRuntimeError::CapacityReached)
    );
    assert!(!runtime.lifecycle.ready(SeatId::One).unwrap());
    assert_eq!(runtime.revision, grenadiers_core::protocol::MAX_COUNTER);
}

#[test]
fn terminal_departure_freezes_physics_and_preserves_the_final_result() {
    let mut runtime = runtime(3);
    start(&mut runtime, 3);
    runtime
        .settle_boundary([
            None,
            Some(DepartureReason::Explicit),
            Some(DepartureReason::Explicit),
            None,
        ])
        .unwrap();
    assert!(matches!(
        runtime.lifecycle.phase(),
        MatchPhase::MatchResult { .. }
    ));
    let physical_tick = runtime.sim.as_ref().unwrap().state.tick();
    let result = runtime.sim.as_ref().unwrap().state.result();
    for _ in 0..3 {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.sim.as_ref().unwrap().state.tick(), physical_tick);
    assert_eq!(runtime.sim.as_ref().unwrap().state.result(), result);
}

#[test]
fn running_actions_and_cpu_publication_use_the_current_round_epoch() {
    let mut runtime = runtime(4);
    start(&mut runtime, 4);
    runtime
        .action(
            SeatId::Three,
            1,
            RosterAction::Movement(Some(grenadiers_core::game::Direction::South)),
        )
        .unwrap();
    runtime
        .action(SeatId::Four, 1, RosterAction::Bomb(1))
        .unwrap();
    assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
    assert!(runtime.settle_boundary([None; 4]).unwrap().is_none());
    let notice = runtime.settle_boundary([None; 4]).unwrap().unwrap();
    assert!(notice.frame(SeatId::Three).is_some());
    assert!(runtime.prepare_cpu_publication().unwrap().is_some());
    assert_eq!(
        runtime
            .action(SeatId::One, 2, RosterAction::Movement(None))
            .err(),
        Some(RosterRuntimeError::StaleEpoch)
    );
}

#[test]
fn environment_runtime_publishes_checked_v15_state_for_disabled_and_single() {
    for environment in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
        let mut runtime = RosterRuntime::new_environment(
            identity(configuration(2)),
            RoomSettings::DEFAULT,
            environment,
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(17);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        )
        .unwrap();
        start(&mut runtime, 2);
        let notice = runtime.current_notice().unwrap();
        let room = notice.environment_room().unwrap();
        assert_eq!(
            room.rules_version,
            grenadiers_core::protocol::v15::RulesVersion::ENVIRONMENT_V7
        );
        assert_eq!(
            room.settings.environment.storm_cloud,
            environment == StormCloudConfig::Single
        );
        let publication = notice.environment_cpu_publication().unwrap();
        let projected = grenadiers_core::protocol::v15::project_recipient(
            publication,
            grenadiers_core::protocol::v15::Seat::One,
        )
        .unwrap();
        assert_eq!(
            projected.state.storm_cloud.is_some(),
            environment == StormCloudConfig::Single
        );
    }
}

#[test]
fn explicit_departure_and_grace_expiry_are_eliminated_as_one_batch() {
    let mut runtime = runtime(4);
    start(&mut runtime, 4);
    runtime
        .settle_boundary([None, None, Some(DepartureReason::TransportLoss), None])
        .unwrap();
    for _ in 1..grenadiers_core::room::reconnect::RECONNECT_GRACE_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let before = runtime.lifecycle.tick();
    runtime
        .settle_boundary([None, None, None, Some(DepartureReason::Explicit)])
        .unwrap();
    assert_eq!(runtime.lifecycle.tick().0, before.0 + 1);
    let actors = runtime.sim.as_ref().unwrap().state.actors();
    assert!(!actors[2].alive());
    assert!(!actors[3].alive());
    assert!(actors[0].alive());
    assert!(actors[1].alive());
    assert_eq!(runtime.sim.as_ref().unwrap().state.result(), None);
}
