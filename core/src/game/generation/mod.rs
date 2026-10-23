use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

use super::arena::{ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use super::enabled_items::EnabledItems;
use super::fixtures::{classic_spawns, classic_terrain};
use super::pickups::PickupKind;
use super::types::CellId;

pub const GENERATOR_VERSION: u16 = 1;
pub const EXPANDED_GENERATOR_VERSION: u16 = 2;
const SAMPLE_LIMIT: u32 = 4_294_967_200;
const MAX_DRAWS_PER_SAMPLE: usize = 16;

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
        formatter
            .debug_struct("GeneratedArena")
            .field("arena", &self.arena)
            .field("concealed_pickup_count", &self.concealed_pickups.len())
            .field("record", &self.record)
            .finish()
    }
}

#[must_use]
pub fn generate_arena(seed: [u8; 32]) -> GeneratedArena {
    let mut rng = ChaCha20Rng::from_seed(seed);
    rng.set_stream(0);
    rng.set_word_pos(0);
    generate_with_source(seed, &mut ChaChaWords(rng), classic_terrain())
}

#[must_use]
pub fn generate_expanded_arena(seed: [u8; 32], enabled: EnabledItems) -> GeneratedArena {
    let mut rng = ChaCha20Rng::from_seed(seed);
    rng.set_stream(0);
    rng.set_word_pos(0);
    generate_expanded_with_source(seed, &mut ChaChaWords(rng), classic_terrain(), enabled)
}

trait RandomWords {
    fn next_u32(&mut self) -> Option<u32>;
}

struct ChaChaWords(ChaCha20Rng);

impl RandomWords for ChaChaWords {
    fn next_u32(&mut self) -> Option<u32> {
        Some(self.0.next_u32())
    }
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
        concealed.insert(cell, kind);
        concealed.insert(mirror, kind);
    }
    Some((blocks, concealed))
}

fn sample_pickup(source: &mut impl RandomWords, enabled: EnabledItems) -> Option<PickupKind> {
    let heart = u32::from(enabled.contains_heart());
    let bound = 100 + heart * 25 + (enabled.bits().count_ones() - heart) * 15;
    let value = if bound == 100 {
        sample_percent(source)?
    } else {
        sample_below(source, bound)?
    };
    let heart_end = 100 + u32::from(enabled.contains_heart()) * 25;
    let shield_end = heart_end + u32::from(enabled.contains_shield()) * 15;
    let golden_end = shield_end + u32::from(enabled.contains_golden_flame()) * 15;
    let short_fuse_end = golden_end + u32::from(enabled.contains_short_fuse()) * 15;
    let short_flame_end = short_fuse_end + u32::from(enabled.contains_short_flame()) * 15;
    let long_fuse_end = short_flame_end + u32::from(enabled.contains_long_fuse()) * 15;
    let reverse_controls_end = long_fuse_end + u32::from(enabled.contains_reverse_controls()) * 15;
    let diarrhea_end = reverse_controls_end + u32::from(enabled.contains_diarrhea()) * 15;
    let slow_foot_end = diarrhea_end + u32::from(enabled.contains_slow_foot()) * 15;
    let piercing_end = slow_foot_end + u32::from(enabled.contains_piercing_bomb()) * 15;
    let power_end = piercing_end + u32::from(enabled.contains_power_bomb()) * 15;
    let remote_end = power_end + u32::from(enabled.contains_remote_bomb()) * 15;
    let landmine_end = remote_end + u32::from(enabled.contains_landmine()) * 15;
    let cloak_end = landmine_end + u32::from(enabled.contains_invisibility_cloak()) * 15;
    let stopwatch_end = cloak_end + u32::from(enabled.contains_stopwatch()) * 15;
    let invisible_bombs_end = stopwatch_end + u32::from(enabled.contains_invisible_bombs()) * 15;
    let teleporter_end = invisible_bombs_end + u32::from(enabled.contains_teleporter()) * 15;
    let wall_pass_end = teleporter_end + u32::from(enabled.contains_wall_pass()) * 15;
    Some(match value {
        0..40 => PickupKind::BombCapacity,
        40..80 => PickupKind::BlastRange,
        80..100 => PickupKind::MovementSpeed,
        value if value < heart_end => PickupKind::Heart,
        value if value < shield_end => PickupKind::Shield,
        value if value < golden_end => PickupKind::GoldenFlame,
        value if value < short_fuse_end => PickupKind::ShortFuse,
        value if value < short_flame_end => PickupKind::ShortFlame,
        value if value < long_fuse_end => PickupKind::LongFuse,
        value if value < reverse_controls_end => PickupKind::ReverseControls,
        value if value < diarrhea_end => PickupKind::Diarrhea,
        value if value < slow_foot_end => PickupKind::SlowFoot,
        value if value < piercing_end => PickupKind::PiercingBomb,
        value if value < power_end => PickupKind::PowerBomb,
        value if value < remote_end => PickupKind::RemoteBomb,
        value if value < landmine_end => PickupKind::Landmine,
        value if value < cloak_end => PickupKind::InvisibilityCloak,
        value if value < stopwatch_end => PickupKind::Stopwatch,
        value if value < invisible_bombs_end => PickupKind::InvisibleBombs,
        value if value < teleporter_end => PickupKind::Teleporter,
        value if value < wall_pass_end => PickupKind::WallPass,
        _ => unreachable!("sample value is bounded by the enabled item pool"),
    })
}

fn sample_below(source: &mut impl RandomWords, bound: u32) -> Option<u32> {
    let limit = u32::MAX - u32::MAX % bound;
    for _ in 0..MAX_DRAWS_PER_SAMPLE {
        let value = source.next_u32()?;
        if value < limit {
            return Some(value % bound);
        }
    }
    None
}

fn sample_percent(source: &mut impl RandomWords) -> Option<u32> {
    for _ in 0..MAX_DRAWS_PER_SAMPLE {
        let value = source.next_u32()?;
        if value < SAMPLE_LIMIT {
            return Some(value % 100);
        }
    }
    None
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
