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
