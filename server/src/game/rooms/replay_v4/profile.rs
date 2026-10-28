use grenadiers_core::room::BoardSize;

pub(super) const FORMAT_V4: u16 = 4;
pub(super) const FORMAT_V5: u16 = 5;
pub(super) const GENERATOR_V4: u16 = 3;
pub(super) const GENERATOR_V5: u16 = 4;

const _: () = assert!(GENERATOR_V4 == grenadiers_core::game::MULTIPLAYER_GENERATOR_VERSION);
const _: () = assert!(GENERATOR_V5 == grenadiers_core::game::VARIABLE_GENERATOR_VERSION);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayProfile {
    V4,
    V5,
}

impl ReplayProfile {
    pub(super) const fn format(self) -> u16 {
        match self {
            Self::V4 => FORMAT_V4,
            Self::V5 => FORMAT_V5,
        }
    }

    pub(super) const fn generator(self) -> u16 {
        match self {
            Self::V4 => GENERATOR_V4,
            Self::V5 => GENERATOR_V5,
        }
    }

    pub(super) const fn accepts_board(self, board: BoardSize) -> bool {
        match self {
            Self::V4 => matches!(board, BoardSize::FifteenByThirteen),
            Self::V5 => matches!(board, BoardSize::ElevenByNine | BoardSize::ThirteenByEleven),
        }
    }
}
