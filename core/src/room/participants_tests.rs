use super::*;

#[test]
fn human_duel_has_two_human_controllers() {
    for seat in SeatId::DUEL {
        assert_eq!(
            RoomMode::HumanDuel.controller(seat),
            Some(ControllerKind::Human)
        );
    }
    assert_eq!(RoomMode::HumanDuel.controller(SeatId::Three), None);
    assert_eq!(RoomMode::HumanDuel.controller(SeatId::Four), None);
}

#[test]
fn solo_keeps_creator_human_and_cpu_profile_on_second_seat() {
    let mode = RoomMode::Solo(CpuProfile::INITIAL);
    assert_eq!(mode.controller(SeatId::One), Some(ControllerKind::Human));
    assert_eq!(
        mode.controller(SeatId::Two),
        Some(ControllerKind::Cpu(CpuProfile {
            policy: CpuPolicy::UtilityV1,
            difficulty: CpuDifficulty::Standard,
        }))
    );
}

#[test]
fn policy_versions_are_independent_of_controller_assignment() {
    assert_eq!(CpuProfile::INITIAL.policy.version(), 1);
    assert_eq!(CpuProfile::PULSE_V2.policy.version(), 2);
    assert_eq!(CpuProfile::ROUTE_V3.policy.version(), 3);
    assert_eq!(CpuProfile::PLANT_V4.policy.version(), 4);
    assert_eq!(CpuProfile::GOALS_V5.policy.version(), 5);
    assert_eq!(CpuProfile::WEAPONS_V6.policy.version(), 6);
    assert_eq!(CpuProfile::WALL_PASS_V7.policy.version(), 7);
    assert_eq!(CpuProfile::RESPONSIVE_V8.policy.version(), 8);
    assert_eq!(CpuProfile::ORIGIN_V9.policy.version(), 9);
    assert_eq!(CpuProfile::GUIDED_V10.policy.version(), 10);
    assert_eq!(CpuProfile::CLEAR_GAP_V11.policy.version(), 11);
    assert_eq!(CpuProfile::STEERING_V12.policy.version(), 12);
    assert_eq!(CpuProfile::BRACELET_V13.policy.version(), 13);
    assert_eq!(CpuProfile::PUNCH_V14.policy.version(), 14);
    assert_eq!(CpuProfile::EQUIPMENT_GOALS_V15.policy.version(), 15);
    assert_eq!(CpuProfile::CURRENT.policy.version(), 16);
    assert_eq!(CpuProfile::CURRENT_EASY.policy.version(), 16);
    assert_eq!(CpuProfile::MULTIPLAYER_STANDARD.policy.version(), 17);
    assert_eq!(CpuProfile::MULTIPLAYER_EASY.policy.version(), 17);
    assert_eq!(CpuProfile::ENVIRONMENT_STANDARD.policy.version(), 18);
    assert_eq!(CpuProfile::ENVIRONMENT_EASY.policy.version(), 18);
    assert_eq!(CpuProfile::LAIRS_V19_STANDARD.policy.version(), 19);
    assert_eq!(CpuProfile::LAIRS_V19_EASY.policy.version(), 19);
    assert_eq!(CpuProfile::LAIRS_STANDARD.policy.version(), 20);
    assert_eq!(CpuProfile::LAIRS_EASY.policy.version(), 20);
    assert_eq!(CpuProfile::RESPAWN_LAIRS_STANDARD.policy.version(), 25);
    assert_eq!(CpuProfile::RESPAWN_V24_STANDARD.policy.version(), 24);
    assert_eq!(CpuProfile::RESPAWN_V23_STANDARD.policy.version(), 23);
    assert_eq!(CpuProfile::RESPAWN_V22_STANDARD.policy.version(), 22);
    assert_eq!(CpuProfile::RESPAWN_V21_STANDARD.policy.version(), 21);
    assert_eq!(CpuProfile::RESPAWN_LAIRS_EASY.policy.version(), 25);
    assert_eq!(CpuProfile::RESPAWN_V24_EASY.policy.version(), 24);
    assert_eq!(CpuProfile::RESPAWN_V23_EASY.policy.version(), 23);
    assert_eq!(CpuProfile::RESPAWN_V22_EASY.policy.version(), 22);
    assert_eq!(CpuProfile::RESPAWN_V21_EASY.policy.version(), 21);
    assert_eq!(CpuProfile::SPIKE_PITS_STANDARD.policy.version(), 26);
    assert_eq!(CpuProfile::SPIKE_PITS_EASY.policy.version(), 26);
    assert_eq!(CpuProfile::RELIABLE_V27_STANDARD.policy.version(), 27);
    assert_eq!(CpuProfile::RELIABLE_V27_EASY.policy.version(), 27);
    assert!(CpuProfile::CURRENT.is_current());
    assert!(CpuProfile::CURRENT_EASY.is_current());
    assert!(!CpuProfile::MULTIPLAYER_STANDARD.is_current());
    assert!(!CpuProfile::MULTIPLAYER_EASY.is_current());
    assert!(!CpuProfile::RELIABLE_V27_STANDARD.is_current());
    assert!(!CpuProfile::RELIABLE_V27_EASY.is_current());
    assert!(!CpuProfile::EQUIPMENT_GOALS_V15.is_current());
    assert!(
        !CpuProfile {
            policy: CpuPolicy::UtilityV15,
            difficulty: CpuDifficulty::Easy,
        }
        .is_current()
    );
    assert_eq!(
        RoomMode::Solo(CpuProfile::CURRENT).controller(SeatId::Two),
        Some(ControllerKind::Cpu(CpuProfile::CURRENT))
    );
}

