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
        }
    }
}

/// Initial profile identity; tuning and additional levels require measured playtesting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDifficulty {
    Easy,
    Standard,
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

    /// Rules-v9 profile with persistent subjects and phased strategic intent.
    pub const RESPAWN_LAIRS_STANDARD: Self = Self {
        policy: CpuPolicy::UtilityV24,
        difficulty: CpuDifficulty::Standard,
    };

    pub const RESPAWN_LAIRS_EASY: Self = Self {
        policy: CpuPolicy::UtilityV24,
        difficulty: CpuDifficulty::Easy,
    };

    /// Only the two current profiles may be selected for a new live controller.
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
mod tests {
    use super::*;

    #[test]
    fn human_duel_has_two_human_controllers() {
        for seat in SeatId::DUEL {
            assert_eq!(
                RoomMode::HumanDuel.controller(seat),
                Some(ControllerKind::Human)
            );
        }
        assert_eq!(RoomMode::HumanDuel.controller(SeatId::Three), None);
        assert_eq!(RoomMode::HumanDuel.controller(SeatId::Four), None);
    }

    #[test]
    fn solo_keeps_creator_human_and_cpu_profile_on_second_seat() {
        let mode = RoomMode::Solo(CpuProfile::INITIAL);
        assert_eq!(mode.controller(SeatId::One), Some(ControllerKind::Human));
        assert_eq!(
            mode.controller(SeatId::Two),
            Some(ControllerKind::Cpu(CpuProfile {
                policy: CpuPolicy::UtilityV1,
                difficulty: CpuDifficulty::Standard,
            }))
        );
    }

    #[test]
    fn policy_versions_are_independent_of_controller_assignment() {
        assert_eq!(CpuProfile::INITIAL.policy.version(), 1);
        assert_eq!(CpuProfile::PULSE_V2.policy.version(), 2);
        assert_eq!(CpuProfile::ROUTE_V3.policy.version(), 3);
        assert_eq!(CpuProfile::PLANT_V4.policy.version(), 4);
        assert_eq!(CpuProfile::GOALS_V5.policy.version(), 5);
        assert_eq!(CpuProfile::WEAPONS_V6.policy.version(), 6);
        assert_eq!(CpuProfile::WALL_PASS_V7.policy.version(), 7);
        assert_eq!(CpuProfile::RESPONSIVE_V8.policy.version(), 8);
        assert_eq!(CpuProfile::ORIGIN_V9.policy.version(), 9);
        assert_eq!(CpuProfile::GUIDED_V10.policy.version(), 10);
        assert_eq!(CpuProfile::CLEAR_GAP_V11.policy.version(), 11);
        assert_eq!(CpuProfile::STEERING_V12.policy.version(), 12);
        assert_eq!(CpuProfile::BRACELET_V13.policy.version(), 13);
        assert_eq!(CpuProfile::PUNCH_V14.policy.version(), 14);
        assert_eq!(CpuProfile::EQUIPMENT_GOALS_V15.policy.version(), 15);
        assert_eq!(CpuProfile::CURRENT.policy.version(), 16);
        assert_eq!(CpuProfile::CURRENT_EASY.policy.version(), 16);
        assert_eq!(CpuProfile::MULTIPLAYER_STANDARD.policy.version(), 17);
        assert_eq!(CpuProfile::MULTIPLAYER_EASY.policy.version(), 17);
        assert_eq!(CpuProfile::ENVIRONMENT_STANDARD.policy.version(), 18);
        assert_eq!(CpuProfile::ENVIRONMENT_EASY.policy.version(), 18);
        assert_eq!(CpuProfile::LAIRS_V19_STANDARD.policy.version(), 19);
        assert_eq!(CpuProfile::LAIRS_V19_EASY.policy.version(), 19);
        assert_eq!(CpuProfile::LAIRS_STANDARD.policy.version(), 20);
        assert_eq!(CpuProfile::LAIRS_EASY.policy.version(), 20);
        assert_eq!(CpuProfile::RESPAWN_LAIRS_STANDARD.policy.version(), 24);
        assert_eq!(CpuProfile::RESPAWN_V23_STANDARD.policy.version(), 23);
        assert_eq!(CpuProfile::RESPAWN_V22_STANDARD.policy.version(), 22);
        assert_eq!(CpuProfile::RESPAWN_V21_STANDARD.policy.version(), 21);
        assert_eq!(CpuProfile::RESPAWN_LAIRS_EASY.policy.version(), 24);
        assert_eq!(CpuProfile::RESPAWN_V23_EASY.policy.version(), 23);
        assert_eq!(CpuProfile::RESPAWN_V22_EASY.policy.version(), 22);
        assert_eq!(CpuProfile::RESPAWN_V21_EASY.policy.version(), 21);
        assert!(CpuProfile::CURRENT.is_current());
        assert!(CpuProfile::CURRENT_EASY.is_current());
        assert!(!CpuProfile::MULTIPLAYER_STANDARD.is_current());
        assert!(!CpuProfile::MULTIPLAYER_EASY.is_current());
        assert!(!CpuProfile::EQUIPMENT_GOALS_V15.is_current());
        assert!(
            !CpuProfile {
                policy: CpuPolicy::UtilityV15,
                difficulty: CpuDifficulty::Easy,
            }
            .is_current()
        );
        assert_eq!(
            RoomMode::Solo(CpuProfile::CURRENT).controller(SeatId::Two),
            Some(ControllerKind::Cpu(CpuProfile::CURRENT))
        );
    }
}
