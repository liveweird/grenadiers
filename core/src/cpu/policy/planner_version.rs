#[derive(Clone, Copy)]
pub(in crate::cpu::policy) enum PlannerVersion {
    Legacy,
    Lairs,
    Respawn,
    RespawnV23,
    RespawnV24,
    RespawnV25,
    SpikePitsV26,
    HardV28,
    FarmV38,
    HardFarmV39,
    FarmPitsV52,
    HardFarmPitsV53,
    SpikePitsCautiousV52,
    HardPitsCautiousV53,
    HardFarmExitPressureV58,
    HardPitsExitPressureV58,
    HardFarmYieldV59,
    HardPitsYieldV59,
}

impl PlannerVersion {
    pub(super) const fn follow_committed(self) -> bool {
        matches!(self, Self::Legacy)
    }

    pub(super) const fn persistent_goals(self) -> bool {
        matches!(
            self,
            Self::RespawnV23
                | Self::RespawnV24
                | Self::RespawnV25
                | Self::SpikePitsV26
                | Self::HardV28
                | Self::FarmV38
                | Self::HardFarmV39
                | Self::FarmPitsV52
                | Self::HardFarmPitsV53
                | Self::SpikePitsCautiousV52
                | Self::HardPitsCautiousV53
                | Self::HardFarmExitPressureV58
                | Self::HardPitsExitPressureV58
                | Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
        )
    }

    pub(super) const fn strategic_goals(self) -> bool {
        matches!(
            self,
            Self::RespawnV24
                | Self::RespawnV25
                | Self::SpikePitsV26
                | Self::HardV28
                | Self::FarmV38
                | Self::HardFarmV39
                | Self::FarmPitsV52
                | Self::HardFarmPitsV53
                | Self::SpikePitsCautiousV52
                | Self::HardPitsCautiousV53
                | Self::HardFarmExitPressureV58
                | Self::HardPitsExitPressureV58
                | Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
        )
    }

    pub(super) const fn failed_plant_recovery(self) -> bool {
        matches!(
            self,
            Self::RespawnV25
                | Self::SpikePitsV26
                | Self::HardV28
                | Self::FarmV38
                | Self::HardFarmV39
                | Self::FarmPitsV52
                | Self::HardFarmPitsV53
                | Self::SpikePitsCautiousV52
                | Self::HardPitsCautiousV53
                | Self::HardFarmExitPressureV58
                | Self::HardPitsExitPressureV58
                | Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
        )
    }

    pub(super) const fn pressure_mode(self) -> super::super::PressureMode {
        if matches!(
            self,
            Self::HardFarmExitPressureV58
                | Self::HardPitsExitPressureV58
                | Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
        ) {
            super::super::PressureMode::ExitCoverage
        } else if matches!(
            self,
            Self::HardV28 | Self::HardFarmV39 | Self::HardFarmPitsV53 | Self::HardPitsCautiousV53
        ) {
            super::super::PressureMode::Baseline
        } else {
            super::super::PressureMode::Disabled
        }
    }

    pub(super) const fn farm_origin_recovery(self) -> bool {
        matches!(
            self,
            Self::FarmV38
                | Self::HardFarmV39
                | Self::FarmPitsV52
                | Self::HardFarmPitsV53
                | Self::HardFarmExitPressureV58
                | Self::HardFarmYieldV59
        )
    }

    pub(super) const fn avoid_pits(self) -> bool {
        matches!(
            self,
            Self::FarmPitsV52
                | Self::HardFarmPitsV53
                | Self::SpikePitsCautiousV52
                | Self::HardPitsCautiousV53
                | Self::HardFarmExitPressureV58
                | Self::HardPitsExitPressureV58
                | Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
        )
    }

    pub(super) const fn farm_origin_mode(self) -> super::super::FarmOriginMode {
        if matches!(self, Self::HardFarmYieldV59 | Self::HardPitsYieldV59) {
            super::super::FarmOriginMode::VisibleYield
        } else {
            super::super::FarmOriginMode::Nearest
        }
    }
}
