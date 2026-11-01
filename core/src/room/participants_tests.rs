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

#[test]
fn refuge_profiles_keep_prior_pickup_policy_identities() {
    assert_eq!(CpuProfile::REFUGE_V33_STANDARD.policy.version(), 33);
    assert_eq!(
        CpuProfile::REFUGE_V33_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::REFUGE_V33_EASY.policy.version(), 33);
    assert_eq!(CpuProfile::REFUGE_V33_EASY.difficulty, CpuDifficulty::Easy);
    assert_eq!(CpuProfile::HARD_V34.policy.version(), 34);
    assert_eq!(CpuProfile::HARD_V34.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::PICKUP_V31_STANDARD.policy.version(), 31);
    assert_eq!(CpuProfile::HARD_V32.policy.version(), 32);
    assert!(!CpuProfile::REFUGE_V33_STANDARD.is_current());
    assert!(!CpuProfile::REFUGE_V33_EASY.is_current());
    assert!(!CpuProfile::HARD_V34.is_current());
}

#[test]
fn hard35_adds_handoff_without_relabeling_refuge_profiles() {
    assert_eq!(CpuProfile::HARD_V35.policy.version(), 35);
    assert_eq!(CpuProfile::HARD_V35.difficulty, CpuDifficulty::Hard);
    assert!(!CpuProfile::HARD_V35.is_current());
    assert_eq!(CpuProfile::HARD_V34.policy.version(), 34);
    assert_eq!(CpuProfile::REFUGE_V33_STANDARD.policy.version(), 33);
    assert_eq!(CpuProfile::REFUGE_V33_EASY.policy.version(), 33);
}

#[test]
fn farm_refuge_profiles_extend_without_relabeling_prior_policies() {
    assert_eq!(CpuProfile::FARM_REFUGES_V36_STANDARD.policy.version(), 36);
    assert_eq!(
        CpuProfile::FARM_REFUGES_V36_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::FARM_REFUGES_V36_EASY.policy.version(), 36);
    assert_eq!(
        CpuProfile::FARM_REFUGES_V36_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V37.policy.version(), 37);
    assert_eq!(CpuProfile::HARD_V37.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::HARD_V35.policy.version(), 35);
    assert_eq!(CpuProfile::REFUGE_V33_STANDARD.policy.version(), 33);
    assert!(!CpuProfile::FARM_REFUGES_V36_STANDARD.is_current());
    assert!(!CpuProfile::FARM_REFUGES_V36_EASY.is_current());
    assert!(!CpuProfile::HARD_V37.is_current());
}

#[test]
fn farm_origin_profiles_extend_without_relabeling_prior_policies() {
    assert_eq!(CpuProfile::FARM_ORIGINS_V38_STANDARD.policy.version(), 38);
    assert_eq!(
        CpuProfile::FARM_ORIGINS_V38_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::FARM_ORIGINS_V38_EASY.policy.version(), 38);
    assert_eq!(
        CpuProfile::FARM_ORIGINS_V38_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V39.policy.version(), 39);
    assert_eq!(CpuProfile::HARD_V39.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::HARD_V37.policy.version(), 37);
    assert_eq!(CpuProfile::FARM_REFUGES_V36_STANDARD.policy.version(), 36);
    assert!(!CpuProfile::FARM_ORIGINS_V38_STANDARD.is_current());
    assert!(!CpuProfile::FARM_ORIGINS_V38_EASY.is_current());
    assert!(!CpuProfile::HARD_V39.is_current());
}

#[test]
fn contact_topology_profiles_extend_without_relabeling_prior_policies() {
    assert_eq!(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD.policy.version(),
        40
    );
    assert_eq!(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::CONTACT_TOPOLOGY_V40_EASY.policy.version(), 40);
    assert_eq!(
        CpuProfile::CONTACT_TOPOLOGY_V40_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V41.policy.version(), 41);
    assert_eq!(CpuProfile::HARD_V41.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::HARD_V39.policy.version(), 39);
    assert_eq!(CpuProfile::FARM_ORIGINS_V38_STANDARD.policy.version(), 38);
    assert!(!CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD.is_current());
    assert!(!CpuProfile::CONTACT_TOPOLOGY_V40_EASY.is_current());
    assert!(!CpuProfile::HARD_V41.is_current());
}

#[test]
fn blocked_birth_profiles_extend_without_relabeling_contact_topology() {
    assert_eq!(CpuProfile::BLOCKED_BIRTHS_V42_STANDARD.policy.version(), 42);
    assert_eq!(
        CpuProfile::BLOCKED_BIRTHS_V42_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::BLOCKED_BIRTHS_V42_EASY.policy.version(), 42);
    assert_eq!(
        CpuProfile::BLOCKED_BIRTHS_V42_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V43.policy.version(), 43);
    assert_eq!(CpuProfile::HARD_V43.difficulty, CpuDifficulty::Hard);
    assert_eq!(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD.policy.version(),
        40
    );
    assert_eq!(CpuProfile::HARD_V41.policy.version(), 41);
    assert!(!CpuProfile::BLOCKED_BIRTHS_V42_STANDARD.is_current());
    assert!(!CpuProfile::BLOCKED_BIRTHS_V42_EASY.is_current());
    assert!(!CpuProfile::HARD_V43.is_current());
}

