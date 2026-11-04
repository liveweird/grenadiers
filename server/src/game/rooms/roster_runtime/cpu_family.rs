use super::runtime::RosterRuntime;

/// Planner identity is independent of arena generation and wire protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum CpuPolicyFamily {
    Historical,
    Clearance,
    ReactiveHardV65,
}

impl RosterRuntime {
    pub(in crate::game::rooms) const fn uses_clearance_cpu_profiles(&self) -> bool {
        matches!(
            self.cpu_family,
            CpuPolicyFamily::Clearance | CpuPolicyFamily::ReactiveHardV65
        )
    }

    pub(in crate::game::rooms) const fn uses_reactive_hard_profile(&self) -> bool {
        matches!(self.cpu_family, CpuPolicyFamily::ReactiveHardV65)
    }
}
