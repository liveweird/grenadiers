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
    UtilityV29,
    UtilityV31,
    UtilityV32,
    UtilityV33,
    UtilityV34,
    UtilityV35,
    UtilityV36,
    UtilityV37,
    UtilityV38,
    UtilityV39,
    UtilityV40,
    UtilityV41,
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
            Self::UtilityV29 => 29,
            Self::UtilityV31 => 31,
            Self::UtilityV32 => 32,
            Self::UtilityV33 => 33,
            Self::UtilityV34 => 34,
            Self::UtilityV35 => 35,
            Self::UtilityV36 => 36,
            Self::UtilityV37 => 37,
            Self::UtilityV38 => 38,
            Self::UtilityV39 => 39,
            Self::UtilityV40 => 40,
            Self::UtilityV41 => 41,
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

#[path = "cpu_profiles.rs"]
mod cpu_profiles;

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
