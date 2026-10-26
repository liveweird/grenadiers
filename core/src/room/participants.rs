//! Controller identity is independent of seat occupancy and network connections.
//!
//! These are internal domain contracts, not additions to the live room protocol.

use crate::game::SeatId;

/// A policy identity changes when decision semantics change, independently of game rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuPolicy {
    UtilityV1,
    UtilityV2,
}

impl CpuPolicy {
    #[must_use]
    pub const fn version(self) -> u16 {
        match self {
            Self::UtilityV1 => 1,
            Self::UtilityV2 => 2,
        }
    }
}

/// Initial profile identity; tuning and additional levels require measured playtesting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuDifficulty {
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

    /// The only policy profile used to create a new live CPU controller.
    pub const CURRENT: Self = Self {
        policy: CpuPolicy::UtilityV2,
        difficulty: CpuDifficulty::Standard,
    };
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
    pub const fn controller(self, seat: SeatId) -> ControllerKind {
        match (self, seat) {
            (Self::HumanDuel, _) | (Self::Solo(_), SeatId::One) => ControllerKind::Human,
            (Self::Solo(profile), SeatId::Two) => ControllerKind::Cpu(profile),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_duel_has_two_human_controllers() {
        for seat in SeatId::ALL {
            assert_eq!(RoomMode::HumanDuel.controller(seat), ControllerKind::Human);
        }
    }

    #[test]
    fn solo_keeps_creator_human_and_cpu_profile_on_second_seat() {
        let mode = RoomMode::Solo(CpuProfile::INITIAL);
        assert_eq!(mode.controller(SeatId::One), ControllerKind::Human);
        assert_eq!(
            mode.controller(SeatId::Two),
            ControllerKind::Cpu(CpuProfile {
                policy: CpuPolicy::UtilityV1,
                difficulty: CpuDifficulty::Standard,
            })
        );
    }

    #[test]
    fn policy_versions_are_independent_of_controller_assignment() {
        assert_eq!(CpuProfile::INITIAL.policy.version(), 1);
        assert_eq!(CpuProfile::CURRENT.policy.version(), 2);
        assert_eq!(
            RoomMode::Solo(CpuProfile::CURRENT).controller(SeatId::Two),
            ControllerKind::Cpu(CpuProfile::CURRENT)
        );
    }
}