#[test]
fn hard28_is_distinct_from_current_and_archived_cpu_profiles() {
    assert_eq!(CpuProfile::HARD_V28.policy.version(), 28);
    assert_eq!(CpuProfile::HARD_V28.difficulty, CpuDifficulty::Hard);
    assert!(!CpuProfile::HARD_V28.is_current());
    assert_eq!(CpuProfile::RELIABLE_V27_STANDARD.policy.version(), 27);
    assert_eq!(CpuProfile::SPIKE_PITS_STANDARD.policy.version(), 26);
    assert_eq!(CpuProfile::CURRENT.policy.version(), 16);
}

#[test]
fn hard29_is_distinct_while_hard28_and_legacy_profiles_remain_immutable() {
    assert_eq!(CpuProfile::HARD_V29.policy.version(), 29);
    assert_eq!(CpuProfile::HARD_V29.difficulty, CpuDifficulty::Hard);
    assert!(!CpuProfile::HARD_V29.is_current());
    assert_eq!(CpuProfile::HARD_V28.policy.version(), 28);
    assert_eq!(CpuProfile::RELIABLE_V27_STANDARD.policy.version(), 27);
    assert_eq!(CpuProfile::CURRENT.policy.version(), 16);
}

#[test]
fn pickup_profiles_preserve_retained_policy_and_difficulty_identities() {
    assert_eq!(CpuProfile::PICKUP_V31_STANDARD.policy.version(), 31);
    assert_eq!(
        CpuProfile::PICKUP_V31_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::PICKUP_V31_EASY.policy.version(), 31);
    assert_eq!(CpuProfile::PICKUP_V31_EASY.difficulty, CpuDifficulty::Easy);
    assert_eq!(CpuProfile::HARD_V32.policy.version(), 32);
    assert_eq!(CpuProfile::HARD_V32.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::RELIABLE_V27_STANDARD.policy.version(), 27);
    assert_eq!(CpuProfile::HARD_V29.policy.version(), 29);
    assert!(!CpuProfile::PICKUP_V31_STANDARD.is_current());
    assert!(!CpuProfile::PICKUP_V31_EASY.is_current());
    assert!(!CpuProfile::HARD_V32.is_current());
}
