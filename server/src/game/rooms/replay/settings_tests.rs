use grenadiers_core::{
    game::{EnabledItems, SeatId, SpeedLevel},
    room::{DepartureReason, MatchPhase, RoomSettings},
};

use super::{
    item_mask_plan::{ItemMaskPlan, PARTITION_CAPACITY},
    record::{Header, Journal, M5_LIFECYCLE_PROFILE, Operation, ReplayAction},
    replay,
    runner::{ReplayError, identity_for_profile},
};
use crate::{
    game::rooms::{
        match_runtime::{
            MatchAction, MatchRuntime, absorb_generated_heart, collect_generated_heart,
            expanded_heart_runtime, start_expanded_heart_round, wait_clear,
        },
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

#[test]
fn expanded_item_masks_and_disabled_preset_replay_deterministically() {
    let catalog = [
        16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144,
    ];
    assert_eq!(catalog.len(), 15);
    let plan = ItemMaskPlan::checked(&catalog, EnabledItems::ALL.bits(), PARTITION_CAPACITY)
        .unwrap_or_else(|error| panic!("invalid exhaustive item-mask plan: {error}"));
    let selection = plan
        .selection_from_environment()
        .unwrap_or_else(|error| panic!("invalid item-mask partition selection: {error}"));
    let context = ItemMaskPlan::context_from_environment()
        .unwrap_or_else(|error| panic!("invalid item-mask gate context: {error}"));
    let mut observed = 0;
    for ordinal in selection.range.clone() {
        let enabled_bits = plan.enabled_bits(ordinal);
        let enabled = EnabledItems::from_bits(enabled_bits).unwrap();
        let mut runtime = MatchRuntime::new(
            identity_for_profile(RoomProfile::ExpandedV5),
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(3);
                Ok(())
            }),
        );
        runtime.initial_notice().unwrap();
        runtime.occupy(SeatId::One).unwrap();
        runtime.occupy(SeatId::Two).unwrap();
        if enabled != EnabledItems::HEART {
            runtime
                .action(
                    SeatId::One,
                    1,
                    MatchAction::UpdateSettings {
                        settings_revision: 1,
                        settings: RoomSettings::DEFAULT.with_enabled_items(enabled),
                    },
                )
                .unwrap();
        }
        for seat in SeatId::ALL {
            runtime
                .action(
                    seat,
                    1,
                    MatchAction::ReadyChecked {
                        ready: true,
                        settings_revision: if enabled != EnabledItems::HEART { 2 } else { 1 },
                    },
                )
                .unwrap();
        }
        while runtime.lifecycle().phase() != MatchPhase::Running {
            runtime.tick().unwrap();
        }
        let journal = runtime.replay_journal();
        assert_eq!(journal.header.format_version, 2);
        assert_eq!(journal.header.rules_version, 5);
        assert_eq!(journal.header.generator_version, 2);
        assert_eq!(runtime.lifecycle().settings().enabled_items(), enabled);
        assert_eq!(replay(journal).unwrap(), runtime.replay_inspection());
        observed += 1;
    }
    plan.emit_completion(&selection, &context, observed);
}

#[test]
fn generated_heart_collection_absorption_and_later_elimination_replay_exactly() {
    let mut runtime = expanded_heart_runtime();
    start_expanded_heart_round(&mut runtime);
    collect_generated_heart(&mut runtime);
    absorb_generated_heart(&mut runtime);
    wait_clear(&mut runtime);

    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(3))
        .unwrap();
    while runtime.sim().unwrap().state.actors()[0].alive() {
        runtime.tick().unwrap();
    }

    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn replay_rejects_crossed_legacy_and_expanded_identity_tuples() {
    let mut expanded_as_legacy = Header::expanded_v5();
    expanded_as_legacy.format_version = 1;
    assert_eq!(
        replay(&Journal::new(expanded_as_legacy)),
        Err(ReplayError::UnsupportedRules)
    );

    let mut legacy_with_items = Header::configurable_v4();
    legacy_with_items.initial_settings =
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART);
    assert_eq!(
        replay(&Journal::new(legacy_with_items)),
        Err(ReplayError::UnsupportedRules)
    );

    let mut legacy_update_with_items = Journal::new(Header::configurable_v4());
    legacy_update_with_items.record(
        grenadiers_core::game::Tick(0),
        Operation::Action {
            seat: SeatId::One,
            epoch: 1,
            action: ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART),
            },
        },
    );
    assert_eq!(
        replay(&legacy_update_with_items),
        Err(ReplayError::InvalidActionProfile)
    );
}
