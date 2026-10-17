use grenadiers_core::{
    game::{SeatId, SpeedLevel},
    room::{DepartureReason, MatchPhase, RoomSettings},
};

use super::{
    record::{Header, Journal, M5_LIFECYCLE_PROFILE, Operation, ReplayAction},
    replay,
    runner::{ReplayError, identity_for_profile},
};
use crate::{
    game::rooms::{
        match_runtime::{MatchAction, MatchRuntime},
        profile::RoomProfile,
    },
    round_seed::RoundSeedSource,
};

#[test]
fn configurable_settings_actions_and_authority_replay_exactly() {
    let mut runtime = MatchRuntime::new(
        identity_for_profile(RoomProfile::GeneratedConfigurableV4),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(53);
            Ok(())
        }),
    );
    runtime.initial_notice().unwrap();
    runtime.occupy(SeatId::One).unwrap();
    runtime.occupy(SeatId::Two).unwrap();
    let settings = RoomSettings::new(SpeedLevel::Three, 64, 2, 90).unwrap();
    runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .unwrap();
    runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 2,
                settings,
            },
        )
        .unwrap();
    runtime
        .action(
            SeatId::Two,
            1,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 2,
            },
        )
        .unwrap();
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
    while runtime.lifecycle().phase() != MatchPhase::Running {
        runtime.tick().unwrap();
    }

    assert_eq!(runtime.replay_journal().header.rules_version, 4);
    assert_eq!(
        runtime.replay_journal().header.initial_settings,
        RoomSettings::DEFAULT
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn permanent_editor_release_is_part_of_the_replayed_state() {
    let mut runtime = MatchRuntime::new(
        identity_for_profile(RoomProfile::GeneratedConfigurableV4),
        RoundSeedSource::with_entropy(|_| panic!("no round starts")),
    );
    runtime.initial_notice().unwrap();
    runtime.occupy(SeatId::One).unwrap();
    runtime
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap();

    assert_eq!(runtime.replay_inspection().settings_editor, None);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn legacy_rules_reject_revision_checked_ready_actions() {
    let mut journal = Journal::new(Header::m6([false; 2]));
    journal.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::ReadyChecked {
                ready: true,
                settings_revision: 1,
            },
        },
    );

    assert_eq!(replay(&journal), Err(ReplayError::InvalidActionProfile));
}

#[test]
fn legacy_rules_reject_settings_updates() {
    let mut journal = Journal::new(Header::m6_v3([false; 2]));
    journal.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT,
            },
        },
    );

    assert_eq!(replay(&journal), Err(ReplayError::InvalidActionProfile));
}

#[test]
fn configurable_rules_reject_unfenced_legacy_ready_actions() {
    let mut journal = Journal::new(Header::configurable_v4());
    for operation in [
        Operation::Occupy(SeatId::One),
        Operation::Occupy(SeatId::Two),
    ] {
        journal.record(grenadiers_core::game::Tick(0), operation);
    }
    journal.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::new(SpeedLevel::Three, 64, 2, 90).unwrap(),
            },
        },
    );
    journal.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::Ready(true),
        },
    );

    assert_eq!(replay(&journal), Err(ReplayError::InvalidActionProfile));
}

#[test]
fn configurable_rules_reject_the_legacy_lifecycle_profile() {
    let mut header = Header::configurable_v4();
    header.lifecycle_profile = M5_LIFECYCLE_PROFILE;

    assert_eq!(
        replay(&Journal::new(header)),
        Err(ReplayError::UnsupportedLifecycle)
    );
}
