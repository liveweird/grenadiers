use std::collections::BTreeMap;

use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::{
    ArenaGenerationConfig, CellId, Direction, EnabledItems, EntityId, Equipment, OilState,
    PickupKind, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId, SimulationSeed,
    SpeedLevel, StepError, StormCloudConfig, Tick, World, fixed_arena,
    generate_variable_multiplayer_arena,
};
use crate::room::{BoardSize, LairConfig, LairSettings, RoomSettings};

fn rules(items: EnabledItems) -> RulesConfig {
    RulesConfig::environment_v8(
        RoomSettings::DEFAULT.with_enabled_items(items),
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(60, 8).unwrap()),
    )
}

fn actors(one: Position, two: Position) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(10),
            seat: SeatId::One,
            position: one,
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(20),
            seat: SeatId::Two,
            position: two,
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn clear_world(oil: impl IntoIterator<Item = (CellId, OilState)>) -> World {
    let mut arena = fixed_arena();
    let blocks = arena.blocks().clone();
    arena.remove_blocks(&blocks);
    World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        oil.into_iter().collect(),
    )
    .unwrap()
}

fn insert_mob(state: &mut State, position: Position, heading: Direction) {
    state
        .lairs
        .as_mut()
        .unwrap()
        .insert_test_mob(EntityId(90), 0, position, heading);
}

#[test]
fn forced_sweep_contact_eliminates_before_the_victim_can_plant() {
    let y = Position::center(CellId::new(4, 3)).y;
    let mut state = State::new(
        rules(EnabledItems::BRACELET),
        clear_world([]),
        actors(Position { x: 4_096, y }, Position { x: 4_736, y }),
    )
    .unwrap();
    state.replace_test_actor_equipment(SeatId::One, Equipment::Bracelet);
    insert_mob(&mut state, Position { x: 5_200, y }, Direction::West);

    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();

    assert!(!state.actor(SeatId::Two).unwrap().alive());
    assert!(
        state
            .world()
            .bombs()
            .values()
            .all(|bomb| bomb.owner() != EntityId(20))
    );
}

#[test]
fn forced_endpoint_contact_eliminates_before_heart_collection() {
    let y = Position::center(CellId::new(4, 3)).y;
    let mut state = State::new(
        rules(EnabledItems::from_bits(
            EnabledItems::BRACELET.bits() | EnabledItems::HEART.bits(),
        )
        .unwrap()),
        clear_world([]),
        actors(Position { x: 4_096, y }, Position { x: 4_736, y }),
    )
    .unwrap();
    state.replace_test_actor_equipment(SeatId::One, Equipment::Bracelet);
    state.insert_test_pickup(CellId::new(5, 3), PickupKind::Heart);
    insert_mob(&mut state, Position { x: 5_760, y }, Direction::West);

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert!(!state.actor(SeatId::Two).unwrap().alive());
    assert_eq!(state.actor(SeatId::Two).unwrap().inventory().equipment(), None);
    assert!(state.world().exposed_pickups().contains_key(&CellId::new(5, 3)));
}

#[test]
fn voluntary_sweep_and_teleport_use_only_their_actual_contact_geometry() {
    let y = Position::center(CellId::new(3, 3)).y;
    let mut voluntary = State::new(
        rules(EnabledItems::EMPTY),
        clear_world([]),
        actors(Position { x: 3_520, y }, Position::center(CellId::new(13, 11))),
    )
    .unwrap();
    insert_mob(&mut voluntary, Position { x: 4_200, y }, Direction::West);
    voluntary
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(!voluntary.actor(SeatId::One).unwrap().alive());

    let one = Position::center(CellId::new(1, 1));
    let two = Position::center(CellId::new(13, 1));
    let mut endpoint = State::new(
        rules(EnabledItems::EMPTY),
        clear_world([]),
        actors(one, two),
    )
    .unwrap();
    insert_mob(&mut endpoint, two, Direction::West);
    endpoint.commit_test_teleport([true, false], Tick(1));
    endpoint.apply_lair_endpoint_contacts(
        &mut [false; 2],
        Tick(1),
        &mut Vec::new(),
        &mut [false; 2],
    );
    assert!(!endpoint.actor(SeatId::One).unwrap().alive());

    let mut no_sweep = State::new(
        rules(EnabledItems::EMPTY),
        clear_world([]),
        actors(one, two),
    )
    .unwrap();
    insert_mob(
        &mut no_sweep,
        Position::center(CellId::new(7, 1)),
        Direction::West,
    );
    no_sweep.commit_test_teleport([true, false], Tick(1));
    no_sweep.apply_lair_endpoint_contacts(
        &mut [false; 2],
        Tick(1),
        &mut Vec::new(),
        &mut [false; 2],
    );
    assert!(no_sweep.actors().iter().all(|actor| actor.alive()));
}

