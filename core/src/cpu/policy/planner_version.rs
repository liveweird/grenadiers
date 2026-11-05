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
    HardPickupV66,
    HardProgressV69,
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
                | Self::HardPickupV66
                | Self::HardProgressV69
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
                | Self::HardPickupV66
                | Self::HardProgressV69
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
                | Self::HardPickupV66
                | Self::HardProgressV69
        )
    }

    pub(super) const fn pressure_mode(self) -> super::super::PressureMode {
        if matches!(
            self,
            Self::HardFarmExitPressureV58
                | Self::HardPitsExitPressureV58
                | Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
                | Self::HardPickupV66
                | Self::HardProgressV69
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
                | Self::HardPickupV66
                | Self::HardProgressV69
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
                | Self::HardPickupV66
                | Self::HardProgressV69
        )
    }

    pub(super) const fn farm_origin_mode(self) -> super::super::FarmOriginMode {
        if matches!(
            self,
            Self::HardFarmYieldV59
                | Self::HardPitsYieldV59
                | Self::HardPickupV66
                | Self::HardProgressV69
        ) {
            super::super::FarmOriginMode::VisibleYield
        } else {
            super::super::FarmOriginMode::Nearest
        }
    }

    pub(super) const fn pickup_mode(self) -> super::super::PickupMode {
        if matches!(self, Self::HardPickupV66 | Self::HardProgressV69) {
            super::super::PickupMode::VisibleContention
        } else {
            super::super::PickupMode::Legacy
        }
    }

    pub(super) const fn progress_mode(self) -> super::super::ProgressMode {
        if matches!(self, Self::HardProgressV69) {
            super::super::ProgressMode::ActorMovement
        } else {
            super::super::ProgressMode::Legacy
        }
    }
}

#[cfg(test)]
pub(in crate::cpu::policy) fn assert_hard69_inherits_hard66() {
    let old = PlannerVersion::HardPickupV66;
    let new = PlannerVersion::HardProgressV69;
    assert_eq!(new.follow_committed(), old.follow_committed());
    assert_eq!(new.persistent_goals(), old.persistent_goals());
    assert_eq!(new.strategic_goals(), old.strategic_goals());
    assert_eq!(new.failed_plant_recovery(), old.failed_plant_recovery());
    assert_eq!(new.pressure_mode(), old.pressure_mode());
    assert_eq!(new.farm_origin_recovery(), old.farm_origin_recovery());
    assert_eq!(new.avoid_pits(), old.avoid_pits());
    assert_eq!(new.farm_origin_mode(), old.farm_origin_mode());
    assert_eq!(new.pickup_mode(), old.pickup_mode());
    assert_ne!(new.progress_mode(), old.progress_mode());
}
