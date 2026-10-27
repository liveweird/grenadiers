//! Dormant multiplayer room configuration contracts.
//!
//! This module does not activate larger rooms or add a wire representation.

use crate::game::{PlayerCount, SeatId};

use super::{CpuDifficulty, RoomMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeatControl {
    Human,
    Cpu,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoardSize {
    ElevenByNine,
    ThirteenByEleven,
    #[default]
    FifteenByThirteen,
}

impl BoardSize {
    #[must_use]
    pub const fn dimensions(self) -> (u16, u16) {
        match self {
            Self::ElevenByNine => (11, 9),
            Self::ThirteenByEleven => (13, 11),
            Self::FifteenByThirteen => (15, 13),
        }
    }
}

impl TryFrom<(u16, u16)> for BoardSize {
    type Error = ();

    fn try_from(dimensions: (u16, u16)) -> Result<Self, Self::Error> {
        match dimensions {
            (11, 9) => Ok(Self::ElevenByNine),
            (13, 11) => Ok(Self::ThirteenByEleven),
            (15, 13) => Ok(Self::FifteenByThirteen),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomConfigurationError {
    InvalidPlayerCount { actual: usize },
    CreatorMustBeHuman,
    MissingCpuDifficulty,
    UnexpectedCpuDifficulty,
}

impl std::fmt::Display for RoomConfigurationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for RoomConfigurationError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomConfiguration {
    player_count: PlayerCount,
    controls: [SeatControl; 4],
    difficulty: Option<CpuDifficulty>,
    board: BoardSize,
}

impl RoomConfiguration {
    pub fn new(
        controls: &[SeatControl],
        difficulty: Option<CpuDifficulty>,
        board: BoardSize,
    ) -> Result<Self, RoomConfigurationError> {
        let player_count = PlayerCount::try_from(controls.len()).map_err(|()| {
            RoomConfigurationError::InvalidPlayerCount {
                actual: controls.len(),
            }
        })?;
        if controls[0] != SeatControl::Human {
            return Err(RoomConfigurationError::CreatorMustBeHuman);
        }

        let has_cpu = controls.contains(&SeatControl::Cpu);
        match (has_cpu, difficulty) {
            (true, None) => return Err(RoomConfigurationError::MissingCpuDifficulty),
            (false, Some(_)) => return Err(RoomConfigurationError::UnexpectedCpuDifficulty),
            _ => {}
        }

        let mut canonical_controls = [SeatControl::Human; 4];
        canonical_controls[..controls.len()].copy_from_slice(controls);
        Ok(Self {
            player_count,
            controls: canonical_controls,
            difficulty,
            board,
        })
    }

    #[must_use]
    pub const fn human_duel() -> Self {
        Self {
            player_count: PlayerCount::Two,
            controls: [SeatControl::Human; 4],
            difficulty: None,
            board: BoardSize::FifteenByThirteen,
        }
    }

    #[must_use]
    pub const fn solo(difficulty: CpuDifficulty) -> Self {
        Self {
            player_count: PlayerCount::Two,
            controls: [
                SeatControl::Human,
                SeatControl::Cpu,
                SeatControl::Human,
                SeatControl::Human,
            ],
            difficulty: Some(difficulty),
            board: BoardSize::FifteenByThirteen,
        }
    }

    #[must_use]
    pub const fn from_legacy(mode: RoomMode) -> Self {
        match mode {
            RoomMode::HumanDuel => Self::human_duel(),
            RoomMode::Solo(profile) => Self::solo(profile.difficulty),
        }
    }

    #[must_use]
    pub const fn player_count(self) -> PlayerCount {
        self.player_count
    }

    #[must_use]
    pub fn controllers(&self) -> &[SeatControl] {
        &self.controls[..self.player_count.get()]
    }

    #[must_use]
    pub const fn controller(self, seat: SeatId) -> Option<SeatControl> {
        if self.player_count.contains(seat) {
            Some(self.controls[seat.index()])
        } else {
            None
        }
    }

    #[must_use]
    pub const fn cpu_difficulty(self) -> Option<CpuDifficulty> {
        self.difficulty
    }

    #[must_use]
    pub const fn board(self) -> BoardSize {
        self.board
    }

    pub fn human_seats(&self) -> impl Iterator<Item = SeatId> + '_ {
        self.player_count
            .seats()
            .iter()
            .copied()
            .filter(|&seat| self.controls[seat.index()] == SeatControl::Human)
    }

    pub fn cpu_seats(&self) -> impl Iterator<Item = SeatId> + '_ {
        self.player_count
            .seats()
            .iter()
            .copied()
            .filter(|&seat| self.controls[seat.index()] == SeatControl::Cpu)
    }
}

#[cfg(test)]
#[path = "configuration_tests.rs"]
mod tests;
