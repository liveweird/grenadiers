use crate::game::{EnabledItems, layout_clearance::apply_lair_clearance};
use crate::room::BombCapacityDropMultiplier;

use super::{
    ArenaGenerationConfig, GeneratedArena, GenerationError, generate_weighted_multiplayer_arena,
};

pub const WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION: u16 = 7;

pub fn generate_weighted_lair_clearance_multiplayer_arena(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
    multiplier: BombCapacityDropMultiplier,
    lairs_enabled: bool,
) -> Result<GeneratedArena, GenerationError> {
    let mut generated = generate_weighted_multiplayer_arena(seed, enabled, config, multiplier)?;
    if lairs_enabled {
        apply_lair_clearance(&mut generated);
    }
    generated.record.version = WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION;
    Ok(generated)
}

#[cfg(test)]
#[path = "generated_v7_tests.rs"]
mod tests;
