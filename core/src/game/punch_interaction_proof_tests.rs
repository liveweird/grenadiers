use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    BlastKind, Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, Position,
    RulesConfig, SeatId, SpeedLevel, StepEventKind, Tick, World,
};
use crate::room::RoomSettings;

fn arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
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

fn state(one: Position, two: Position) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL)),
        World::new(arena()),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: one,
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: two,
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap()
}

fn timed(id: u32, cell: CellId, detonates_at: Tick) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Timed { detonates_at },
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn tangent_west(cell: CellId) -> Position {
    Position {
        x: i32::from(cell.column) * crate::game::types::UNITS_PER_CELL
            - crate::game::types::HALF_FOOTPRINT,
        y: Position::center(cell).y,
    }
}

#[test]
fn punch_preserves_piercing_identity_range_timing_and_trigger() {
    let mut state = state(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
    );
    let bomb = Bomb::new_with_blast(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(2),
        7,
        Tick(3),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
        BlastKind::Piercing,
    )
    .unwrap();
    state.world.bombs.insert(bomb.id(), bomb);
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);

    state
        .step(&[input(SeatId::One, InputAction::UseEquipment)])
        .unwrap();

    let moved = state.world.bombs[&EntityId(9)];
    assert_eq!(moved.cell(), CellId::new(8, 5));
    assert_eq!(moved.id(), EntityId(9));
    assert_eq!(moved.owner(), EntityId(2));
    assert_eq!(moved.range(), 7);
    assert_eq!(moved.placed_at(), Tick(3));
    assert_eq!(
        moved.trigger(),
        BombTrigger::Timed {
            detonates_at: Tick(100)
        }
    );
    assert_eq!(moved.blast(), BlastKind::Piercing);
}

#[test]
fn punched_bomb_can_be_kicked_again_on_a_later_update() {
    let source = CellId::new(5, 5);
    let mut state = state(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
    );
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .step(&[input(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));

    state.actors[0].position = tangent_west(CellId::new(8, 5));
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state
        .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();

    let armed = state.world.bombs[&EntityId(9)];
    assert_eq!(armed.cell(), CellId::new(8, 5));
    assert_eq!(armed.motion().unwrap().direction(), Direction::East);
    assert_eq!(armed.motion().unwrap().countdown(), 0);
    assert!(armed.has_been_kicked());
    assert!(armed.has_been_punched());
}

#[test]
fn newly_armed_kick_is_visible_to_same_update_punch_and_motion_is_cleared() {
    let source = CellId::new(5, 5);
    let mut state = state(tangent_west(source), Position::center(CellId::new(6, 5)));
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state.replace_actor_equipment(SeatId::Two, Equipment::Punch);

    state
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::East)),
            input(SeatId::Two, InputAction::UseEquipment),
        ])
        .unwrap();

    let moved = state.world.bombs[&EntityId(9)];
    assert_eq!(moved.cell(), CellId::new(2, 5));
    assert_eq!(moved.motion(), None);
    assert!(moved.has_been_kicked());
    assert!(moved.has_been_punched());
}

#[test]
fn due_bomb_rejects_punch_then_actually_detonates_without_starting_cooldown() {
    let source = CellId::new(5, 5);
    let mut state = state(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
    );
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(1)));
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);

    let report = state
        .step(&[input(SeatId::One, InputAction::UseEquipment)])
        .unwrap();

    assert!(!state.world.bombs.contains_key(&EntityId(9)));
    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(9)))
        .unwrap();
    assert!(matches!(event.kind, StepEventKind::BombDetonated(view) if view.cell == source));
    assert!(!event.bomb_has_been_punched);
    assert_eq!(state.actors[0].last_successful_punch_tick(), None);
}

#[test]
fn contact_bomb_rejects_punch_and_remains_unchanged_without_cooldown() {
    let source = CellId::new(5, 5);
    let mut state = state(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
    );
    let mine = Bomb::new(
        EntityId(9),
        source,
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    state.world.bombs.insert(mine.id(), mine);
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);

    let report = state
        .step(&[input(SeatId::One, InputAction::UseEquipment)])
        .unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)], mine);
    assert!(report.events.iter().all(
        |event| !matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(9))
    ));
    assert_eq!(state.actors[0].last_successful_punch_tick(), None);
}
