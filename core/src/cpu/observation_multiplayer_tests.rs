use crate::game::{CellId, EntityId, SeatId, Tick};

use super::tests::observation;
use super::*;

#[test]
fn multiplayer_observation_accepts_canonical_four_actors_and_thirty_two_bombs() {
    let mut parts = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    parts.rules.version = 6;
    parts.actors.push(ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    });
    parts.actors.push(ObservedActor::Hidden {
        id: EntityId(4),
        seat: SeatId::Four,
        alive: true,
    });
    parts.bombs = BombKnowledge::Visible(
        (0..MAX_MULTIPLAYER_OBSERVED_BOMBS)
            .map(|index| ObservedBomb {
                id: EntityId(100 + index as u32),
                placed_at: Tick(0),
                cell: CellId::new((index % 13 + 1) as u16, (index / 13 + 1) as u16),
                owner: EntityId(2),
                range: 2,
                trigger: ObservedBombTrigger::Remote,
                blast: crate::game::BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            })
            .collect(),
    );
    assert_eq!(Observation::new(parts).unwrap().parts().actors.len(), 4);
}

#[test]
fn actor_roster_rejects_legacy_expansion_and_noncanonical_multiplayer_order() {
    let mut legacy = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    legacy.actors.push(ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    });
    assert_eq!(
        Observation::new(legacy),
        Err(ObservationError::InvalidActors)
    );

    let mut multiplayer = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    multiplayer.rules.version = 6;
    multiplayer.actors.swap(0, 1);
    assert_eq!(
        Observation::new(multiplayer),
        Err(ObservationError::InvalidActors)
    );
}

fn resize(parts: &mut ObservationParts, width: u16, height: u16) {
    parts.rules.version = 6;
    parts.arena.width = width;
    parts.arena.height = height;
    parts.arena.terrain = vec![crate::game::Terrain::Floor; usize::from(width * height)];
    for row in 0..height {
        for column in 0..width {
            if row == 0 || column == 0 || row + 1 == height || column + 1 == width {
                parts.arena.terrain[usize::from(row) * usize::from(width) + usize::from(column)] =
                    crate::game::Terrain::Wall;
            }
        }
    }
}

#[test]
fn small_multiplayer_observation_requires_exact_preset_geometry() {
    let mut parts = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    resize(&mut parts, 11, 9);
    let observed = Observation::new(parts.clone()).unwrap();
    assert_eq!(
        observed.parts().arena.board(),
        Some(crate::room::BoardSize::ElevenByNine)
    );
    assert_eq!(observed.parts().arena.cell_count(), 99);

    let mut legacy = parts.clone();
    legacy.rules.version = 5;
    assert_eq!(
        Observation::new(legacy),
        Err(ObservationError::InvalidArena)
    );

    parts.arena.terrain.push(crate::game::Terrain::Wall);
    assert_eq!(Observation::new(parts), Err(ObservationError::InvalidArena));
}

#[test]
fn small_observation_rejects_flat_index_axis_aliases() {
    let mut parts = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    resize(&mut parts, 11, 9);
    parts.flames.push(ObservedFlame {
        cell: CellId::new(11, 0),
        remaining_ticks: 1,
    });
    assert_eq!(
        Observation::new(parts),
        Err(ObservationError::InvalidFlames)
    );
}