#[test]
fn continuation_profiles_extend_without_relabeling_blocked_births() {
    assert_eq!(CpuProfile::CONTINUATION_V44_STANDARD.policy.version(), 44);
    assert_eq!(
        CpuProfile::CONTINUATION_V44_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(CpuProfile::CONTINUATION_V44_EASY.policy.version(), 44);
    assert_eq!(
        CpuProfile::CONTINUATION_V44_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V45.policy.version(), 45);
    assert_eq!(CpuProfile::HARD_V45.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::BLOCKED_BIRTHS_V42_STANDARD.policy.version(), 42);
    assert_eq!(CpuProfile::HARD_V43.policy.version(), 43);
    assert!(!CpuProfile::CONTINUATION_V44_STANDARD.is_current());
    assert!(!CpuProfile::CONTINUATION_V44_EASY.is_current());
    assert!(!CpuProfile::HARD_V45.is_current());
}

#[test]
fn teleporter_profiles_extend_for_stable_dead_rosters_without_relabeling_archived_policies() {
    assert_eq!(CpuProfile::TELEPORTER_V46_STANDARD.policy.version(), 46);
    assert_eq!(CpuProfile::TELEPORTER_V46_EASY.policy.version(), 46);
    assert_eq!(CpuProfile::HARD_V47.policy.version(), 47);
    assert_eq!(
        CpuProfile::TELEPORTER_V46_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(
        CpuProfile::TELEPORTER_V46_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V47.difficulty, CpuDifficulty::Hard);
    assert!(!CpuProfile::TELEPORTER_V46_STANDARD.is_current());
    assert!(!CpuProfile::TELEPORTER_V46_EASY.is_current());
    assert!(!CpuProfile::HARD_V47.is_current());
    assert_eq!(CpuProfile::TELEPORTER_V48_STANDARD.policy.version(), 48);
    assert_eq!(CpuProfile::TELEPORTER_V48_EASY.policy.version(), 48);
    assert_eq!(CpuProfile::HARD_V49.policy.version(), 49);
    assert_eq!(
        CpuProfile::TELEPORTER_V48_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(
        CpuProfile::TELEPORTER_V48_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V49.difficulty, CpuDifficulty::Hard);
    assert!(!CpuProfile::TELEPORTER_V48_STANDARD.is_current());
    assert!(!CpuProfile::TELEPORTER_V48_EASY.is_current());
    assert!(!CpuProfile::HARD_V49.is_current());
    assert_eq!(CpuProfile::CURRENT.policy.version(), 16);
    assert_eq!(CpuProfile::CURRENT_EASY.policy.version(), 16);
}

#[test]
fn multiple_teleporter_profiles_preserve_the_previous_pair() {
    assert_eq!(CpuProfile::TELEPORTER_V50_STANDARD.policy.version(), 50);
    assert_eq!(CpuProfile::TELEPORTER_V50_EASY.policy.version(), 50);
    assert_eq!(CpuProfile::HARD_V51.policy.version(), 51);
    assert_eq!(CpuProfile::TELEPORTER_V48_STANDARD.policy.version(), 48);
    assert_eq!(CpuProfile::HARD_V49.policy.version(), 49);
    assert_eq!(
        CpuProfile::TELEPORTER_V50_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(
        CpuProfile::TELEPORTER_V50_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V51.difficulty, CpuDifficulty::Hard);
}

#[test]
fn pit_avoidance_profiles_extend_without_relabeling_the_previous_pair() {
    assert_eq!(CpuProfile::PIT_AVOIDANCE_V52_STANDARD.policy.version(), 52);
    assert_eq!(CpuProfile::PIT_AVOIDANCE_V52_EASY.policy.version(), 52);
    assert_eq!(CpuProfile::HARD_V53.policy.version(), 53);
    assert_eq!(
        CpuProfile::PIT_AVOIDANCE_V52_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(
        CpuProfile::PIT_AVOIDANCE_V52_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V53.difficulty, CpuDifficulty::Hard);
    assert_eq!(CpuProfile::TELEPORTER_V50_STANDARD.policy.version(), 50);
    assert_eq!(CpuProfile::HARD_V51.policy.version(), 51);
}

#[test]
fn shared_recovery_profiles_have_closed_version_and_difficulty_pairs() {
    use super::{CpuDifficulty, CpuProfile};
    assert_eq!(CpuProfile::RECOVERY_V54_STANDARD.policy.version(), 54);
    assert_eq!(CpuProfile::RECOVERY_V54_EASY.policy.version(), 54);
    assert_eq!(CpuProfile::HARD_V55.policy.version(), 55);
    assert_eq!(
        CpuProfile::RECOVERY_V54_STANDARD.difficulty,
        CpuDifficulty::Standard
    );
    assert_eq!(
        CpuProfile::RECOVERY_V54_EASY.difficulty,
        CpuDifficulty::Easy
    );
    assert_eq!(CpuProfile::HARD_V55.difficulty, CpuDifficulty::Hard);
}
