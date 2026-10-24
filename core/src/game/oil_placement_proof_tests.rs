use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, BlastKind, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction,
    OilState, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State, World,
    fixed_arena,
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

fn state() -> State {
    let enabled = EnabledItems::from_bits(
        EnabledItems::OIL_BARREL.bits()
            | EnabledItems::WALL_PASS.bits()
            | EnabledItems::PIERCING_BOMB.bits(),
    )
    .unwrap();
    let oil = CellId::new(3, 1);
    let world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Unlit)]),
    )
    .unwrap();
    let mut state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        world,
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);
    state.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Piercing);
    state
}

fn place(seat: SeatId) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::PlaceBomb,
    }
}

#[test]
fn unlit_veto_preserves_allocator_weapon_and_capacity_with_a_free_center_control() {
    let mut blocked = state();
    let next = blocked.next_entity_id;
    let capacity = blocked.actors[0].capacity;
    blocked.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(blocked.next_entity_id, next);
    assert_eq!(blocked.actors[0].capacity, capacity);
    assert_eq!(
        blocked.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
    assert!(blocked.world.bombs.is_empty());

    let mut allowed = state();
    allowed.actors[0].position = Position { x: 2_753, y: 1_536 };
    assert!(crate::game::movement::position_overlaps_cell(
        allowed.actors[0].position,
        CellId::new(3, 1)
    ));
    allowed.step(&[place(SeatId::One)]).unwrap();
    let bomb = allowed.world.bombs.get(&EntityId(next as u32)).unwrap();
    assert_eq!(bomb.cell(), CellId::new(2, 1));
    assert_eq!(bomb.blast(), BlastKind::Piercing);
    assert_eq!(allowed.actors[0].inventory.weapon(), None);
    assert_eq!(allowed.next_entity_id, next + 1);
}

#[test]
fn one_unlit_veto_does_not_suppress_the_other_actors_valid_neighbor_placement() {
    let mut state = state();
    let next = state.next_entity_id;
    state.actors[1].position = Position { x: 2_753, y: 1_536 };
    assert!(crate::game::movement::position_overlaps_cell(
        state.actors[1].position,
        CellId::new(3, 1)
    ));

    state
        .step(&[place(SeatId::One), place(SeatId::Two)])
        .unwrap();

    assert_eq!(state.world.bombs.len(), 1);
    let bomb = state.world.bombs.get(&EntityId(next as u32)).unwrap();
    assert_eq!(bomb.owner(), EntityId(2));
    assert_eq!(bomb.cell(), CellId::new(2, 1));
    assert_eq!(
        state.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
}
