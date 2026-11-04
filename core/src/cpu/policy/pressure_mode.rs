#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PressureMode {
    Disabled,
    Baseline,
    ExitCoverage,
}

impl PressureMode {
    pub(crate) const fn enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FarmOriginMode {
    Nearest,
    VisibleYield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PickupMode {
    Legacy,
    VisibleContention,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GoalModes {
    pub(crate) pressure: PressureMode,
    pub(crate) farm_origin: FarmOriginMode,
    pub(crate) pickup: PickupMode,
}

impl GoalModes {
    pub(crate) const fn nearest(pressure: PressureMode) -> Self {
        Self {
            pressure,
            farm_origin: FarmOriginMode::Nearest,
            pickup: PickupMode::Legacy,
        }
    }

    pub(crate) const fn visible_yield() -> Self {
        Self {
            pressure: PressureMode::ExitCoverage,
            farm_origin: FarmOriginMode::VisibleYield,
            pickup: PickupMode::Legacy,
        }
    }

    pub(crate) const fn visible_yield_contention() -> Self {
        Self {
            pickup: PickupMode::VisibleContention,
            ..Self::visible_yield()
        }
    }
}
