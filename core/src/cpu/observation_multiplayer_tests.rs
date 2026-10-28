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
