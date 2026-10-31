#[derive(Clone, Copy)]
pub(super) enum PlannerVersion {
    Legacy,
    Lairs,
    Respawn,
    RespawnV23,
    RespawnV24,
    RespawnV25,
    SpikePitsV26,
}

impl PlannerVersion {
    pub(super) const fn follow_committed(self) -> bool {
        matches!(self, Self::Legacy)
    }

    pub(super) const fn persistent_goals(self) -> bool {
        matches!(
            self,
            Self::RespawnV23 | Self::RespawnV24 | Self::RespawnV25 | Self::SpikePitsV26
        )
    }

    pub(super) const fn strategic_goals(self) -> bool {
        matches!(
            self,
            Self::RespawnV24 | Self::RespawnV25 | Self::SpikePitsV26
        )
    }

    pub(super) const fn failed_plant_recovery(self) -> bool {
        matches!(self, Self::RespawnV25 | Self::SpikePitsV26)
    }
}