#[test]
fn pending_oil_waits_for_mob_exit_and_unlit_oil_is_a_real_collision() {
    let pending = CellId::new(3, 3);
    let mut reveal = State::new(
        rules(EnabledItems::OIL_BARREL),
        clear_world([(pending, OilState::Pending)]),
        actors(
            Position::center(CellId::new(1, 1)),
            Position::center(CellId::new(13, 11)),
        ),
    )
    .unwrap();
    insert_mob(&mut reveal, Position::center(pending), Direction::West);
    reveal.step(&[]).unwrap();
    assert_eq!(reveal.world.oil.state(pending), Some(OilState::Pending));
    for _ in 0..80 {
        reveal.step(&[]).unwrap();
        if reveal.world.oil.state(pending) == Some(OilState::Unlit) {
            break;
        }
    }
    assert_eq!(reveal.world.oil.state(pending), Some(OilState::Unlit));
    assert!(!reveal.lairs.as_ref().unwrap().overlaps_cell(pending));

    let solid = CellId::new(4, 3);
    let mut collision = State::new(
        rules(EnabledItems::OIL_BARREL),
        clear_world([(solid, OilState::Unlit)]),
        actors(
            Position::center(CellId::new(13, 3)),
            Position::center(CellId::new(1, 11)),
        ),
    )
    .unwrap();
    insert_mob(
        &mut collision,
        Position {
            x: 3_700,
            y: Position::center(solid).y,
        },
        Direction::East,
    );
    for _ in 0..8 {
        collision.step(&[]).unwrap();
    }
    let mob = collision.mobs()[0];
    assert_eq!(mob.position.x, 3_776);
    assert!(!crate::game::movement::position_overlaps_cell(mob.position, solid));
}

#[test]
fn late_entity_id_failure_rolls_back_seeded_teleport_randomness() {
    let assignment = RoundAssignment::first();
    let generated = generate_variable_multiplayer_arena(
        [41; 32],
        EnabledItems::TELEPORTER,
        ArenaGenerationConfig {
            player_count: PlayerCount::Three,
            board: BoardSize::FifteenByThirteen,
            assignment,
        },
    )
    .unwrap();
    let (mut world, _) = World::from_generated(generated).unwrap();
    let blocks = world.arena.blocks().clone();
    world.arena.remove_blocks(&blocks);
    let inits: Vec<_> = SeatId::ALL[..3]
        .iter()
        .copied()
        .map(|seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(world.arena.spawns()[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect();
    let mut state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v8(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER),
            StormCloudConfig::Disabled,
            LairConfig::Enabled(LairSettings::new(5, 8).unwrap()),
        ),
        world,
        inits,
        Tick(0),
        assignment,
        SimulationSeed::new([73; 32]),
    )
    .unwrap();
    state.tick = Tick(299);
    let pickup = state.actors[0].position.cell().unwrap();
    state.insert_test_pickup(pickup, PickupKind::Teleporter);
    state.next_entity_id = u64::from(u32::MAX) + 1;
    let before = state.clone();

    assert_eq!(state.step(&[]), Err(StepError::EntityIdOverflow));
    assert_eq!(state, before);
}
