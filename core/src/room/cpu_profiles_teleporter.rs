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
    /// Equal-distance visible opponent exit coverage, retaining corner refuges.
    pub const HARD_V58: Self = Self::profile(CpuPolicy::UtilityV58, CpuDifficulty::Hard);
    /// Farm-origin yield ranking, retaining the Hard58 recovery and pressure stack.
    pub const HARD_V59: Self = Self::profile(CpuPolicy::UtilityV59, CpuDifficulty::Hard);

    /// Contact-punch aware live profiles, retaining the archived v56/v59 strategy stack.
    pub const CONTACT_PUNCH_V60_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV60, CpuDifficulty::Standard);
    pub const CONTACT_PUNCH_V60_EASY: Self =
        Self::profile(CpuPolicy::UtilityV60, CpuDifficulty::Easy);
    pub const HARD_V61: Self = Self::profile(CpuPolicy::UtilityV61, CpuDifficulty::Hard);

    /// Crossing-aware origin continuation, retaining the v60/v61 ordinary planners.
    pub const RECOVERY_V62_STANDARD: Self =
        Self::profile(CpuPolicy::UtilityV62, CpuDifficulty::Standard);
    pub const RECOVERY_V62_EASY: Self = Self::profile(CpuPolicy::UtilityV62, CpuDifficulty::Easy);
    pub const HARD_V63: Self = Self::profile(CpuPolicy::UtilityV63, CpuDifficulty::Hard);
    /// Unshipped hard prototype: certified opportunistic visible-block demolition.
    pub const HARD_V64: Self = Self::profile(CpuPolicy::UtilityV64, CpuDifficulty::Hard);
    /// Measured-risk reactive foreign-bomb demolition; Hard-only and opt-in.
    pub const HARD_V65: Self = Self::profile(CpuPolicy::UtilityV65, CpuDifficulty::Hard);
    /// Fresh equal-cost pickup contention, retaining the complete Hard65 tactic stack.
    pub const HARD_V66: Self = Self::profile(CpuPolicy::UtilityV66, CpuDifficulty::Hard);
    /// One handoff-authorized useful owned-bomb combination, retaining Hard66 ordinary play.
    pub const HARD_V67: Self = Self::profile(CpuPolicy::UtilityV67, CpuDifficulty::Hard);
    /// Easy62 ordinary decisions with certified post-navigation cadence guidance.
    pub const CERTIFIED_CADENCE_V68_EASY: Self =
        Self::profile(CpuPolicy::UtilityV68, CpuDifficulty::Easy);
    /// Hard67 tactics with actor-motion-only strategic progress renewal.
    pub const HARD_V69: Self = Self::profile(CpuPolicy::UtilityV69, CpuDifficulty::Hard);

    pub const HARD_V57: Self = Self::profile(CpuPolicy::UtilityV57, CpuDifficulty::Hard);
}
