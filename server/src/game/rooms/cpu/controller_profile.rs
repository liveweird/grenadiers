use super::CpuController;

impl CpuController {
    pub(super) fn profile_supports(&self, rules_version: u16) -> bool {
        if self.profile.difficulty == grenadiers_core::room::CpuDifficulty::Hard {
            return rules_version == 10
                && matches!(
                    self.profile,
                    grenadiers_core::room::CpuProfile::HARD_V28
                        | grenadiers_core::room::CpuProfile::HARD_V29
                        | grenadiers_core::room::CpuProfile::HARD_V32
                        | grenadiers_core::room::CpuProfile::HARD_V34
                        | grenadiers_core::room::CpuProfile::HARD_V35
                        | grenadiers_core::room::CpuProfile::HARD_V37
                        | grenadiers_core::room::CpuProfile::HARD_V39
                        | grenadiers_core::room::CpuProfile::HARD_V41
                        | grenadiers_core::room::CpuProfile::HARD_V43
                        | grenadiers_core::room::CpuProfile::HARD_V45
                        | grenadiers_core::room::CpuProfile::HARD_V47
                        | grenadiers_core::room::CpuProfile::HARD_V49
                        | grenadiers_core::room::CpuProfile::HARD_V51
                );
        }
        match rules_version {
            5 => self.profile.policy.version() <= 16,
            6 => matches!(
                self.profile,
                grenadiers_core::room::CpuProfile::MULTIPLAYER_STANDARD
                    | grenadiers_core::room::CpuProfile::MULTIPLAYER_EASY
            ),
            7 => matches!(
                self.profile,
                grenadiers_core::room::CpuProfile::ENVIRONMENT_STANDARD
                    | grenadiers_core::room::CpuProfile::ENVIRONMENT_EASY
            ),
            8 => matches!(
                self.profile,
                grenadiers_core::room::CpuProfile::LAIRS_STANDARD
                    | grenadiers_core::room::CpuProfile::LAIRS_EASY
            ),
            9 => matches!(
                self.profile,
                grenadiers_core::room::CpuProfile::RESPAWN_LAIRS_STANDARD
                    | grenadiers_core::room::CpuProfile::RESPAWN_LAIRS_EASY
                    | grenadiers_core::room::CpuProfile::RESPAWN_V22_STANDARD
                    | grenadiers_core::room::CpuProfile::RESPAWN_V22_EASY
                    | grenadiers_core::room::CpuProfile::RESPAWN_V24_STANDARD
                    | grenadiers_core::room::CpuProfile::RESPAWN_V24_EASY
                    | grenadiers_core::room::CpuProfile::RESPAWN_V23_STANDARD
                    | grenadiers_core::room::CpuProfile::RESPAWN_V23_EASY
                    | grenadiers_core::room::CpuProfile::RESPAWN_V21_STANDARD
                    | grenadiers_core::room::CpuProfile::RESPAWN_V21_EASY
            ),
            10 => matches!(
                self.profile,
                grenadiers_core::room::CpuProfile::SPIKE_PITS_STANDARD
                    | grenadiers_core::room::CpuProfile::SPIKE_PITS_EASY
                    | grenadiers_core::room::CpuProfile::RELIABLE_V27_STANDARD
                    | grenadiers_core::room::CpuProfile::RELIABLE_V27_EASY
                    | grenadiers_core::room::CpuProfile::PICKUP_V31_STANDARD
                    | grenadiers_core::room::CpuProfile::PICKUP_V31_EASY
                    | grenadiers_core::room::CpuProfile::REFUGE_V33_STANDARD
                    | grenadiers_core::room::CpuProfile::REFUGE_V33_EASY
                    | grenadiers_core::room::CpuProfile::FARM_REFUGES_V36_STANDARD
                    | grenadiers_core::room::CpuProfile::FARM_REFUGES_V36_EASY
                    | grenadiers_core::room::CpuProfile::FARM_ORIGINS_V38_STANDARD
                    | grenadiers_core::room::CpuProfile::FARM_ORIGINS_V38_EASY
                    | grenadiers_core::room::CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD
                    | grenadiers_core::room::CpuProfile::CONTACT_TOPOLOGY_V40_EASY
                    | grenadiers_core::room::CpuProfile::BLOCKED_BIRTHS_V42_STANDARD
                    | grenadiers_core::room::CpuProfile::BLOCKED_BIRTHS_V42_EASY
                    | grenadiers_core::room::CpuProfile::CONTINUATION_V44_STANDARD
                    | grenadiers_core::room::CpuProfile::CONTINUATION_V44_EASY
                    | grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                    | grenadiers_core::room::CpuProfile::TELEPORTER_V46_EASY
                    | grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                    | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                    | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                    | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
            ),
            _ => false,
        }
    }

