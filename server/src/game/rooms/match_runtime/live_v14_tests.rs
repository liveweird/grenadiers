use std::sync::Arc;

use grenadiers_core::{
    game::SeatId,
    room::{CpuProfile, MatchPhase, RoomMode, RoomSettings},
};

use super::{MatchAction, MatchNotice, MatchRuntime};
use crate::{
    game::rooms::{match_runtime::test_support::identity, profile::RoomProfile},
    round_seed::RoundSeedSource,
};

fn live(profile: RoomProfile) -> MatchRuntime {
    solo(profile, true)
}

fn solo(profile: RoomProfile, live14: bool) -> MatchRuntime {
    let mut identity = identity(profile);
    identity.mode = RoomMode::Solo(CpuProfile::CURRENT);
    identity.private_cpu_seed = Some([17; 32]);
    let seeds = RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(23);
        Ok(())
    });
    if live14 {
        MatchRuntime::new_live14(identity, seeds)
    } else {
        MatchRuntime::new(identity, seeds)
    }
}

fn json(notice: &MatchNotice, seat: SeatId) -> serde_json::Value {
    serde_json::from_str(&notice.frame(seat).expect("live v14 notice frame")).unwrap()
}

fn admit(runtime: &mut MatchRuntime) -> MatchNotice {
    let prepared = runtime.prepare_solo_creator(SeatId::One).unwrap();
    let notice = prepared.notice.clone();
    runtime.commit_admission(prepared);
    notice
}

fn start(runtime: &mut MatchRuntime) -> MatchNotice {
    start_with_settings_revision(runtime, 1)
}

fn start_with_settings_revision(runtime: &mut MatchRuntime, settings_revision: u32) -> MatchNotice {
    runtime
        .action(
            SeatId::One,
            runtime.lifecycle().round_attempt(),
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision,
            },
        )
        .unwrap();
    loop {
        if let Some(notice) = runtime.tick().unwrap()
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            return notice;
        }
    }
}

fn bomb_cpu_and_wait_for_result(runtime: &mut MatchRuntime) -> MatchNotice {
    runtime
        .action(
            SeatId::Two,
            runtime.lifecycle().round_attempt(),
            MatchAction::Bomb(1),
        )
        .unwrap();
    loop {
        if let Some(notice) = runtime.tick().unwrap()
            && matches!(
                runtime.lifecycle().phase(),
                MatchPhase::Result { .. } | MatchPhase::MatchResult { .. }
            )
        {
            return notice;
        }
    }
}

#[test]
fn initial_and_admission_notices_are_v14_room_states_with_real_cpu_ready() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    let initial = runtime.initial_notice().unwrap();
    let one = initial.frame(SeatId::One).unwrap();
    let two = initial.frame(SeatId::Two).unwrap();
    assert!(Arc::ptr_eq(&one, &two));
    let value: serde_json::Value = serde_json::from_str(&one).unwrap();
    assert_eq!(value["type"], "roomState");
    assert_eq!(value["room"]["settingsEditor"], serde_json::Value::Null);
    assert_eq!(value["room"]["seats"][1]["ready"], true);
    assert!(runtime.lifecycle().seats()[SeatId::Two.index()].ready());

    let prepared = runtime.prepare_solo_creator(SeatId::One).unwrap();
    let frame = prepared
        .encode(
            SeatId::One,
            grenadiers_core::protocol::Counter::new(1).unwrap(),
            grenadiers_core::protocol::v13::ResumeToken::new("a".repeat(64)).unwrap(),
        )
        .unwrap();
    let admitted: serde_json::Value = serde_json::from_str(&frame).unwrap();
    assert_eq!(admitted["type"], "admitted");
    assert_eq!(admitted["room"]["seats"][1]["ready"], true);
}

#[test]
fn failed_prepared_admission_does_not_commit_candidate_or_cpu_state() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    runtime.initial_notice().unwrap();
    let before = runtime.replay_inspection();
    let revision = runtime.publication_revision_for_test();
    let prepared = runtime.prepare_solo_creator(SeatId::One).unwrap();

    assert!(
        prepared
            .encode(
                SeatId::Two,
                grenadiers_core::protocol::Counter::new(1).unwrap(),
                grenadiers_core::protocol::v13::ResumeToken::new("b".repeat(64)).unwrap(),
            )
            .is_err()
    );
    drop(prepared);
    assert_eq!(runtime.replay_inspection(), before);
    assert_eq!(runtime.publication_revision_for_test(), revision);
}

#[test]
fn settings_and_human_ready_batch_with_cpu_consensus() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    runtime.initial_notice().unwrap();
    admit(&mut runtime);
    let settings =
        RoomSettings::DEFAULT.with_enabled_items(grenadiers_core::game::EnabledItems::ALL);
    let changed = runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::new(
                    settings.starting_speed(),
                    192,
                    settings.wins_to_finish(),
                    settings.bomb_fuse_ticks(),
                )
                .unwrap()
                .with_enabled_items(settings.enabled_items()),
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        json(&changed, SeatId::One)["room"]["seats"][1]["ready"],
        true
    );
    assert!(runtime.lifecycle().seats()[SeatId::Two.index()].ready());

    let countdown = runtime
        .action(
            SeatId::One,
            1,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 2,
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        json(&countdown, SeatId::One)["room"]["phase"]["type"],
        "countdown"
    );
}

