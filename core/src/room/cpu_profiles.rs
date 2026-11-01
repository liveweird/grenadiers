use super::{CpuDifficulty, CpuPolicy, CpuProfile};

impl CpuProfile {
    /// Original policy identity, retained for archived projections and replays.
    pub const INITIAL: Self = Self {
        policy: CpuPolicy::UtilityV1,
        difficulty: CpuDifficulty::Standard,
    };

    /// First pulse policy, retained for archived projections and replays.
    pub const PULSE_V2: Self = Self {
        policy: CpuPolicy::UtilityV2,
        difficulty: CpuDifficulty::Standard,
    };

    /// First committed-route policy, retained for archived projections and replays.
    pub const ROUTE_V3: Self = Self {
        policy: CpuPolicy::UtilityV3,
        difficulty: CpuDifficulty::Standard,
    };

    /// First certified planting policy, retained for archived projections and replays.
    pub const PLANT_V4: Self = Self {
        policy: CpuPolicy::UtilityV4,
        difficulty: CpuDifficulty::Standard,
    };

    /// First scored-navigation policy, retained for archived projections and replays.
    pub const GOALS_V5: Self = Self {
        policy: CpuPolicy::UtilityV5,
        difficulty: CpuDifficulty::Standard,
    };

    /// First timed-weapon policy, retained for archived projections and replays.
    pub const WEAPONS_V6: Self = Self {
        policy: CpuPolicy::UtilityV6,
        difficulty: CpuDifficulty::Standard,
    };

    /// First known-block phasing policy, retained for archived projections and replays.
    pub const WALL_PASS_V7: Self = Self {
        policy: CpuPolicy::UtilityV7,
        difficulty: CpuDifficulty::Standard,
    };

    /// First finite-hazard responsiveness policy, retained for archived projections and replays.
    pub const RESPONSIVE_V8: Self = Self {
        policy: CpuPolicy::UtilityV8,
        difficulty: CpuDifficulty::Standard,
    };

    /// First confirmed-origin recovery policy, retained for archived projections and replays.
    pub const ORIGIN_V9: Self = Self {
        policy: CpuPolicy::UtilityV9,
        difficulty: CpuDifficulty::Standard,
    };

    /// First observed-placement and guided-origin policy, retained for replays.
    pub const GUIDED_V10: Self = Self {
        policy: CpuPolicy::UtilityV10,
        difficulty: CpuDifficulty::Standard,
    };

    /// First clear-gap route-completion policy, retained for archived replays.
    pub const CLEAR_GAP_V11: Self = Self {
        policy: CpuPolicy::UtilityV11,
        difficulty: CpuDifficulty::Standard,
    };

    /// First ordinary turn-alignment policy, retained for archived replays.
    pub const STEERING_V12: Self = Self {
        policy: CpuPolicy::UtilityV12,
        difficulty: CpuDifficulty::Standard,
    };

    /// First guarded Bracelet tactic, retained for archived replays.
    pub const BRACELET_V13: Self = Self {
        policy: CpuPolicy::UtilityV13,
        difficulty: CpuDifficulty::Standard,
    };

    /// First guarded Punch tactic, retained for archived replays.
    pub const PUNCH_V14: Self = Self {
        policy: CpuPolicy::UtilityV14,
        difficulty: CpuDifficulty::Standard,
    };

    /// First equipment-goal policy, retained for archived replays.
    pub const EQUIPMENT_GOALS_V15: Self = Self {
        policy: CpuPolicy::UtilityV15,
        difficulty: CpuDifficulty::Standard,
    };

    /// Standard profile used to create a new live CPU controller.
    pub const CURRENT: Self = Self {
        policy: CpuPolicy::UtilityV16,
        difficulty: CpuDifficulty::Standard,
    };

    /// Easy profile used to create a new live CPU controller.
    pub const CURRENT_EASY: Self = Self {
        policy: CpuPolicy::UtilityV16,
        difficulty: CpuDifficulty::Easy,
    };

