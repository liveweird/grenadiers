use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, State, StepReport, Terrain, World,
};
use crate::room::RoomSettings;

pub(super) fn bracelet_shove_report() -> StepReport {
    bracelet_shove().1
}

pub(super) fn bracelet_shove() -> (State, StepReport) {
    let mut state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::BRACELET)),
        World::with_pickups(
            open_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(CellId::new(4, 4), PickupKind::Bracelet)]),
        )
        .unwrap(),
        [
            actor(
                1,
                SeatId::One,
                Position { x: 4_608, y: 4_096 },
                Direction::East,
            ),
            actor(
                2,
                SeatId::Two,
                Position { x: 5_248, y: 4_096 },
                Direction::West,
            ),
        ],
    )
    .unwrap();
    state.step(&[]).unwrap();
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(report.forced_movement.as_ref().unwrap().seat, SeatId::Two);
    (state, report)
}

#[test]
fn base_event_projection_rejects_only_unsupported_forced_movement_victims() {
    let report = bracelet_shove_report();
    assert!(super::project_events(std::slice::from_ref(&report)).is_ok());

    let mut malicious = report;
    malicious.forced_movement.as_mut().unwrap().seat = SeatId::Three;
    assert_eq!(
        super::project_events(&[malicious]),
        Err(super::ProjectionError::InvalidEvent)
    );
}

fn actor(id: u32, seat: SeatId, position: Position, heading: Direction) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed: SpeedLevel::Base,
    }
}

fn open_arena() -> Arena {
    let mut terrain = [Terrain::Floor; crate::game::arena::ARENA_CELL_COUNT];
    for row in 0..crate::game::arena::ARENA_HEIGHT {
        for column in 0..crate::game::arena::ARENA_WIDTH {
            if row == 0
                || column == 0
                || row == crate::game::arena::ARENA_HEIGHT - 1
                || column == crate::game::arena::ARENA_WIDTH - 1
            {
                terrain[usize::from(row * crate::game::arena::ARENA_WIDTH + column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}
