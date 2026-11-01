//! Controller identity is independent of seat occupancy and network connections.
//!
//! These are internal domain contracts, not additions to the live room protocol.

use crate::game::SeatId;

/// A policy identity changes when decision semantics change, independently of game rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuPolicy {
    UtilityV1,
    UtilityV2,
    UtilityV3,
    UtilityV4,
    UtilityV5,
    UtilityV6,
    UtilityV7,
    UtilityV8,
    UtilityV9,
    UtilityV10,
    UtilityV11,
    UtilityV12,
    UtilityV13,
    UtilityV14,
    UtilityV15,
    UtilityV16,
    UtilityV17,
    UtilityV18,
    UtilityV19,
    UtilityV20,
    UtilityV21,
    UtilityV22,
    UtilityV23,
    UtilityV24,
    UtilityV25,
    UtilityV26,
    UtilityV27,
    UtilityV28,
}

impl CpuPolicy {
    #[must_use]
    pub const fn version(self) -> u16 {
        match self {
            Self::UtilityV1 => 1,
            Self::UtilityV2 => 2,
            Self::UtilityV3 => 3,
            Self::UtilityV4 => 4,
            Self::UtilityV5 => 5,
            Self::UtilityV6 => 6,
            Self::UtilityV7 => 7,
            Self::UtilityV8 => 8,
            Self::UtilityV9 => 9,
            Self::UtilityV10 => 10,
            Self::UtilityV11 => 11,
            Self::UtilityV12 => 12,
            Self::UtilityV13 => 13,
            Self::UtilityV14 => 14,
            Self::UtilityV15 => 15,
            Self::UtilityV16 => 16,
            Self::UtilityV17 => 17,
            Self::UtilityV18 => 18,
            Self::UtilityV19 => 19,
            Self::UtilityV20 => 20,
            Self::UtilityV21 => 21,
            Self::UtilityV22 => 22,
            Self::UtilityV23 => 23,
            Self::UtilityV24 => 24,
            Self::UtilityV25 => 25,
            Self::UtilityV26 => 26,
            Self::UtilityV27 => 27,
            Self::UtilityV28 => 28,
        }
    }
}

/// Initial profile identity; tuning and additional levels require measured playtesting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDifficulty {
    Easy,
    Standard,
    Hard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuProfile {
    pub policy: CpuPolicy,
    pub difficulty: CpuDifficulty,
}

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

    /// Only the two current legacy profiles may be selected by the archived mode.
    #[must_use]
    pub const fn is_current(self) -> bool {
        matches!(self, Self::CURRENT | Self::CURRENT_EASY)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerKind {
    Human,
    Cpu(CpuProfile),
}

/// A mode determines controller kinds; it cannot assign a CPU to the creator seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomMode {
    HumanDuel,
    Solo(CpuProfile),
}

impl RoomMode {
    #[must_use]
    pub const fn controller(self, seat: SeatId) -> Option<ControllerKind> {
        match (self, seat) {
            (Self::HumanDuel, SeatId::One | SeatId::Two) | (Self::Solo(_), SeatId::One) => {
                Some(ControllerKind::Human)
            }
            (Self::Solo(profile), SeatId::Two) => Some(ControllerKind::Cpu(profile)),
            (_, SeatId::Three | SeatId::Four) => None,
        }
    }
}

#[cfg(test)]
#[path = "participants_tests.rs"]
mod tests;