    /// Multiplayer Standard profile. Legacy current profiles remain replay-stable v16.
    pub const MULTIPLAYER_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV17,
        difficulty: CpuDifficulty::Standard,
    };

    /// Multiplayer Easy profile. Legacy current profiles remain replay-stable v16.
    pub const MULTIPLAYER_EASY: Self = Self {
        policy: CpuPolicy::UtilityV17,
        difficulty: CpuDifficulty::Easy,
    };

    /// Rules-v7 Standard profile, with either disabled or single-cloud environment.
    pub const ENVIRONMENT_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV18,
        difficulty: CpuDifficulty::Standard,
    };

    /// Rules-v7 Easy profile, with either disabled or single-cloud environment.
    pub const ENVIRONMENT_EASY: Self = Self {
        policy: CpuPolicy::UtilityV18,
        difficulty: CpuDifficulty::Easy,
    };

    /// Original rules-v8 Standard identity, retained for archived records.
    pub const LAIRS_V19_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV19,
        difficulty: CpuDifficulty::Standard,
    };

    /// Original rules-v8 Easy identity, retained for archived records.
    pub const LAIRS_V19_EASY: Self = Self {
        policy: CpuPolicy::UtilityV19,
        difficulty: CpuDifficulty::Easy,
    };

    /// Rules-v8 Standard profile with contact-certified interrupted-plant recovery.
    pub const LAIRS_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV20,
        difficulty: CpuDifficulty::Standard,
    };

    /// Rules-v8 Easy profile with contact-certified interrupted-plant recovery.
    pub const LAIRS_EASY: Self = Self {
        policy: CpuPolicy::UtilityV20,
        difficulty: CpuDifficulty::Easy,
    };

    /// Archived rules-v9 profile before certified pickup handoffs.
    pub const RESPAWN_V21_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV21,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RESPAWN_V21_EASY: Self = Self {
        policy: CpuPolicy::UtilityV21,
        difficulty: CpuDifficulty::Easy,
    };

    /// Archived rules-v9 profile with certified pickup handoffs.
    pub const RESPAWN_V22_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV22,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RESPAWN_V22_EASY: Self = Self {
        policy: CpuPolicy::UtilityV22,
        difficulty: CpuDifficulty::Easy,
    };

    /// Archived rules-v9 profile with goal-cell persistence and certified planting.
    pub const RESPAWN_V23_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV23,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RESPAWN_V23_EASY: Self = Self {
        policy: CpuPolicy::UtilityV23,
        difficulty: CpuDifficulty::Easy,
    };

    /// Archived rules-v9 identity with persistent subjects and phased strategic intent.
    pub const RESPAWN_V24_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV24,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RESPAWN_V24_EASY: Self = Self {
        policy: CpuPolicy::UtilityV24,
        difficulty: CpuDifficulty::Easy,
    };

    /// Rules-v9 identity with certified repositioning after unavailable planting.
    pub const RESPAWN_LAIRS_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV25,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RESPAWN_LAIRS_EASY: Self = Self {
        policy: CpuPolicy::UtilityV25,
        difficulty: CpuDifficulty::Easy,
    };

    /// Dormant rules-v10 profile with spike-pit-aware certified execution.
    pub const SPIKE_PITS_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV26,
        difficulty: CpuDifficulty::Standard,
    };

    /// Dormant Easy rules-v10 profile; live room selection remains on CPU25.
    pub const SPIKE_PITS_EASY: Self = Self {
        policy: CpuPolicy::UtilityV26,
        difficulty: CpuDifficulty::Easy,
    };

    /// Rules-v10 Easy/Standard reliability profile; archived constants remain unchanged.
    pub const RELIABLE_V27_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV27,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RELIABLE_V27_EASY: Self = Self {
        policy: CpuPolicy::UtilityV27,
        difficulty: CpuDifficulty::Easy,
    };

    /// Rules-v10 Hard tactics use the same public observation and safety proofs.
    pub const HARD_V28: Self = Self {
        policy: CpuPolicy::UtilityV28,
        difficulty: CpuDifficulty::Hard,
    };

    /// Hard counter-pressure with the unchanged certified route model.
    pub const HARD_V29: Self = Self {
        policy: CpuPolicy::UtilityV29,
        difficulty: CpuDifficulty::Hard,
    };

    /// Rules-v10 one-update pickup handoffs with retained Standard tactics.
    pub const PICKUP_V31_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV31,
        difficulty: CpuDifficulty::Standard,
    };

    pub const PICKUP_V31_EASY: Self = Self {
        policy: CpuPolicy::UtilityV31,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard counter-pressure and one-update pickup progress.
    pub const HARD_V32: Self = Self {
        policy: CpuPolicy::UtilityV32,
        difficulty: CpuDifficulty::Hard,
    };

    /// Pickup progress plus full-footprint refuge centering.
    pub const REFUGE_V33_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV33,
        difficulty: CpuDifficulty::Standard,
    };

    pub const REFUGE_V33_EASY: Self = Self {
        policy: CpuPolicy::UtilityV33,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard tactics plus full-footprint refuge centering.
    pub const HARD_V34: Self = Self {
        policy: CpuPolicy::UtilityV34,
        difficulty: CpuDifficulty::Hard,
    };

    /// Hard refuge guidance with certified terminal-hold replacement.
    pub const HARD_V35: Self = Self {
        policy: CpuPolicy::UtilityV35,
        difficulty: CpuDifficulty::Hard,
    };

    /// Pickup progress with certified farming refuges.
    pub const FARM_REFUGES_V36_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV36,
        difficulty: CpuDifficulty::Standard,
    };

    pub const FARM_REFUGES_V36_EASY: Self = Self {
        policy: CpuPolicy::UtilityV36,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard farming refuges with terminal-hold replacement retained.
    pub const HARD_V37: Self = Self {
        policy: CpuPolicy::UtilityV37,
        difficulty: CpuDifficulty::Hard,
    };

    /// Farming refuges with bounded failed-approach memory.
    pub const FARM_ORIGINS_V38_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV38,
        difficulty: CpuDifficulty::Standard,
    };

    pub const FARM_ORIGINS_V38_EASY: Self = Self {
        policy: CpuPolicy::UtilityV38,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard failed-approach memory with terminal-hold replacement retained.
    pub const HARD_V39: Self = Self {
        policy: CpuPolicy::UtilityV39,
        difficulty: CpuDifficulty::Hard,
    };

    /// Farming with plant-bound contact-topology authority.
    pub const CONTACT_TOPOLOGY_V40_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV40,
        difficulty: CpuDifficulty::Standard,
    };

    pub const CONTACT_TOPOLOGY_V40_EASY: Self = Self {
        policy: CpuPolicy::UtilityV40,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard contact topology with terminal-hold replacement retained.
    pub const HARD_V41: Self = Self {
        policy: CpuPolicy::UtilityV41,
        difficulty: CpuDifficulty::Hard,
    };

    /// Contact topology that can prove every possible birth remains blocked.
    pub const BLOCKED_BIRTHS_V42_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV42,
        difficulty: CpuDifficulty::Standard,
    };

    pub const BLOCKED_BIRTHS_V42_EASY: Self = Self {
        policy: CpuPolicy::UtilityV42,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard blocked-birth topology with terminal-hold replacement retained.
    pub const HARD_V43: Self = Self {
        policy: CpuPolicy::UtilityV43,
        difficulty: CpuDifficulty::Hard,
    };

    /// Blocked-birth farming with opt-in certified origin continuation.
    pub const CONTINUATION_V44_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV44,
        difficulty: CpuDifficulty::Standard,
    };

    pub const CONTINUATION_V44_EASY: Self = Self {
        policy: CpuPolicy::UtilityV44,
        difficulty: CpuDifficulty::Easy,
    };

    /// Hard blocked-birth farming with opt-in certified origin continuation.
    pub const HARD_V45: Self = Self {
        policy: CpuPolicy::UtilityV45,
        difficulty: CpuDifficulty::Hard,
    };

    /// Only the two current legacy profiles may be selected by the archived mode.
    #[must_use]
    pub const fn is_current(self) -> bool {
        matches!(self, Self::CURRENT | Self::CURRENT_EASY)
    }
}

#[path = "cpu_profiles_teleporter.rs"]
mod teleporter;
