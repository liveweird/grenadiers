use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::arena::{ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use super::enabled_items::EnabledItems;
use super::fixtures::{classic_spawns, classic_terrain};
use super::oil::{generated_pair_fits, ordinary_pickup_count};
use super::pickups::PickupKind;
use super::types::CellId;
use random_words::{ChaChaWords, RandomWords};
mod generated_v3;
mod random_words;
mod sampling;
pub use generated_v3::{
    ArenaGenerationConfig, GenerationError, MULTIPLAYER_GENERATOR_VERSION,
    generate_multiplayer_arena,
};
use sampling::{sample_percent, sample_pickup};

pub const GENERATOR_VERSION: u16 = 1;
pub const EXPANDED_GENERATOR_VERSION: u16 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GenerationRecord {
    version: u16,
    seed: [u8; 32],
    fallback_used: bool,
}

impl GenerationRecord {
    #[must_use]
    pub const fn version(self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn seed(self) -> [u8; 32] {
        self.seed
    }

    #[must_use]
    pub const fn fallback_used(self) -> bool {
        self.fallback_used
    }
}

impl fmt::Debug for GenerationRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GenerationRecord")
            .field("version", &self.version)
            .field("seed", &"<redacted>")
            .field("fallback_used", &self.fallback_used)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GeneratedArena {
    arena: Arena,
    concealed_pickups: BTreeMap<CellId, PickupKind>,
    record: GenerationRecord,
}

impl GeneratedArena {
    #[must_use]
    pub const fn arena(&self) -> &Arena {
        &self.arena
    }

    #[must_use]
    pub const fn concealed_pickups(&self) -> &BTreeMap<CellId, PickupKind> {
        &self.concealed_pickups
    }

    #[must_use]
    pub const fn record(&self) -> GenerationRecord {
        self.record
    }

    #[must_use]
    pub fn into_parts(self) -> (Arena, BTreeMap<CellId, PickupKind>, GenerationRecord) {
        (self.arena, self.concealed_pickups, self.record)
    }
}

impl fmt::Debug for GeneratedArena {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ordinary_count = ordinary_pickup_count(&self.concealed_pickups);
        formatter
            .debug_struct("GeneratedArena")
            .field("arena", &self.arena)
            .field("concealed_pickup_count", &ordinary_count)
            .field("record", &self.record)
            .finish()
    }
}

#[must_use]
pub fn generate_arena(seed: [u8; 32]) -> GeneratedArena {
    generate_with_source(seed, &mut ChaChaWords::seeded(seed), classic_terrain())
}

#[must_use]
pub fn generate_expanded_arena(seed: [u8; 32], enabled: EnabledItems) -> GeneratedArena {
    generate_expanded_with_source(
        seed,
        &mut ChaChaWords::seeded(seed),
        classic_terrain(),
        enabled,
    )
}

fn generate_with_source(
    seed: [u8; 32],
    source: &mut impl RandomWords,
    terrain: [Terrain; super::arena::ARENA_CELL_COUNT],
) -> GeneratedArena {
    generate_versioned(
        seed,
        source,
        terrain,
        EnabledItems::EMPTY,
        GENERATOR_VERSION,
    )
}

fn generate_expanded_with_source(
    seed: [u8; 32],
    source: &mut impl RandomWords,
    terrain: [Terrain; super::arena::ARENA_CELL_COUNT],
    enabled: EnabledItems,
) -> GeneratedArena {
    generate_versioned(seed, source, terrain, enabled, EXPANDED_GENERATOR_VERSION)
}

fn generate_versioned(
    seed: [u8; 32],
    source: &mut impl RandomWords,
    terrain: [Terrain; super::arena::ARENA_CELL_COUNT],
    enabled: EnabledItems,
    version: u16,
) -> GeneratedArena {
    let generated = generate_layers(source, enabled).and_then(|(blocks, concealed)| {
        let arena = Arena::new(terrain, blocks, classic_spawns()).ok()?;
        validate_generated(&arena, &concealed).then_some((arena, concealed))
    });
    let (arena, concealed_pickups, fallback_used) = match generated {
        Some((arena, concealed)) => (arena, concealed, false),
        None => (fallback_arena(), BTreeMap::new(), true),
    };
    GeneratedArena {
        arena,
        concealed_pickups,
        record: GenerationRecord {
            version,
            seed,
            fallback_used,
        },
    }
}
fn generate_layers(
    source: &mut impl RandomWords,
    enabled: EnabledItems,
) -> Option<(BTreeSet<CellId>, BTreeMap<CellId, PickupKind>)> {
    let mut blocks = BTreeSet::new();
    let mut concealed = BTreeMap::new();
    for (cell, mirror) in eligible_pairs() {
        if sample_percent(source)? >= 65 {
            continue;
        }
        blocks.insert(cell);
        blocks.insert(mirror);
        if sample_percent(source)? >= 30 {
            continue;
        }
        let kind = sample_pickup(source, enabled)?;
        if kind == PickupKind::OilBarrel && !generated_pair_fits(&concealed, cell, mirror) {
            continue;
        }
        concealed.insert(cell, kind);
        concealed.insert(mirror, kind);
    }
    Some((blocks, concealed))
}
fn eligible_pairs() -> Vec<(CellId, CellId)> {
    let terrain = classic_terrain();
    let mut pairs = Vec::new();
    for row in 1..ARENA_HEIGHT - 1 {
        for column in 1..ARENA_WIDTH - 1 {
            let cell = CellId::new(column, row);
            let mirror = rotate(cell);
            let index = usize::from(row * ARENA_WIDTH + column);
            if cell <= mirror && terrain[index] == Terrain::Floor && !RESERVED_CELLS.contains(&cell)
            {
                pairs.push((cell, mirror));
            }
        }
    }
    pairs
}

fn validate_generated(arena: &Arena, concealed: &BTreeMap<CellId, PickupKind>) -> bool {
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            let cell = CellId::new(column, row);
            let mirror = rotate(cell);
            if arena.terrain(cell) != arena.terrain(mirror)
                || arena.blocks().contains(&cell) != arena.blocks().contains(&mirror)
                || concealed.get(&cell) != concealed.get(&mirror)
            {
                return false;
            }
        }
    }
    concealed.keys().all(|cell| arena.blocks().contains(cell)) && has_escape_witness(arena)
}

