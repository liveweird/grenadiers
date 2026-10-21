use std::collections::{BTreeMap, BTreeSet};

use crate::room::RoomSettings;

use super::super::observations::ExitGraceObservation;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, OilState,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, Terrain, Tick, World, fixed_arena,
};

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn rules(extra: EnabledItems) -> RulesConfig {
    let enabled = EnabledItems::from_bits(EnabledItems::OIL_BARREL.bits() | extra.bits()).unwrap();
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

fn oil_world(cell: CellId, state: OilState) -> World {
    World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cell, state)]),
    )
    .unwrap()
}

#[test]
fn typed_oil_grace_is_cleared_by_reacquisition_and_death() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::WALL_PASS),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    assert_eq!(
        state.actors[0]
            .exit_grace_observations()
            .collect::<Vec<_>>(),
        [ExitGraceObservation::UnlitOil(oil)]
    );

    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    assert!(state.actors[0].unlit_oil_grace.is_empty());
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state.world.oil.set_state(oil, OilState::Burning);
    state.step(&[]).unwrap();
    assert!(!state.actors[0].alive);
    assert!(state.actors[0].unlit_oil_grace.is_empty());
}

#[test]
fn teleport_does_not_transfer_an_actors_oil_grace_to_the_other_actor() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::WALL_PASS),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);
    let before = state.actors.each_ref().map(|actor| actor.position);

    state.commit_teleport([true, false], Tick(1));

    assert_eq!(state.actors.each_ref().map(|actor| actor.position), before);
    assert!(state.actors[1].unlit_oil_grace.is_empty());
}

#[test]
fn forced_exit_prunes_oil_grace_from_the_victim() {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row == ARENA_HEIGHT - 1 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let oil = CellId::new(5, 4);
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Pending)]),
    )
    .unwrap();
    let mut init = actors();
    init[0].position = Position { x: 4_161, y: 4_096 };
    init[1].position = Position { x: 4_801, y: 4_096 };
    let enabled =
        EnabledItems::from_bits(EnabledItems::BRACELET.bits() | EnabledItems::WALL_PASS.bits())
            .unwrap();
    let mut state = State::new(rules(enabled), world, init).unwrap();
    state.world.oil.set_state(oil, OilState::Unlit);
    state.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    state.replace_actor_equipment(SeatId::Two, Equipment::WallPass);
    state.replace_actor_equipment(SeatId::Two, Equipment::Heart);
    assert_eq!(state.actors[1].unlit_oil_grace, [oil].into());
    state.world.oil.set_state(oil, OilState::Pending);

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert!(report.forced_movement.is_some());
    assert!(state.actors[1].unlit_oil_grace.is_empty());
}
