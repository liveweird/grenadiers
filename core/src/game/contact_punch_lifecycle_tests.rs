use std::collections::{BTreeMap, BTreeSet};

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment,
    PickupKind, Position, PunchActivationPolicy, RulesConfig, SeatId, SpeedLevel, StepError, Tick,
    World,
};
use crate::room::RoomSettings;

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn tangent_west(cell: CellId) -> Position {
    Position {
        x: i32::from(cell.column) * crate::game::types::UNITS_PER_CELL
            - crate::game::types::HALF_FOOTPRINT,
        y: Position::center(cell).y,
    }
}

fn actors(position: Position) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position,
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

fn rules(enabled: EnabledItems) -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
        .with_punch_activation_policy(PunchActivationPolicy::ContactV2)
}

fn input(action: InputAction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action,
    }
}

fn state(source: CellId) -> State {
    let mut state = State::new(
        rules(EnabledItems::ALL),
        World::new(arena(BTreeSet::new())),
        actors(tangent_west(source)),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
}

#[test]
fn due_mine_and_reserved_remote_bombs_are_not_contact_punched() {
    let source = CellId::new(5, 5);
    for (id, trigger) in [
        (
            9,
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        ),
        (10, BombTrigger::Contact),
    ] {
        let mut state = state(source);
        state.world.bombs.insert(
            EntityId(id),
            Bomb::new(EntityId(id), source, EntityId(2), 2, Tick(0), trigger).unwrap(),
        );
        state
            .step(&[input(InputAction::Press(Direction::East))])
            .unwrap();
        assert!(!state.world.bombs.contains_key(&EntityId(id)));
        assert_eq!(state.actors[0].last_successful_punch_tick(), None);
    }

    let mut remote = state(source);
    remote.world.bombs.insert(
        EntityId(11),
        Bomb::new(
            EntityId(11),
            source,
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Remote,
        )
        .unwrap(),
    );
    remote
        .step(&[
            input(InputAction::Press(Direction::East)),
            input(InputAction::PlaceBomb),
        ])
        .unwrap();
    assert!(!remote.world.bombs.contains_key(&EntityId(11)));
    assert_eq!(remote.actors[0].last_successful_punch_tick(), None);
    assert!(remote.actors[0].contact_punch_latches.is_empty());
}

#[test]
fn frozen_actor_cannot_contact_punch() {
    let source = CellId::new(5, 5);
    let mut state = state(source);
    state.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            source,
            EntityId(2),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap(),
    );
    state.actors[1]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();

    state
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert!(state.actors[0].contact_punch_latches.is_empty());
}

#[test]
fn failed_contact_latch_is_rolled_back_with_the_transaction() {
    let source = CellId::new(5, 5);
    let position = tangent_west(source);
    let blocks = (1..ARENA_WIDTH - 1)
        .map(|column| CellId::new(column, source.row))
        .filter(|cell| *cell != source && *cell != position.cell().unwrap())
        .collect();
    let world = World::with_pickups(
        arena(blocks),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(position.cell().unwrap(), PickupKind::Shield)]),
    )
    .unwrap();
    let enabled =
        EnabledItems::from_bits(EnabledItems::PUNCH.bits() | EnabledItems::SHIELD.bits()).unwrap();
    let mut state = State::new_at_tick(
        rules(enabled),
        world,
        actors(position),
        Tick(u64::MAX - 300),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            source,
            EntityId(2),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(u64::MAX - 1),
            },
        )
        .unwrap(),
    );
    let before = state.clone();

    assert_eq!(
        state.step(&[input(InputAction::Press(Direction::East))]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(state, before);
}
