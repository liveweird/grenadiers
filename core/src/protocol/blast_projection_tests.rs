use std::collections::BTreeSet;

use super::{ProjectionError, project_events, project_state};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    ActorInit, Arena, BlastKind, BombTrigger, CellId, Direction, EntityId, EventAudience,
    InputAction, MovementTrace, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    StepEvent, StepEventKind, StepReport, Terrain, Tick, World,
};

fn state() -> State {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actor = |id, seat, cell| ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    };
    State::new(
        RulesConfig::classic_v1(),
        World::new(arena),
        [
            actor(10, SeatId::One, CellId::new(1, 1)),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap()
}

#[test]
fn legacy_state_projection_rejects_every_special_blast_without_downgrading() {
    let mut state = state();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        let mut view = state.public_view();
        view.bombs[0].blast = blast;
        assert_eq!(
            project_state(&view),
            Err(ProjectionError::UnsupportedBombBlast)
        );
    }
}

#[test]
fn legacy_event_projection_rejects_every_special_blast_without_final_state_bombs() {
    let state = state();
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        let bomb = crate::game::BombView {
            id: EntityId(30),
            cell: CellId::new(3, 3),
            owner: EntityId(10),
            range: 2,
            placed_at: Tick(0),
            trigger: BombTrigger::Timed {
                detonates_at: Tick(180),
            },
            blast,
        };
        for kind in [
            StepEventKind::BombPlaced(bomb),
            StepEventKind::BombDetonated(bomb),
        ] {
            let report = StepReport {
                tick: Tick(0),
                traces: SeatId::ALL
                    .map(|seat| MovementTrace::new(state.actors()[seat.index()].position())),
                events: vec![StepEvent::new(Tick(0), EventAudience::Both, kind)],
            };
            assert_eq!(
                project_events(&[report]),
                Err(ProjectionError::InvalidEvent)
            );
        }
    }
}
