use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::TeleporterStep;

/// Inactive selector output. Only survival selector modules may construct it.
pub struct TeleporterStepPlan {
    pub(super) step: TeleporterStep,
    pub(super) next_memory: UtilityMemory,
}

impl TeleporterStepPlan {
    #[cfg(test)]
    pub(crate) const fn new_for_test(step: TeleporterStep, next_memory: UtilityMemory) -> Self {
        Self { step, next_memory }
    }
}
