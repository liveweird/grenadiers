use super::{CpuDifficulty, CpuPolicy, CpuProfile};

impl CpuProfile {
    /// Teleporter-handoff Standard profile for private replay23.
    pub const TELEPORTER_V46_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV46,
        difficulty: CpuDifficulty::Standard,
    };

    /// Teleporter-handoff Easy profile for private replay23.
    pub const TELEPORTER_V46_EASY: Self = Self {
        policy: CpuPolicy::UtilityV46,
        difficulty: CpuDifficulty::Easy,
    };

    /// Teleporter-handoff Hard profile for private replay23.
    pub const HARD_V47: Self = Self {
        policy: CpuPolicy::UtilityV47,
        difficulty: CpuDifficulty::Hard,
    };

    pub const TELEPORTER_V48_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV48, CpuDifficulty::Standard);
    pub const TELEPORTER_V48_EASY: Self = Self::profile(CpuPolicy::UtilityV48, CpuDifficulty::Easy);
    pub const HARD_V49: Self = Self::profile(CpuPolicy::UtilityV49, CpuDifficulty::Hard);
    const fn profile(policy: CpuPolicy, difficulty: CpuDifficulty) -> Self {
        Self { policy, difficulty }
    }

    /// Multi-Teleporter recovery, preserving the v48 physical executor.
    pub const TELEPORTER_V50_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV50, CpuDifficulty::Standard);
    pub const TELEPORTER_V50_EASY: Self = Self::profile(CpuPolicy::UtilityV50, CpuDifficulty::Easy);
    pub const HARD_V51: Self = Self::profile(CpuPolicy::UtilityV51, CpuDifficulty::Hard);
}
