use super::runtime::RosterRuntime;
use grenadiers_core::room::CpuProfile;

/// Planner identity is independent of arena generation and wire protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum CpuPolicyFamily {
    Historical,
    Clearance,
    ReactiveHardV65,
    PickupHardV66,
    CombinationHardV67,
}

impl RosterRuntime {
    pub(in crate::game::rooms) const fn uses_clearance_cpu_profiles(&self) -> bool {
        matches!(
            self.cpu_family,
            CpuPolicyFamily::Clearance
                | CpuPolicyFamily::ReactiveHardV65
                | CpuPolicyFamily::PickupHardV66
                | CpuPolicyFamily::CombinationHardV67
        )
    }

    pub(in crate::game::rooms) const fn hard_cpu_profile(&self) -> Option<CpuProfile> {
        match self.cpu_family {
            CpuPolicyFamily::Historical => None,
            CpuPolicyFamily::Clearance => Some(CpuProfile::HARD_V64),
            CpuPolicyFamily::ReactiveHardV65 => Some(CpuProfile::HARD_V65),
            CpuPolicyFamily::PickupHardV66 => Some(CpuProfile::HARD_V66),
            CpuPolicyFamily::CombinationHardV67 => Some(CpuProfile::HARD_V67),
        }
    }
}
