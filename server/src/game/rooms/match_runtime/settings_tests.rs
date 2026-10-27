use grenadiers_core::{
    game::{SeatId, SpeedLevel},
    protocol::{Seat, v13::RulesVersion},
    room::{DepartureReason, MatchPhase, RoomSettings},
};

use super::test_support::*;
use super::{MatchAction, MatchRuntimeError};
use crate::round_seed::RoundSeedSource;

fn settings(speed: SpeedLevel, turn: i32, wins: u8, fuse: u64) -> RoomSettings {
    RoomSettings::new(speed, turn, wins, fuse).unwrap()
}

fn runtime() -> super::MatchRuntime {
    configurable_runtime(RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(41);
        Ok(())
    }))
}

#[test]
fn creator_updates_are_revision_fenced_atomic_and_clear_both_readiness_flags() {
    let mut runtime = runtime();
    let initial = runtime.initial_notice().unwrap();
    assert_eq!(initial.room.rules_version, RulesVersion::ConfigurableV4);
    assert_eq!(initial.room.settings_revision.get(), 1);
    assert_eq!(initial.room.settings_editor, None);

    let creator = runtime.occupy(SeatId::One).unwrap();
    runtime.occupy(SeatId::Two).unwrap();
    assert_eq!(creator.room.settings_editor, Some(Seat::One));
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();

    let replacement = settings(SpeedLevel::Four, 64, 1, 60);
    let changed = runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: replacement,
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(changed.room.settings_revision.get(), 2);
    assert_eq!(
        grenadiers_core::protocol::v13::expanded_settings(changed.room.settings).unwrap(),
        replacement
    );
    assert!(changed.room.seats.iter().all(|seat| !seat.ready));

    let unchanged_revision = changed.room.revision;
    let unchanged = runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 2,
                settings: replacement,
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.room.revision, unchanged_revision);
    assert_eq!(unchanged.room.settings_revision.get(), 2);

    assert_eq!(
        runtime.action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT,
            },
        ),
        Err(MatchRuntimeError::StaleSettings)
    );
    assert_eq!(
        runtime.action(
            SeatId::Two,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 2,
                settings: RoomSettings::DEFAULT,
            },
        ),
        Err(MatchRuntimeError::SettingsLocked)
    );
    runtime
        .action(
            SeatId::One,
            1,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 2,
            },
        )
        .unwrap();
    assert_eq!(
        runtime.action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 2,
                settings: RoomSettings::DEFAULT,
            },
        ),
        Err(MatchRuntimeError::SettingsLocked)
    );
    assert_eq!(runtime.lifecycle().settings(), replacement);
}

#[test]
fn exhausted_settings_revision_rejects_before_mutating_room_configuration() {
    let mut runtime = runtime();
    runtime.occupy(SeatId::One).unwrap();
    runtime.set_settings_revision(grenadiers_core::protocol::MAX_COUNTER);
    assert_eq!(
        runtime.action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: grenadiers_core::protocol::MAX_COUNTER,
                settings: settings(SpeedLevel::One, 192, 2, 150),
            },
        ),
        Err(MatchRuntimeError::CapacityReached)
    );
    assert_eq!(runtime.lifecycle().settings(), RoomSettings::DEFAULT);
}

#[test]
fn ready_fence_and_configured_round_use_the_authoritative_settings() {
    let mut runtime = runtime();
    occupy_both(&mut runtime);
    let configured = settings(SpeedLevel::Four, 64, 1, 60);
    runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: configured,
            },
        )
        .unwrap();
    assert_eq!(
        runtime.action(
            SeatId::One,
            1,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 1,
            },
        ),
        Err(MatchRuntimeError::StaleSettings)
    );
    for seat in SeatId::ALL {
        runtime
            .action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    assert!(matches!(
        runtime.lifecycle().phase(),
        MatchPhase::Countdown { .. }
    ));
    assert_eq!(
        runtime.action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 2,
                settings: RoomSettings::DEFAULT,
            },
        ),
        Err(MatchRuntimeError::SettingsLocked)
    );
    tick_until_running(&mut runtime);
    let state = &runtime.sim().unwrap().state;
    assert_eq!(state.rules().room_settings(), Some(configured));
    assert!(
        state
            .actors()
            .iter()
            .all(|actor| actor.speed() == SpeedLevel::Four)
    );
    assert_eq!(runtime.lifecycle().series().wins_to_finish(), 1);
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(1))
        .unwrap();
    runtime.tick().unwrap();
    let bomb = state_bomb(&runtime);
    assert_eq!(
        bomb.detonates_at().unwrap().0 - runtime.lifecycle().tick().0,
        60
    );
}

#[test]
fn rematch_retains_settings_and_reopens_editing_for_the_original_creator() {
    let mut runtime = runtime();
    occupy_both(&mut runtime);
    let configured = settings(SpeedLevel::One, 192, 1, 60);
    runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: configured,
            },
        )
        .unwrap();
    for seat in SeatId::ALL {
        runtime
            .action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    tick_until_running(&mut runtime);
    finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    for _ in 0..180 {
        runtime.tick().unwrap();
    }
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(SeatId::One, epoch, MatchAction::Rematch(true))
        .unwrap();
    runtime
        .action(SeatId::Two, epoch, MatchAction::Rematch(true))
        .unwrap();
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Waiting);
    assert_eq!(runtime.lifecycle().settings(), configured);
    runtime
        .action(
            SeatId::One,
            epoch + 1,
            MatchAction::UpdateSettings {
                settings_revision: 2,
                settings: RoomSettings::DEFAULT,
            },
        )
        .unwrap();
    assert_eq!(runtime.lifecycle().settings(), RoomSettings::DEFAULT);
}

fn state_bomb(runtime: &super::MatchRuntime) -> grenadiers_core::game::Bomb {
    *runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap()
}

#[test]
fn creator_authority_survives_transport_but_never_transfers_after_release() {
    let mut retained = runtime();
    start_first_round(&mut retained);
    retained
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    assert_eq!(
        retained.current_notice().unwrap().room.settings_editor,
        Some(Seat::One)
    );
    let prepared = retained.prepare_resume(SeatId::One).unwrap();
    retained.commit_admission(prepared);

    let mut expired = runtime();
    start_first_round(&mut expired);
    expired
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    for _ in 0..grenadiers_core::room::reconnect::RECONNECT_GRACE_TICKS {
        expired.settle_boundary([None; 2]).unwrap();
    }
    assert_eq!(expired.current_notice().unwrap().room.settings_editor, None);

    let mut released = runtime();
    released.occupy(SeatId::One).unwrap();
    released
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap();
    assert_eq!(
        released.current_notice().unwrap().room.settings_editor,
        None
    );
    released.occupy(SeatId::One).unwrap();
    assert_eq!(
        released.current_notice().unwrap().room.settings_editor,
        None
    );

    let mut prematch_loss = runtime();
    prematch_loss.occupy(SeatId::One).unwrap();
    prematch_loss
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    prematch_loss.occupy(SeatId::One).unwrap();
    assert_eq!(
        prematch_loss.current_notice().unwrap().room.settings_editor,
        None
    );
}