    pub(super) const fn supports_refuge_handoff(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::HARD_V35
                | grenadiers_core::room::CpuProfile::HARD_V37
                | grenadiers_core::room::CpuProfile::HARD_V39
                | grenadiers_core::room::CpuProfile::HARD_V41
                | grenadiers_core::room::CpuProfile::HARD_V43
                | grenadiers_core::room::CpuProfile::HARD_V45
                | grenadiers_core::room::CpuProfile::HARD_V47
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
        )
    }

    pub(super) const fn supports_contact_topology(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD
                | grenadiers_core::room::CpuProfile::CONTACT_TOPOLOGY_V40_EASY
                | grenadiers_core::room::CpuProfile::HARD_V41
                | grenadiers_core::room::CpuProfile::BLOCKED_BIRTHS_V42_STANDARD
                | grenadiers_core::room::CpuProfile::BLOCKED_BIRTHS_V42_EASY
                | grenadiers_core::room::CpuProfile::HARD_V43
                | grenadiers_core::room::CpuProfile::CONTINUATION_V44_STANDARD
                | grenadiers_core::room::CpuProfile::CONTINUATION_V44_EASY
                | grenadiers_core::room::CpuProfile::HARD_V45
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_EASY
                | grenadiers_core::room::CpuProfile::HARD_V47
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
        )
    }

    pub(super) const fn supports_blocked_birth_contact(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::BLOCKED_BIRTHS_V42_STANDARD
                | grenadiers_core::room::CpuProfile::BLOCKED_BIRTHS_V42_EASY
                | grenadiers_core::room::CpuProfile::HARD_V43
                | grenadiers_core::room::CpuProfile::CONTINUATION_V44_STANDARD
                | grenadiers_core::room::CpuProfile::CONTINUATION_V44_EASY
                | grenadiers_core::room::CpuProfile::HARD_V45
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_EASY
                | grenadiers_core::room::CpuProfile::HARD_V47
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
        )
    }

    pub(super) const fn supports_continuation(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::CONTINUATION_V44_STANDARD
                | grenadiers_core::room::CpuProfile::CONTINUATION_V44_EASY
                | grenadiers_core::room::CpuProfile::HARD_V45
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_EASY
                | grenadiers_core::room::CpuProfile::HARD_V47
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
        )
    }

    pub(super) const fn supports_teleporter_step(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V46_EASY
                | grenadiers_core::room::CpuProfile::HARD_V47
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
        )
    }

    pub(super) const fn supports_stable_dead_roster(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rooms::cpu::CpuPlanner;
    use grenadiers_core::{
        game::SeatId,
        room::{CpuDifficulty, CpuPolicy, CpuProfile},
    };

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
        ] {
            assert!(controller(profile).profile_supports(10));
            assert!(!controller(profile).profile_supports(9));
        }
        assert!(controller(CpuProfile::HARD_V35).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V37).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V39).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V41).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V43).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V45).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V47).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V49).supports_refuge_handoff());
        assert!(controller(CpuProfile::HARD_V51).supports_refuge_handoff());
        assert!(!controller(CpuProfile::HARD_V34).supports_refuge_handoff());
        assert!(controller(CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD).supports_contact_topology());
        assert!(controller(CpuProfile::CONTACT_TOPOLOGY_V40_EASY).supports_contact_topology());
        assert!(controller(CpuProfile::HARD_V41).supports_contact_topology());
        assert!(controller(CpuProfile::BLOCKED_BIRTHS_V42_STANDARD).supports_contact_topology());
        assert!(controller(CpuProfile::HARD_V43).supports_contact_topology());
        assert!(controller(CpuProfile::HARD_V45).supports_contact_topology());
        assert!(controller(CpuProfile::HARD_V47).supports_contact_topology());
        assert!(
            controller(CpuProfile::BLOCKED_BIRTHS_V42_STANDARD).supports_blocked_birth_contact()
        );
        assert!(controller(CpuProfile::HARD_V43).supports_blocked_birth_contact());
        assert!(controller(CpuProfile::HARD_V45).supports_blocked_birth_contact());
        assert!(controller(CpuProfile::HARD_V47).supports_blocked_birth_contact());
        assert!(controller(CpuProfile::CONTINUATION_V44_STANDARD).supports_continuation());
        assert!(controller(CpuProfile::HARD_V45).supports_continuation());
        assert!(controller(CpuProfile::HARD_V47).supports_continuation());
        assert!(controller(CpuProfile::TELEPORTER_V46_STANDARD).supports_continuation());
        assert!(controller(CpuProfile::TELEPORTER_V46_EASY).supports_teleporter_step());
        assert!(controller(CpuProfile::HARD_V47).supports_teleporter_step());
        for profile in [
            CpuProfile::TELEPORTER_V48_STANDARD,
            CpuProfile::TELEPORTER_V50_STANDARD,
            CpuProfile::TELEPORTER_V50_EASY,
            CpuProfile::TELEPORTER_V48_EASY,
            CpuProfile::HARD_V49,
            CpuProfile::HARD_V51,
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
}
