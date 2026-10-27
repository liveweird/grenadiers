use grenadiers_core::{
    cpu::{Intent, ManualIntent, MovementIntent},
    game::{Direction, SeatId},
    room::MatchPhase,
};

use super::super::{
    match_runtime::{MatchAction, MatchRuntime, MatchRuntimeError},
    profile::RoomProfile,
};
use super::{Operation, ReplayAction, replay, runner::identity_for_profile};
use crate::round_seed::RoundSeedSource;

fn recording_runtime() -> MatchRuntime {
    let mut runtime = MatchRuntime::new_recording(
        identity_for_profile(RoomProfile::ExpandedV5),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(91);
            Ok(())
        }),
    );
    runtime.initial_notice().unwrap();
    runtime.occupy(SeatId::One).unwrap();
    runtime.occupy(SeatId::Two).unwrap();
    runtime
}

fn solo_recording_runtime(profile: grenadiers_core::room::CpuProfile) -> MatchRuntime {
    let mut identity = identity_for_profile(RoomProfile::ExpandedV5);
    identity.mode = grenadiers_core::room::RoomMode::Solo(profile);
    identity.private_cpu_seed = Some([73; 32]);
    let mut runtime = MatchRuntime::new_recording(
        identity,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(91);
            Ok(())
        }),
    );
    runtime.initial_notice().unwrap();
    let creator = runtime.prepare_solo_creator(SeatId::One).unwrap();
    runtime.commit_admission(creator);
    runtime
}

fn start(runtime: &mut MatchRuntime) {
    let revision = runtime.replay_inspection().settings_revision;
    for seat in SeatId::ALL {
        runtime
            .action(
                seat,
                runtime.lifecycle().round_attempt(),
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: revision,
                },
            )
            .unwrap();
    }
    while runtime.lifecycle().phase() != MatchPhase::Running {
        runtime.tick().unwrap();
    }
}

/// Tiny deterministic fixture only: this is not a playable CPU policy.
fn fixture_policy(step: usize) -> Intent {
    match step {
        0 => Intent::moving(Direction::West).with_manual(ManualIntent::Bomb),
        1 => Intent::moving(Direction::North),
        2 => Intent::STOP.with_manual(ManualIntent::Equipment),
        _ => Intent::STOP,
    }
}

fn submit_intent(
    runtime: &mut MatchRuntime,
    seat: SeatId,
    action_id: u32,
    intent: Intent,
) -> Result<usize, MatchRuntimeError> {
    let epoch = runtime.lifecycle().round_attempt();
    let movement = match intent.movement {
        MovementIntent::Direction(direction) => Some(direction),
        MovementIntent::Stop => None,
    };
    runtime.action(seat, epoch, MatchAction::Movement(movement))?;
    let mut accepted = 1;
    if let Some(manual) = intent.manual {
        let action = match manual {
            ManualIntent::Bomb => MatchAction::Bomb(action_id),
            ManualIntent::Equipment => MatchAction::Equipment(action_id),
        };
        runtime.action(seat, epoch, action)?;
        accepted += 1;
    }
    Ok(accepted)
}

fn cpu_action_records(runtime: &MatchRuntime) -> usize {
    runtime
        .replay_journal()
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.operation,
                Operation::Action {
                    seat: SeatId::Two,
                    action: ReplayAction::Movement(_)
                        | ReplayAction::Bomb(_)
                        | ReplayAction::Equipment(_),
                    ..
                }
            )
        })
        .count()
}

#[test]
fn accepted_fixture_intents_replay_exactly_without_rerunning_policy() {
    let mut runtime = recording_runtime();
    start(&mut runtime);
    let mut policy_calls = 0;
    let mut accepted = 0;
    for step in 0..4 {
        let intent = fixture_policy(step);
        policy_calls += 1;
        accepted += submit_intent(&mut runtime, SeatId::Two, step as u32 + 1, intent).unwrap();
        runtime.tick().unwrap();
    }
    assert_eq!(cpu_action_records(&runtime), accepted);
    let expected = runtime.replay_inspection();

    let replayed = replay(runtime.replay_journal()).unwrap();

    assert_eq!(
        policy_calls, 4,
        "replay must consume records, not the policy"
    );
    assert_eq!(replayed, expected);
}

