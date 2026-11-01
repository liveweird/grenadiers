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
            ),
            _ => false,
        }
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
        let hard = controller(CpuProfile::HARD_V32);
        assert!(hard.profile_supports(10));
        assert!(controller(CpuProfile::HARD_V28).profile_supports(10));
        assert!(controller(CpuProfile::HARD_V29).profile_supports(10));
        for rules in 5..10 {
            assert!(!hard.profile_supports(rules));
        }
        for policy in [
            CpuPolicy::UtilityV1,
            CpuPolicy::UtilityV16,
            CpuPolicy::UtilityV27,
            CpuPolicy::UtilityV31,
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
        ] {
            for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
                let forged = controller(CpuProfile { policy, difficulty });
                assert!(!forged.profile_supports(10));
            }
        }
        for profile in [CpuProfile::PICKUP_V31_EASY, CpuProfile::PICKUP_V31_STANDARD] {
            assert!(controller(profile).profile_supports(10));
            assert!(!controller(profile).profile_supports(9));
        }
    }
}
