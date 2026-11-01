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
                        | grenadiers_core::room::CpuProfile::HARD_V53
                        | grenadiers_core::room::CpuProfile::HARD_V55
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
                    | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_STANDARD
                    | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_EASY
                    | grenadiers_core::room::CpuProfile::RECOVERY_V54_STANDARD
                    | grenadiers_core::room::CpuProfile::RECOVERY_V54_EASY
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
                | grenadiers_core::room::CpuProfile::HARD_V53
                | grenadiers_core::room::CpuProfile::HARD_V55
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
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_STANDARD
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_EASY
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_STANDARD
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
                | grenadiers_core::room::CpuProfile::HARD_V53
                | grenadiers_core::room::CpuProfile::HARD_V55
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
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_STANDARD
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_EASY
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_STANDARD
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
                | grenadiers_core::room::CpuProfile::HARD_V53
                | grenadiers_core::room::CpuProfile::HARD_V55
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
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_STANDARD
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_EASY
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_STANDARD
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
                | grenadiers_core::room::CpuProfile::HARD_V53
                | grenadiers_core::room::CpuProfile::HARD_V55
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
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_STANDARD
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_EASY
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_STANDARD
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
                | grenadiers_core::room::CpuProfile::HARD_V53
                | grenadiers_core::room::CpuProfile::HARD_V55
        )
    }

    pub(super) const fn supports_stable_dead_roster(&self) -> bool {
        matches!(
            self.profile,
            grenadiers_core::room::CpuProfile::TELEPORTER_V48_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V48_EASY
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_STANDARD
                | grenadiers_core::room::CpuProfile::TELEPORTER_V50_EASY
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_STANDARD
                | grenadiers_core::room::CpuProfile::PIT_AVOIDANCE_V52_EASY
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_STANDARD
                | grenadiers_core::room::CpuProfile::RECOVERY_V54_EASY
                | grenadiers_core::room::CpuProfile::HARD_V49
                | grenadiers_core::room::CpuProfile::HARD_V51
                | grenadiers_core::room::CpuProfile::HARD_V53
                | grenadiers_core::room::CpuProfile::HARD_V55
        )
    }
}

#[cfg(test)]
#[path = "controller_profile_tests.rs"]
mod tests;
