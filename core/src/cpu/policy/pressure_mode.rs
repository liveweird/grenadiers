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
pub(crate) struct GoalModes {
    pub(crate) pressure: PressureMode,
    pub(crate) farm_origin: FarmOriginMode,
}

impl GoalModes {
    pub(crate) const fn nearest(pressure: PressureMode) -> Self {
        Self {
            pressure,
            farm_origin: FarmOriginMode::Nearest,
        }
    }

    pub(crate) const fn visible_yield() -> Self {
        Self {
            pressure: PressureMode::ExitCoverage,
            farm_origin: FarmOriginMode::VisibleYield,
        }
    }
}
