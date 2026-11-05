use super::*;
use crate::game::rooms::cpu::CpuPlanner;
use grenadiers_core::{
    game::SeatId,
    room::{CpuDifficulty, CpuPolicy, CpuProfile},
};

#[test]
fn owned_combination_support_is_exactly_hard67_or_hard69() {
    for (profile, expected) in [
        (CpuProfile::HARD_V65, false),
        (CpuProfile::HARD_V66, false),
        (CpuProfile::HARD_V67, true),
        (CpuProfile::HARD_V69, true),
        (
            CpuProfile {
                policy: CpuPolicy::UtilityV67,
                difficulty: CpuDifficulty::Standard,
            },
            false,
        ),
        (
            CpuProfile {
                policy: CpuPolicy::UtilityV69,
                difficulty: CpuDifficulty::Standard,
            },
            false,
        ),
        (
            CpuProfile {
                policy: CpuPolicy::UtilityV69,
                difficulty: CpuDifficulty::Easy,
            },
            false,
        ),
    ] {
        let controller = CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::Two,
            7,
            [67; 32],
            profile,
        );
        assert_eq!(controller.supports_owned_combination(), expected);
    }
}

#[test]
fn hard_requires_an_exact_supported_policy_and_rules10() {
    let controller = |profile| {
        CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::Two,
            7,
            [28; 32],
            profile,
        )
    };
    let hard = controller(CpuProfile::HARD_V45);
    for difficulty in [CpuDifficulty::Standard, CpuDifficulty::Hard] {
        assert!(
            !controller(CpuProfile {
                policy: CpuPolicy::UtilityV68,
                difficulty,
            })
            .profile_supports(10)
        );
    }
    assert!(hard.profile_supports(10));
    assert!(controller(CpuProfile::HARD_V28).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V29).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V32).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V34).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V35).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V37).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V39).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V41).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V43).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V45).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V47).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V49).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V51).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V53).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V55).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V57).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V58).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V59).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V61).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V63).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V64).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V65).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V66).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V67).profile_supports(10));
    assert!(controller(CpuProfile::HARD_V69).profile_supports(10));
    assert!(controller(CpuProfile::CERTIFIED_CADENCE_V68_EASY).profile_supports(10));
    for rules in 5..10 {
        assert!(!hard.profile_supports(rules));
    }
    for policy in [
        CpuPolicy::UtilityV1,
        CpuPolicy::UtilityV16,
        CpuPolicy::UtilityV27,
        CpuPolicy::UtilityV31,
        CpuPolicy::UtilityV33,
        CpuPolicy::UtilityV36,
        CpuPolicy::UtilityV38,
        CpuPolicy::UtilityV40,
        CpuPolicy::UtilityV42,
        CpuPolicy::UtilityV44,
        CpuPolicy::UtilityV46,
        CpuPolicy::UtilityV48,
        CpuPolicy::UtilityV50,
        CpuPolicy::UtilityV52,
        CpuPolicy::UtilityV54,
        CpuPolicy::UtilityV56,
        CpuPolicy::UtilityV60,
        CpuPolicy::UtilityV62,
    ] {
        let forged = controller(CpuProfile {
            policy,
            difficulty: CpuDifficulty::Hard,
        });
        for rules in 5..=10 {
            assert!(!forged.profile_supports(rules));
        }
    }
    for policy in [
        CpuPolicy::UtilityV28,
        CpuPolicy::UtilityV29,
        CpuPolicy::UtilityV32,
        CpuPolicy::UtilityV34,
        CpuPolicy::UtilityV35,
        CpuPolicy::UtilityV37,
        CpuPolicy::UtilityV39,
        CpuPolicy::UtilityV41,
        CpuPolicy::UtilityV43,
        CpuPolicy::UtilityV45,
        CpuPolicy::UtilityV47,
        CpuPolicy::UtilityV49,
        CpuPolicy::UtilityV51,
        CpuPolicy::UtilityV53,
        CpuPolicy::UtilityV55,
        CpuPolicy::UtilityV57,
        CpuPolicy::UtilityV58,
        CpuPolicy::UtilityV59,
        CpuPolicy::UtilityV61,
        CpuPolicy::UtilityV63,
        CpuPolicy::UtilityV64,
        CpuPolicy::UtilityV65,
        CpuPolicy::UtilityV66,
        CpuPolicy::UtilityV67,
    ] {
        for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
            let forged = controller(CpuProfile { policy, difficulty });
            assert!(!forged.profile_supports(10));
        }
    }
    for profile in [
        CpuProfile::PICKUP_V31_EASY,
        CpuProfile::PICKUP_V31_STANDARD,
        CpuProfile::REFUGE_V33_EASY,
        CpuProfile::REFUGE_V33_STANDARD,
        CpuProfile::FARM_REFUGES_V36_EASY,
        CpuProfile::FARM_REFUGES_V36_STANDARD,
        CpuProfile::FARM_ORIGINS_V38_EASY,
        CpuProfile::FARM_ORIGINS_V38_STANDARD,
        CpuProfile::CONTACT_TOPOLOGY_V40_EASY,
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
        CpuProfile::BLOCKED_BIRTHS_V42_EASY,
        CpuProfile::BLOCKED_BIRTHS_V42_STANDARD,
        CpuProfile::CONTINUATION_V44_EASY,
        CpuProfile::CONTINUATION_V44_STANDARD,
        CpuProfile::TELEPORTER_V46_EASY,
        CpuProfile::TELEPORTER_V46_STANDARD,
        CpuProfile::TELEPORTER_V48_EASY,
        CpuProfile::TELEPORTER_V48_STANDARD,
        CpuProfile::TELEPORTER_V50_STANDARD,
        CpuProfile::TELEPORTER_V50_EASY,
        CpuProfile::PIT_AVOIDANCE_V52_STANDARD,
        CpuProfile::PIT_AVOIDANCE_V52_EASY,
        CpuProfile::RECOVERY_V54_STANDARD,
        CpuProfile::RECOVERY_V54_EASY,
        CpuProfile::CORNER_REFUGES_V56_STANDARD,
        CpuProfile::CORNER_REFUGES_V56_EASY,
        CpuProfile::CONTACT_PUNCH_V60_STANDARD,
        CpuProfile::CONTACT_PUNCH_V60_EASY,
        CpuProfile::RECOVERY_V62_STANDARD,
        CpuProfile::RECOVERY_V62_EASY,
    ] {
        assert!(controller(profile).profile_supports(10));
        assert!(!controller(profile).profile_supports(9));
    }
    assert!(controller(CpuProfile::HARD_V35).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V63).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V64).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V65).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V66).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V67).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V69).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V37).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V39).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V41).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V43).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V45).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V47).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V49).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V51).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V53).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V55).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V57).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V58).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V59).supports_refuge_handoff());
    assert!(controller(CpuProfile::HARD_V61).supports_refuge_handoff());
    assert!(!controller(CpuProfile::HARD_V34).supports_refuge_handoff());
    assert!(!controller(CpuProfile::CERTIFIED_CADENCE_V68_EASY).supports_refuge_handoff());
    assert!(controller(CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD).supports_contact_topology());
    assert!(controller(CpuProfile::CONTACT_TOPOLOGY_V40_EASY).supports_contact_topology());
    assert!(controller(CpuProfile::HARD_V41).supports_contact_topology());
    assert!(controller(CpuProfile::BLOCKED_BIRTHS_V42_STANDARD).supports_contact_topology());
    assert!(controller(CpuProfile::HARD_V43).supports_contact_topology());
    assert!(controller(CpuProfile::HARD_V45).supports_contact_topology());
    assert!(controller(CpuProfile::HARD_V47).supports_contact_topology());
    assert!(controller(CpuProfile::CERTIFIED_CADENCE_V68_EASY).supports_contact_topology());
    assert!(controller(CpuProfile::BLOCKED_BIRTHS_V42_STANDARD).supports_blocked_birth_contact());
    assert!(controller(CpuProfile::HARD_V43).supports_blocked_birth_contact());
    assert!(controller(CpuProfile::HARD_V45).supports_blocked_birth_contact());
    assert!(controller(CpuProfile::HARD_V47).supports_blocked_birth_contact());
    assert!(controller(CpuProfile::CERTIFIED_CADENCE_V68_EASY).supports_blocked_birth_contact());
    assert!(controller(CpuProfile::CONTINUATION_V44_STANDARD).supports_continuation());
    assert!(controller(CpuProfile::HARD_V45).supports_continuation());
    assert!(controller(CpuProfile::HARD_V47).supports_continuation());
    assert!(controller(CpuProfile::TELEPORTER_V46_STANDARD).supports_continuation());
    assert!(controller(CpuProfile::TELEPORTER_V46_EASY).supports_teleporter_step());
    assert!(controller(CpuProfile::CERTIFIED_CADENCE_V68_EASY).supports_continuation());
    assert!(controller(CpuProfile::CERTIFIED_CADENCE_V68_EASY).supports_teleporter_step());
    assert!(controller(CpuProfile::HARD_V47).supports_teleporter_step());
    for profile in [
        CpuProfile::TELEPORTER_V48_STANDARD,
        CpuProfile::TELEPORTER_V50_STANDARD,
        CpuProfile::TELEPORTER_V50_EASY,
        CpuProfile::TELEPORTER_V48_EASY,
        CpuProfile::HARD_V49,
        CpuProfile::HARD_V51,
        CpuProfile::PIT_AVOIDANCE_V52_STANDARD,
        CpuProfile::PIT_AVOIDANCE_V52_EASY,
        CpuProfile::RECOVERY_V54_STANDARD,
        CpuProfile::RECOVERY_V54_EASY,
        CpuProfile::CORNER_REFUGES_V56_STANDARD,
        CpuProfile::CORNER_REFUGES_V56_EASY,
        CpuProfile::HARD_V53,
        CpuProfile::HARD_V55,
        CpuProfile::HARD_V57,
        CpuProfile::HARD_V58,
        CpuProfile::HARD_V59,
        CpuProfile::CONTACT_PUNCH_V60_STANDARD,
        CpuProfile::CONTACT_PUNCH_V60_EASY,
        CpuProfile::HARD_V61,
        CpuProfile::HARD_V65,
        CpuProfile::HARD_V66,
        CpuProfile::HARD_V67,
        CpuProfile::HARD_V69,
    ] {
        let current = controller(profile);
        assert!(current.supports_contact_topology());
        assert!(current.supports_blocked_birth_contact());
        assert!(current.supports_continuation());
        assert!(current.supports_teleporter_step());
        assert!(current.supports_stable_dead_roster());
    }
    assert!(!controller(CpuProfile::TELEPORTER_V46_STANDARD).supports_stable_dead_roster());
    assert!(!controller(CpuProfile::TELEPORTER_V46_EASY).supports_stable_dead_roster());
    assert!(!controller(CpuProfile::HARD_V47).supports_stable_dead_roster());
    assert!(!controller(CpuProfile::HARD_V45).supports_teleporter_step());
    assert!(!controller(CpuProfile::HARD_V43).supports_continuation());
    assert!(!controller(CpuProfile::HARD_V41).supports_blocked_birth_contact());
    assert!(!controller(CpuProfile::FARM_ORIGINS_V38_STANDARD).supports_contact_topology());
    assert!(!controller(CpuProfile::HARD_V39).supports_contact_topology());
}
