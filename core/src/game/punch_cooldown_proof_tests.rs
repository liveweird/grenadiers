use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, RoundResult, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, Position,
    RulesConfig, SeatId, SpeedLevel, Tick, World,
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

fn state_at(tick: Tick) -> State {
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL)),
        World::new(arena()),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(4, 5)),
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
        ],
        tick,
    )
    .unwrap()
}

fn bomb(id: u32, cell: CellId, detonates_at: Tick) -> Bomb {
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

fn use_punch() -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::UseEquipment,
    }
}

#[test]
fn cooldown_uses_physical_ticks_that_elapsed_during_stopwatch_freeze() {
    let mut state = state_at(Tick(170));
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(9, CellId::new(5, 5), Tick(500)));
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.step(&[use_punch()]).unwrap();
    assert_eq!(
        state.actors[0].last_successful_punch_tick(),
        Some(Tick(171))
    );
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));

    state.actors[0].position = Position::center(CellId::new(9, 5));
    state.actors[0].heading = Direction::West;
    state.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
    while state.tick() < Tick(182) {
        state.step(&[use_punch()]).unwrap();
        assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
    }
    state.step(&[use_punch()]).unwrap();

    assert_eq!(state.tick(), Tick(183));
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 5));
    assert_eq!(
        state.actors[0].last_successful_punch_tick(),
        Some(Tick(183))
    );
}

#[test]
fn successful_tick_survives_gear_replacement_death_and_result_then_new_state_resets_it() {
    let mut state = state_at(Tick(0));
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(9, CellId::new(5, 5), Tick(100)));
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.step(&[use_punch()]).unwrap();
    assert_eq!(state.actors[0].last_successful_punch_tick(), Some(Tick(1)));

    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    assert_eq!(state.actors[0].last_successful_punch_tick(), Some(Tick(1)));
    state.world.hazards.insert(CellId::new(4, 5), Tick(10));
    state.step(&[]).unwrap();

    assert!(!state.actors[0].alive());
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
    assert_eq!(state.actors[0].last_successful_punch_tick(), Some(Tick(1)));

    let reset = state_at(Tick(0));
    assert_eq!(reset.actors[0].last_successful_punch_tick(), None);
    assert_eq!(reset.actors[1].last_successful_punch_tick(), None);
}
