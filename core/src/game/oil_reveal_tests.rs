use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, InputAction,
    OilState, Pickup, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    Tick, World, fixed_arena,
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

fn pending_state(cell: CellId, actor_position: Position) -> State {
    let world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cell, OilState::Pending)]),
    )
    .unwrap();
    let enabled = EnabledItems::from_bits(
        EnabledItems::OIL_BARREL.bits()
            | EnabledItems::WALL_PASS.bits()
            | EnabledItems::STOPWATCH.bits()
            | EnabledItems::INVISIBILITY_CLOAK.bits()
            | EnabledItems::SHIELD.bits(),
    )
    .unwrap();
    let rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled));
    let mut init = actors();
    init[0].position = actor_position;
    State::new(rules, world, init).unwrap()
}

#[test]
fn reveal_uses_the_start_scan_even_when_an_actor_moves_away() {
    let oil = CellId::new(3, 1);
    let mut edge_overlap = Position::center(oil);
    edge_overlap.x = 2_753;
    let mut state = pending_state(oil, edge_overlap);

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::West),
        }])
        .unwrap();
    assert_eq!(state.world.oil.state(oil), Some(OilState::Pending));
    assert!(state.actors[0].position.x < 2_752);

    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(oil), Some(OilState::Unlit));
}

#[test]
fn reveal_footprint_tangency_is_safe_but_one_unit_of_overlap_blocks() {
    let oil = CellId::new(3, 1);
    let mut tangent = Position::center(oil);
    tangent.x = 2_752;
    let mut safe = pending_state(oil, tangent);
    safe.step(&[]).unwrap();
    assert_eq!(safe.world.oil.state(oil), Some(OilState::Unlit));

    let mut overlap = tangent;
    overlap.x += 1;
    let mut blocked = pending_state(oil, overlap);
    blocked.step(&[]).unwrap();
    assert_eq!(blocked.world.oil.state(oil), Some(OilState::Pending));
}

#[test]
fn every_living_capability_still_blocks_reveal_and_emits_nothing() {
    let oil = CellId::new(3, 1);
    for setup in 0..4 {
        let mut state = pending_state(oil, Position::center(oil));
        match setup {
            0 => state.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0)),
            1 => state.apply_test_effect(SeatId::One, EffectKind::InvisibilityCloak, Tick(0)),
            2 => state.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(0)),
            3 => state.replace_actor_equipment(SeatId::One, Equipment::WallPass),
            _ => unreachable!(),
        }
        let report = state.step(&[]).unwrap();
        assert_eq!(state.world.oil.state(oil), Some(OilState::Pending));
        assert!(report.events.is_empty());
    }

    let mut dead = pending_state(oil, Position::center(oil));
    dead.actors[0].alive = false;
    dead.step(&[]).unwrap();
    assert_eq!(dead.world.oil.state(oil), Some(OilState::Unlit));
}

#[test]
fn retained_block_and_pickup_each_delay_reveal_without_an_event() {
    let oil = CellId::new(5, 1);
    let safe = Position::center(CellId::new(1, 2));
    let mut retained = World::with_pickups(
        fixed_arena(),
        BTreeMap::from([(oil, PickupKind::OilBarrel)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    retained.oil.set_state(oil, OilState::Pending);
    let mut init = actors();
    init[0].position = safe;
    let mut retained = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::OIL_BARREL),
        ),
        retained,
        init,
    )
    .unwrap();
    let report = retained.step(&[]).unwrap();
    assert_eq!(retained.world.oil.state(oil), Some(OilState::Pending));
    assert!(report.events.is_empty());

    let pickup_oil = CellId::new(3, 1);
    let mut state = pending_state(pickup_oil, safe);
    state
        .world
        .pickups
        .exposed
        .insert(pickup_oil, Pickup::new(PickupKind::BlastRange));
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(pickup_oil), Some(OilState::Pending));

    state.world.pickups.exposed.remove(&pickup_oil);
    let report = state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(pickup_oil), Some(OilState::Unlit));
    assert!(report.events.is_empty());
}
