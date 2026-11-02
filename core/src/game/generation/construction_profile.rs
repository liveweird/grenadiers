use crate::game::SpikePitLayoutPolicy;
use crate::room::BoardSize;

use super::{
    LAIR_CLEARANCE_GENERATOR_VERSION, MULTIPLAYER_GENERATOR_VERSION, VARIABLE_GENERATOR_VERSION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArenaConstructionProfile {
    ReflectedV3V4,
    LairClearanceV5,
}

impl ArenaConstructionProfile {
    #[must_use]
    pub const fn generator_version(self, board: BoardSize) -> u16 {
        match self {
            Self::ReflectedV3V4 => match board {
                BoardSize::FifteenByThirteen => MULTIPLAYER_GENERATOR_VERSION,
                BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => VARIABLE_GENERATOR_VERSION,
            },
            Self::LairClearanceV5 => LAIR_CLEARANCE_GENERATOR_VERSION,
        }
    }

    #[must_use]
    pub const fn spike_pit_layout_policy(self) -> SpikePitLayoutPolicy {
        match self {
            Self::ReflectedV3V4 => SpikePitLayoutPolicy::LegacyV1,
            Self::LairClearanceV5 => SpikePitLayoutPolicy::LairClearanceV2,
        }
    }
}