#[test]
fn live_snapshot_keeps_legacy_cpu_observation_without_v13_frames() {
    let mut live_runtime = live(RoomProfile::ExpandedV5);
    live_runtime.initial_notice().unwrap();
    admit(&mut live_runtime);
    let live_notice = start(&mut live_runtime);
    let live_value = json(&live_notice, SeatId::One);
    assert_eq!(live_value["type"], "snapshot");
    assert_eq!(live_value["room"]["configuration"]["controllers"][1], "cpu");
    assert!(live_notice.cpu_observation.is_some());

    let encoded = live_notice.frame(SeatId::One).unwrap();
    assert!(!encoded.contains("\"mode\""));
    assert!(encoded.contains("\"configuration\""));
}

#[test]
fn live_and_legacy_snapshots_keep_identical_cpu16_observation() {
    let mut legacy = solo(RoomProfile::ExpandedV5, false);
    legacy.initial_notice().unwrap();
    admit(&mut legacy);
    let legacy_notice = start(&mut legacy);

    let mut live = live(RoomProfile::ExpandedV5);
    live.initial_notice().unwrap();
    admit(&mut live);
    let live_notice = start(&mut live);

    assert_eq!(live_notice.cpu_observation, legacy_notice.cpu_observation);
}

#[test]
fn invalid_human_action_does_not_mutate_live_cpu_or_revision() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    runtime.initial_notice().unwrap();
    admit(&mut runtime);
    let before = runtime.replay_inspection();
    let revision = runtime.publication_revision_for_test();
    assert!(
        runtime
            .action(
                SeatId::One,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 99,
                },
            )
            .is_err()
    );
    assert_eq!(runtime.replay_inspection(), before);
    assert_eq!(runtime.publication_revision_for_test(), revision);
}

#[test]
fn failed_live_snapshot_restores_tick_reports_and_revision() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    runtime.initial_notice().unwrap();
    admit(&mut runtime);
    start(&mut runtime);
    assert!(runtime.tick().unwrap().is_none());
    assert!(runtime.tick().unwrap().is_none());
    runtime.replace_generation_for_test(None);
    let before = runtime.replay_inspection();
    let revision = runtime.publication_revision_for_test();

    assert_eq!(runtime.tick(), Err(super::MatchRuntimeError::Publication));
    assert_eq!(runtime.replay_inspection(), before);
    assert_eq!(runtime.publication_revision_for_test(), revision);
}

#[test]
fn inter_round_projects_logical_cpu_ready_without_changing_real_readiness() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    runtime.initial_notice().unwrap();
    admit(&mut runtime);
    start(&mut runtime);
    bomb_cpu_and_wait_for_result(&mut runtime);
    let notice = loop {
        if let Some(notice) = runtime.tick().unwrap()
            && matches!(
                runtime.lifecycle().phase(),
                MatchPhase::Countdown {
                    kind: grenadiers_core::room::CountdownKind::InterRound,
                    ..
                }
            )
        {
            break notice;
        }
    };
    assert!(!runtime.lifecycle().seats()[SeatId::Two.index()].ready());
    let value = json(&notice, SeatId::One);
    assert_eq!(value["room"]["phase"]["kind"], "interRound");
    assert_eq!(value["room"]["seats"][1]["ready"], true);
}

#[test]
fn eligible_cpu_vote_precedes_human_vote_and_starts_one_clean_epoch() {
    let mut runtime = live(RoomProfile::ExpandedV5);
    runtime.initial_notice().unwrap();
    admit(&mut runtime);
    let settings = RoomSettings::new(grenadiers_core::game::SpeedLevel::Base, 256, 1, 180)
        .unwrap()
        .with_enabled_items(grenadiers_core::game::EnabledItems::ALL);
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
    start_with_settings_revision(&mut runtime, 2);
    bomb_cpu_and_wait_for_result(&mut runtime);
    while !matches!(runtime.lifecycle().phase(), MatchPhase::MatchResult { .. }) {
        runtime.tick().unwrap();
    }
    let epoch = runtime.lifecycle().round_attempt();
    assert!(!runtime.lifecycle().series().rematch_votes()[SeatId::Two.index()]);
    let eligible = loop {
        if let Some(notice) = runtime.tick().unwrap()
            && runtime.lifecycle().series().rematch_votes()[SeatId::Two.index()]
        {
            break notice;
        }
    };
    assert_eq!(
        json(&eligible, SeatId::One)["room"]["seats"][1]["rematchReady"],
        true
    );

    let waiting = runtime
        .action(SeatId::One, epoch, MatchAction::Rematch(true))
        .unwrap()
        .unwrap();
    assert_eq!(runtime.lifecycle().round_attempt(), epoch + 1);
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Waiting);
    assert!(runtime.sim().is_none());
    assert!(runtime.lifecycle().seats()[SeatId::Two.index()].ready());
    let value = json(&waiting, SeatId::One);
    assert_eq!(value["type"], "roomState");
    assert_eq!(value["room"]["phase"]["type"], "waiting");
}
