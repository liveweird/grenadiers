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

    /// Pit-avoiding Standard policy, preserving the v48 executor and v50 selector.
    pub const PIT_AVOIDANCE_V52_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV52, CpuDifficulty::Standard);
    pub const PIT_AVOIDANCE_V52_EASY: Self =
        Self::profile(CpuPolicy::UtilityV52, CpuDifficulty::Easy);
    pub const HARD_V53: Self = Self::profile(CpuPolicy::UtilityV53, CpuDifficulty::Hard);

    /// Common recovery preparation before bounded optional continuation.
    pub const RECOVERY_V54_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV54, CpuDifficulty::Standard);
    pub const RECOVERY_V54_EASY: Self = Self::profile(CpuPolicy::UtilityV54, CpuDifficulty::Easy);
    pub const HARD_V55: Self = Self::profile(CpuPolicy::UtilityV55, CpuDifficulty::Hard);

    /// Lateral refuge hints when an ordinary farming escape points at a hard wall.
    pub const CORNER_REFUGES_V56_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV56, CpuDifficulty::Standard);
    pub const CORNER_REFUGES_V56_EASY: Self =
        Self::profile(CpuPolicy::UtilityV56, CpuDifficulty::Easy);
    pub const HARD_V57: Self = Self::profile(CpuPolicy::UtilityV57, CpuDifficulty::Hard);
}
