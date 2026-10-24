use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, RoundResult, SeatInput, State, StateError, StepError};
use super::MAX_INPUTS_PER_TICK;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Position, RulesConfig, SeatId,
    SpeedLevel, Tick, World,
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

fn punched_bomb() -> Bomb {
    let mut bomb = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    bomb.restore_punch_history(true);
    bomb
}

fn punched_contact_bomb() -> Bomb {
    let mut bomb = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    bomb.restore_punch_history(true);
    bomb
}

fn punch_and_landmine_rules() -> RulesConfig {
    let enabled =
        EnabledItems::from_bits(EnabledItems::PUNCH.bits() | EnabledItems::LANDMINE.bits())
            .unwrap();
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

#[test]
fn unsupported_punch_history_is_validated_before_terminal_short_circuit() {
    let mut world = World::new(arena());
    let bomb = punched_bomb();
    world.bombs.insert(bomb.id(), bomb);
    assert_eq!(
        State::new(RulesConfig::classic_v1(), world, actors()),
        Err(StateError::UnsupportedBombPunchHistory(EntityId(9)))
    );

    let rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::PUNCH));
    let mut world = World::new(arena());
    let bomb = punched_bomb();
    world.bombs.insert(bomb.id(), bomb);
    let mut state = State::new(rules, world, actors()).unwrap();
    state.result = Some(RoundResult::Winner(SeatId::One));
    state.rules = RulesConfig::classic_v1();
    assert_eq!(
        state.step(&[]),
        Err(StepError::UnsupportedBombPunchHistory(EntityId(9)))
    );
}

#[test]
fn constructor_rejects_punched_contact_history_when_both_items_are_enabled() {
    let mut world = World::new(arena());
    let bomb = punched_contact_bomb();
    world.bombs.insert(bomb.id(), bomb);

    assert_eq!(
        State::new(punch_and_landmine_rules(), world, actors()),
        Err(StateError::UnsupportedBombPunchHistory(EntityId(9)))
    );
}

#[test]
fn step_preflights_punched_contact_history_atomically_with_existing_precedence() {
    let mut world = World::new(arena());
    let bomb = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    world.bombs.insert(bomb.id(), bomb);
    let mut state = State::new(punch_and_landmine_rules(), world, actors()).unwrap();
    state.result = Some(RoundResult::Winner(SeatId::One));
    state
        .world
        .bombs
        .get_mut(&EntityId(9))
        .unwrap()
        .restore_punch_history(true);
    let before = state.clone();
    let clear = SeatInput {
        seat: SeatId::One,
        action: InputAction::Clear,
    };

    assert_eq!(
        state.step(&[clear; MAX_INPUTS_PER_TICK + 1]),
        Err(StepError::TooManyInputs {
            count: MAX_INPUTS_PER_TICK + 1,
            maximum: MAX_INPUTS_PER_TICK,
        })
    );
    assert_eq!(state, before);

    state.tick = Tick(u64::MAX);
    let before_overflow = state.clone();
    assert_eq!(state.step(&[]), Err(StepError::TickOverflow));
    assert_eq!(state, before_overflow);

    state.tick = Tick(0);
    let before_history = state.clone();
    assert_eq!(
        state.step(&[]),
        Err(StepError::UnsupportedBombPunchHistory(EntityId(9)))
    );
    assert_eq!(state, before_history);
}

#[test]
fn malformed_kick_history_keeps_priority_over_malformed_punch_history() {
    let mut world = World::new(arena());
    let mut bomb = punched_contact_bomb();
    bomb.restore_kick_history(None, true);
    world.bombs.insert(bomb.id(), bomb);
    let enabled = EnabledItems::from_bits(
        EnabledItems::PUNCH.bits() | EnabledItems::LANDMINE.bits() | EnabledItems::KICK.bits(),
    )
    .unwrap();

    assert_eq!(
        State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
            world,
            actors(),
        ),
        Err(StateError::UnsupportedBombKickHistory(EntityId(9)))
    );
}

#[test]
fn raw_equipment_rejection_preserves_existing_step_validation_precedence() {
    let equipment = SeatInput {
        seat: SeatId::One,
        action: InputAction::UseEquipment,
    };
    let mut state = State::new(RulesConfig::classic_v1(), World::new(arena()), actors()).unwrap();
    assert_eq!(
        state.step(&[equipment; MAX_INPUTS_PER_TICK + 1]),
        Err(StepError::TooManyInputs {
            count: MAX_INPUTS_PER_TICK + 1,
            maximum: MAX_INPUTS_PER_TICK,
        })
    );
    state.tick = Tick(u64::MAX);
    assert_eq!(state.step(&[equipment]), Err(StepError::TickOverflow));

    let rules = RulesConfig::expanded_v5(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::REMOTE_BOMB),
    );
    let mut world = World::new(arena());
    let remote = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    world.bombs.insert(remote.id(), remote);
    let mut state = State::new(rules, world, actors()).unwrap();
    state.rules = RulesConfig::classic_v1();
    assert_eq!(
        state.step(&[equipment]),
        Err(StepError::UnsupportedBombTrigger(EntityId(9)))
    );
}
