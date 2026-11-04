use crate::game::{EnabledItems, layout_clearance::apply_lair_clearance};
use crate::room::BoardSize;

use super::{
    ArenaGenerationConfig, GeneratedArena, GenerationError, generate_multiplayer_arena,
    generate_variable_multiplayer_arena,
};

pub const LAIR_CLEARANCE_GENERATOR_VERSION: u16 = 5;

pub fn generate_lair_clearance_multiplayer_arena(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
    lairs_enabled: bool,
) -> Result<GeneratedArena, GenerationError> {
    let mut generated = match config.board {
        BoardSize::FifteenByThirteen => generate_multiplayer_arena(seed, enabled, config)?,
        BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => {
            generate_variable_multiplayer_arena(seed, enabled, config)?
        }
    };
    if lairs_enabled {
        apply_lair_clearance(&mut generated);
    }
    generated.record.version = LAIR_CLEARANCE_GENERATOR_VERSION;
    Ok(generated)
}

#[cfg(test)]
#[path = "generated_v5_tests.rs"]
mod tests;