#[test]
fn accepted_solo_inputs_replay_from_format_three_without_worker_or_policy() {
    let mut runtime = solo_recording_runtime(grenadiers_core::room::CpuProfile::CURRENT);
    start(&mut runtime);
    for step in 0..4 {
        submit_intent(
            &mut runtime,
            SeatId::Two,
            step as u32 + 1,
            fixture_policy(step),
        )
        .unwrap();
        runtime.tick().unwrap();
    }
    let journal = runtime.replay_journal();
    assert_eq!(journal.header.format_version, 3);
    assert_eq!(
        journal.header.mode,
        grenadiers_core::room::RoomMode::Solo(grenadiers_core::room::CpuProfile::CURRENT)
    );
    assert_eq!(journal.header.solo.unwrap().seat, SeatId::Two);
    assert_eq!(journal.header.solo.unwrap().policy_version, 9);
    let expected = runtime.replay_inspection();
    assert_eq!(replay(journal).unwrap(), expected);
}

#[test]
fn rejected_cpu_like_actions_never_create_accepted_records() {
    let mut runtime = recording_runtime();
    let epoch = runtime.lifecycle().round_attempt();

    let before = runtime.replay_journal().len();
    assert_eq!(
        runtime.action(SeatId::Two, epoch, MatchAction::Movement(None)),
        Err(MatchRuntimeError::InvalidPhase)
    );
    assert_eq!(runtime.replay_journal().len(), before);

    assert_eq!(
        runtime.action(
            SeatId::Two,
            0,
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 1,
            },
        ),
        Err(MatchRuntimeError::StaleEpoch)
    );
    assert_eq!(runtime.replay_journal().len(), before);

    start(&mut runtime);
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(SeatId::Two, epoch, MatchAction::Bomb(1))
        .unwrap();
    let after_bomb = runtime.replay_journal().len();
    assert_eq!(
        runtime.action(SeatId::Two, epoch, MatchAction::Bomb(1)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    assert_eq!(runtime.replay_journal().len(), after_bomb);

    runtime.tick().unwrap();
    let before_burst = runtime.replay_journal().len();
    for _ in 0..8 {
        runtime
            .action(
                SeatId::Two,
                epoch,
                MatchAction::Movement(Some(Direction::West)),
            )
            .unwrap();
    }
    assert_eq!(runtime.replay_journal().len(), before_burst + 8);
    let before_rejected = runtime.replay_journal().len();
    assert_eq!(
        runtime.action(
            SeatId::Two,
            epoch,
            MatchAction::Movement(Some(Direction::West)),
        ),
        Err(MatchRuntimeError::RateLimited)
    );
    assert_eq!(runtime.replay_journal().len(), before_rejected);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn original_cpu_journals_replay_without_policy_reexecution() {
    for (profile, version) in [
        (grenadiers_core::room::CpuProfile::INITIAL, 1),
        (grenadiers_core::room::CpuProfile::PULSE_V2, 2),
        (grenadiers_core::room::CpuProfile::ROUTE_V3, 3),
        (grenadiers_core::room::CpuProfile::PLANT_V4, 4),
        (grenadiers_core::room::CpuProfile::GOALS_V5, 5),
        (grenadiers_core::room::CpuProfile::WEAPONS_V6, 6),
        (grenadiers_core::room::CpuProfile::WALL_PASS_V7, 7),
        (grenadiers_core::room::CpuProfile::RESPONSIVE_V8, 8),
    ] {
        let mut runtime = solo_recording_runtime(profile);
        start(&mut runtime);
        submit_intent(&mut runtime, SeatId::Two, 1, fixture_policy(0)).unwrap();
        runtime.tick().unwrap();
        assert_eq!(
            runtime.replay_journal().header.solo.unwrap().policy_version,
            version
        );
        assert_eq!(
            replay(runtime.replay_journal()).unwrap(),
            runtime.replay_inspection()
        );
    }
}

#[test]
fn solo_journal_rejects_mismatched_and_unknown_policy_versions() {
    let mut runtime = solo_recording_runtime(grenadiers_core::room::CpuProfile::CURRENT);
    for version in [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, u16::MAX] {
        let journal = runtime.replay_journal_mut();
        journal.header.solo.as_mut().unwrap().policy_version = version;
        assert_eq!(replay(journal), Err(super::ReplayError::UnsupportedFormat));
    }
    let mut runtime = solo_recording_runtime(grenadiers_core::room::CpuProfile::INITIAL);
    let journal = runtime.replay_journal_mut();
    journal.header.solo.as_mut().unwrap().policy_version = 2;
    assert_eq!(replay(journal), Err(super::ReplayError::UnsupportedFormat));
}
