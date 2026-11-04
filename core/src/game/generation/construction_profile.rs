use crate::game::SpikePitLayoutPolicy;
use crate::room::{BoardSize, BombCapacityDropMultiplier};

use super::{
    ArenaGenerationConfig, GeneratedArena, GenerationError, LAIR_CLEARANCE_GENERATOR_VERSION,
    MULTIPLAYER_GENERATOR_VERSION, VARIABLE_GENERATOR_VERSION, WEIGHTED_GENERATOR_VERSION,
    WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION, generate_lair_clearance_multiplayer_arena,
    generate_multiplayer_arena, generate_variable_multiplayer_arena,
    generate_weighted_lair_clearance_multiplayer_arena, generate_weighted_multiplayer_arena,
};
use crate::game::EnabledItems;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArenaConstructionProfile {
    ReflectedV3V4,
    LairClearanceV5,
    WeightedV6,
    WeightedLairClearanceV7,
}

impl ArenaConstructionProfile {
    pub fn generate(
        self,
        seed: [u8; 32],
        enabled: EnabledItems,
        config: ArenaGenerationConfig,
        multiplier: BombCapacityDropMultiplier,
        lairs_enabled: bool,
    ) -> Result<GeneratedArena, GenerationError> {
        match self {
            Self::ReflectedV3V4 => match config.board {
                BoardSize::FifteenByThirteen => generate_multiplayer_arena(seed, enabled, config),
                BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => {
                    generate_variable_multiplayer_arena(seed, enabled, config)
                }
            },
            Self::LairClearanceV5 => {
                generate_lair_clearance_multiplayer_arena(seed, enabled, config, lairs_enabled)
            }
            Self::WeightedV6 => {
                generate_weighted_multiplayer_arena(seed, enabled, config, multiplier)
            }
            Self::WeightedLairClearanceV7 => generate_weighted_lair_clearance_multiplayer_arena(
                seed,
                enabled,
                config,
                multiplier,
                lairs_enabled,
            ),
        }
    }

    #[must_use]
    pub const fn generator_version(self, board: BoardSize) -> u16 {
        match self {
            Self::ReflectedV3V4 => match board {
                BoardSize::FifteenByThirteen => MULTIPLAYER_GENERATOR_VERSION,
                BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => VARIABLE_GENERATOR_VERSION,
            },
            Self::LairClearanceV5 => LAIR_CLEARANCE_GENERATOR_VERSION,
            Self::WeightedV6 => WEIGHTED_GENERATOR_VERSION,
            Self::WeightedLairClearanceV7 => WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION,
        }
    }

    #[must_use]
    pub const fn spike_pit_layout_policy(self) -> SpikePitLayoutPolicy {
        match self {
            Self::ReflectedV3V4 | Self::WeightedV6 => SpikePitLayoutPolicy::LegacyV1,
            Self::LairClearanceV5 | Self::WeightedLairClearanceV7 => {
                SpikePitLayoutPolicy::LairClearanceV2
            }
        }
    }
}
