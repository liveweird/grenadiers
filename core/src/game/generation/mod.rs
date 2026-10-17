use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

use super::arena::{ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use super::fixtures::{classic_spawns, classic_terrain};
use super::pickups::PickupKind;
use super::types::CellId;

/// The algorithm version recorded with arenas produced by this module.
pub const GENERATOR_VERSION: u16 = 1;
const SAMPLE_LIMIT: u32 = 4_294_967_200;
const MAX_DRAWS_PER_SAMPLE: usize = 16;

/// Replay metadata retained by trusted domain code.
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

/// A generated arena and its server-private concealed pickup layer.
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

/// Generates the version-one arena for an exact 32-byte seed.
#[must_use]
pub fn generate_arena(seed: [u8; 32]) -> GeneratedArena {
    let mut rng = ChaCha20Rng::from_seed(seed);
    rng.set_stream(0);
    rng.set_word_pos(0);
    generate_with_source(seed, &mut ChaChaWords(rng), classic_terrain())
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
    let generated = generate_layers(source).and_then(|(blocks, concealed)| {
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
            version: GENERATOR_VERSION,
            seed,
            fallback_used,
        },
    }
}

fn generate_layers(
    source: &mut impl RandomWords,
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
        let kind = match sample_percent(source)? {
            0..40 => PickupKind::BombCapacity,
            40..80 => PickupKind::BlastRange,
            _ => PickupKind::MovementSpeed,
        };
        concealed.insert(cell, kind);
        concealed.insert(mirror, kind);
    }
    Some((blocks, concealed))
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
    let reserved = reserved_cells();
    let mut pairs = Vec::new();
    for row in 1..ARENA_HEIGHT - 1 {
        for column in 1..ARENA_WIDTH - 1 {
            let cell = CellId::new(column, row);
            let mirror = rotate(cell);
            if cell <= mirror && terrain[index(cell)] == Terrain::Floor && !reserved.contains(&cell)
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
    escape_routes().into_iter().all(|(spawn, bomb, safe)| {
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

const fn reserved_cells() -> [CellId; 6] {
    [
        CellId::new(1, 1),
        CellId::new(2, 1),
        CellId::new(1, 2),
        CellId::new(13, 11),
        CellId::new(12, 11),
        CellId::new(13, 10),
    ]
}

const fn escape_routes() -> [(CellId, CellId, CellId); 2] {
    [
        (CellId::new(1, 1), CellId::new(2, 1), CellId::new(1, 2)),
        (
            CellId::new(13, 11),
            CellId::new(12, 11),
            CellId::new(13, 10),
        ),
    ]
}

fn index(cell: CellId) -> usize {
    usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
