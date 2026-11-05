use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId},
    room::{BombCapacityDropMultiplier, CpuDifficulty, RoomSettings},
};

use super::{
    super::{
        replay_v38::{self, RecordingRosterRuntime},
        roster_runtime::RosterAction,
    },
    live_cpu_evaluation::{configuration, drain_trace, start_round},
};
use crate::round_seed::RoundSeedSource;

#[test]
fn live_evaluation_partial_flush_stop_bomb_and_fence_replay_exactly() {
    let mut shadow = RecordingRosterRuntime::new_for_evaluation(
        configuration(CpuDifficulty::Hard),
        RoomSettings::DEFAULT
            .with_enabled_items(EnabledItems::ALL)
            .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(41);
            Ok(())
        }),
        [113; 32],
        [SeatId::Two, SeatId::Three, SeatId::Four]
            .into_iter()
            .map(|seat| (seat, CpuDifficulty::Hard, [71 + seat.index() as u8; 32]))
            .collect(),
    )
    .unwrap();
    start_round(&mut shadow).unwrap();
    let mut actual = shadow.runtime().clone();
    actual.enable_evaluation_trace_for_test();
    actual
        .action(
            SeatId::One,
            1,
            RosterAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    actual
        .action(SeatId::One, 1, RosterAction::Bomb(1))
        .unwrap();
    actual.current_notice().unwrap();
    actual.stop_seat_inputs(SeatId::One);
    actual.settle_boundary([None; 4]).unwrap();
    drain_trace(&mut actual, &mut shadow).unwrap();

    let expected = actual.replay_inspection();
    assert_eq!(shadow.runtime().replay_inspection(), expected);
    let (_, journal) = shadow.into_parts();
    replay_v38::verify_exact(&journal, &expected).unwrap();
}

#[test]
fn archived_replay37_evaluation_keeps_original_policy_family_and_exact_replay() {
    use grenadiers_core::room::CpuProfile;

    let archived = super::super::replay_v37::RecordingRosterRuntime::new_for_evaluation(
        configuration(CpuDifficulty::Easy),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(41);
            Ok(())
        }),
        [113; 32],
        [SeatId::Two, SeatId::Three, SeatId::Four]
            .into_iter()
            .map(|seat| (seat, CpuDifficulty::Easy, [71 + seat.index() as u8; 32]))
            .collect(),
    )
    .unwrap();
    assert_eq!(
        archived.runtime().easy_cpu_profile(),
        Some(CpuProfile::CERTIFIED_CADENCE_V68_EASY)
    );
    assert_eq!(
        archived.runtime().hard_cpu_profile(),
        Some(CpuProfile::HARD_V67)
    );
    let expected = archived.runtime().replay_inspection();
    let (_, journal) = archived.into_parts();
    super::super::replay_v37::verify_exact(&journal, &expected).unwrap();
}

#[test]
fn archived_replay36_evaluation_facade_remains_closed_and_replayable() {
    use grenadiers_core::room::CpuProfile;

    let archived = super::super::replay_v36::RecordingRosterRuntime::new_for_evaluation(
        configuration(CpuDifficulty::Hard),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(36);
            Ok(())
        }),
        [113; 32],
        [SeatId::Two, SeatId::Three, SeatId::Four]
            .into_iter()
            .map(|seat| (seat, CpuDifficulty::Hard, [71 + seat.index() as u8; 32]))
            .collect(),
    )
    .unwrap();
    assert_eq!(
        archived.runtime().hard_cpu_profile(),
        Some(CpuProfile::HARD_V67)
    );
    assert_eq!(
        archived.runtime().easy_cpu_profile(),
        Some(CpuProfile::RECOVERY_V62_EASY)
    );
    let expected = archived.runtime().replay_inspection();
    let (_, journal) = archived.into_parts();
    super::super::replay_v36::verify_exact(&journal, &expected).unwrap();
}