fn has_escape_witness(arena: &Arena) -> bool {
    ESCAPE_ROUTES.into_iter().all(|(spawn, bomb, safe)| {
        [spawn, bomb, safe].into_iter().all(|cell| {
            arena.terrain(cell) == Some(Terrain::Floor) && !arena.blocks().contains(&cell)
        }) && !blast_reaches(arena, bomb, safe, 2)
    })
}

fn blast_reaches(arena: &Arena, origin: CellId, target: CellId, range: u16) -> bool {
    let directions = [(1_i16, 0_i16), (-1, 0), (0, 1), (0, -1)];
    for (dx, dy) in directions {
        for distance in 1..=range {
            let column = i16::try_from(origin.column).unwrap() + dx * distance as i16;
            let row = i16::try_from(origin.row).unwrap() + dy * distance as i16;
            let Ok(column) = u16::try_from(column) else {
                break;
            };
            let Ok(row) = u16::try_from(row) else { break };
            let cell = CellId::new(column, row);
            if arena.terrain(cell) != Some(Terrain::Floor) {
                break;
            }
            if cell == target {
                return true;
            }
            if arena.blocks().contains(&cell) {
                break;
            }
        }
    }
    false
}

fn fallback_arena() -> Arena {
    Arena::new(classic_terrain(), BTreeSet::new(), classic_spawns())
        .expect("the classic empty fallback arena must remain valid")
}

const fn rotate(cell: CellId) -> CellId {
    CellId::new(ARENA_WIDTH - 1 - cell.column, ARENA_HEIGHT - 1 - cell.row)
}

const RESERVED_CELLS: [CellId; 6] = [
    CellId::new(1, 1),
    CellId::new(2, 1),
    CellId::new(1, 2),
    CellId::new(13, 11),
    CellId::new(12, 11),
    CellId::new(13, 10),
];

const ESCAPE_ROUTES: [(CellId, CellId, CellId); 2] = [
    (CellId::new(1, 1), CellId::new(2, 1), CellId::new(1, 2)),
    (
        CellId::new(13, 11),
        CellId::new(12, 11),
        CellId::new(13, 10),
    ),
];

#[cfg(test)]
#[path = "diarrhea_generation_tests.rs"]
mod diarrhea_tests;
#[cfg(test)]
mod golden_flame_tests;
#[cfg(test)]
mod invisibility_cloak_tests;
#[cfg(test)]
mod invisible_bombs_tests;
#[cfg(test)]
#[path = "landmine_generation_tests.rs"]
mod landmine_tests;
#[cfg(test)]
mod long_fuse_tests;
#[cfg(test)]
mod oil_tests;
#[cfg(test)]
#[path = "piercing_bomb_generation_tests.rs"]
mod piercing_bomb_tests;
#[cfg(test)]
#[path = "power_bomb_generation_tests.rs"]
mod power_bomb_tests;
#[cfg(test)]
#[path = "remote_bomb_generation_tests.rs"]
mod remote_bomb_tests;
#[cfg(test)]
#[path = "reverse_controls_generation_tests.rs"]
mod reverse_controls_tests;
#[cfg(test)]
mod shield_tests;
#[cfg(test)]
mod short_flame_tests;
#[cfg(test)]
mod short_fuse_tests;
#[cfg(test)]
#[path = "slow_foot_generation_tests.rs"]
mod slow_foot_tests;
#[cfg(test)]
mod stopwatch_tests;
#[cfg(test)]
mod teleporter_tests;
#[cfg(test)]
mod tests;
