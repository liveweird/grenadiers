use std::collections::{BTreeMap, BTreeSet};

use crate::room::RoomSettings;

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment,
    InputAction, OilState, Position, RoundResult, RulesConfig, SeatId, SeatInput, SpeedLevel,
    State, Terrain, Tick, World, fixed_arena,
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

fn world(sites: impl IntoIterator<Item = (CellId, OilState)>) -> World {
    World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        sites.into_iter().collect(),
    )
    .unwrap()
}

#[test]
fn stopwatch_does_not_pause_burning_oil_damage() {
    let burning = CellId::new(11, 11);
    let mut init = actors();
    init[1].position = Position::center(burning);
    let mut state = State::new(
        rules(EnabledItems::STOPWATCH),
        world([(burning, OilState::Burning)]),
        init,
    )
    .unwrap();
    state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));
    assert!(state.voluntary_actions_frozen()[1]);

    state.step(&[]).unwrap();

    assert!(!state.actors[1].alive);
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::One)));
}

#[test]
fn terminal_round_freezes_every_oil_phase_while_ticks_continue() {
    let burning = CellId::new(3, 1);
    let pending = CellId::new(4, 1);
    let expiry = Tick(5);
    let mut init = actors();
    init[1].position = Position::center(burning);
    let mut oil_world = world([(burning, OilState::Burning), (pending, OilState::Pending)]);
    oil_world.hazards.insert(pending, expiry);
    let mut state = State::new(rules(EnabledItems::EMPTY), oil_world, init).unwrap();
    state.step(&[]).unwrap();
    assert!(state.result().is_some());
    assert_eq!(state.world.oil.state(pending), Some(OilState::Pending));
    assert_eq!(state.world.hazards.get(&pending), Some(&expiry));
    let frozen = state.world.oil.clone();
    let updates = state.running_updates();

    for expected in 2..=6 {
        state.step(&[]).unwrap();
        assert_eq!(state.tick(), Tick(expected));
    }

    assert_eq!(state.running_updates(), updates);
    assert_eq!(state.world.oil, frozen);
    assert_eq!(state.world.oil.state(pending), Some(OilState::Pending));
    assert_eq!(state.world.hazards.get(&pending), Some(&expiry));

    let fresh = State::new(
        rules(EnabledItems::EMPTY),
        World::new(fixed_arena()),
        actors(),
    )
    .unwrap();
    assert_eq!(fresh.tick(), Tick(0));
    assert_eq!(fresh.running_updates(), 0);
    assert_eq!(fresh.result(), None);
    assert!(fresh.world.oil.iter().next().is_none());
}

#[test]
fn oil_blocked_by_a_living_actor_remains_private_through_exact_timeout() {
    let pending = CellId::new(1, 1);
    let config = rules(EnabledItems::EMPTY);
    let mut state = State::new(config, world([(pending, OilState::Pending)]), actors()).unwrap();

    for _ in 0..config.round_ticks() - 1 {
        state.step(&[]).unwrap();
    }
    assert_eq!(state.running_updates(), config.round_ticks() - 1);
    assert_eq!(state.world.oil.state(pending), Some(OilState::Pending));
    assert!(state.public_view().oil_barrels.is_empty());
    assert_eq!(state.result(), None);
    state.step(&[]).unwrap();

    assert_eq!(state.running_updates(), config.round_ticks());
    assert_eq!(
        state.result(),
        Some(RoundResult::Draw(super::super::DrawReason::Timeout))
    );
    assert_eq!(state.world.oil.state(pending), Some(OilState::Pending));
    assert!(state.public_view().oil_barrels.is_empty());
}

#[test]
fn cloning_and_replaying_preserves_private_oil_deterministically() {
    let pending = CellId::new(3, 1);
    let burning = CellId::new(4, 1);
    let mut left = State::new(
        rules(EnabledItems::EMPTY),
        world([(pending, OilState::Pending), (burning, OilState::Burning)]),
        actors(),
    )
    .unwrap();
    let mut right = left.clone();

    assert_eq!(left.step(&[]), right.step(&[]));
    assert_eq!(left, right);
    assert!(World::new(fixed_arena()).oil.iter().next().is_none());
}

#[test]
fn voluntary_trace_into_burning_oil_causes_damage() {
    let burning = CellId::new(3, 1);
    let mut init = actors();
    init[0].position = Position {
        x: 2_752,
        ..Position::center(burning)
    };
    let mut state = State::new(
        rules(EnabledItems::EMPTY),
        world([(burning, OilState::Burning)]),
        init,
    )
    .unwrap();

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert!(!state.actors[0].alive);
}

#[test]
fn bracelet_forced_segment_through_burning_oil_causes_damage() {
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
    let burning = CellId::new(5, 4);
    let oil_world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(burning, OilState::Burning)]),
    )
    .unwrap();
    let mut state = State::new(
        rules(EnabledItems::BRACELET),
        oil_world,
        [
            ActorInit {
                position: Position { x: 4_096, y: 4_096 },
                ..actors()[0]
            },
            ActorInit {
                position: Position { x: 4_736, y: 4_096 },
                ..actors()[1]
            },
        ],
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Bracelet);

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert!(
        report
            .forced_movements.into_iter().next().unwrap()
            .segment
            .overlaps_cell(burning)
    );
    assert!(!state.actors[1].alive);
}
