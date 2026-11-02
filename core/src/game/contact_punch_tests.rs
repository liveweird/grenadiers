use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombMotion, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, Position,
    PunchActivationPolicy, RulesConfig, SeatId, SpeedLevel, Tick, World,
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

fn rules(policy: PunchActivationPolicy) -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL))
        .with_punch_activation_policy(policy)
}

fn blocked_east_landings(source: CellId, occupied: CellId) -> BTreeSet<CellId> {
    (1..ARENA_WIDTH - 1)
        .map(|column| CellId::new(column, source.row))
        .filter(|cell| *cell != source && *cell != occupied)
        .collect()
}

fn state_at(position: Position, policy: PunchActivationPolicy, tick: Tick) -> State {
    let mut state = State::new_at_tick(
        rules(policy),
        World::new(arena(BTreeSet::new())),
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
        ],
        tick,
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
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

fn input(action: InputAction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action,
    }
}

fn tangent_west(cell: CellId) -> Position {
    Position {
        x: i32::from(cell.column) * crate::game::types::UNITS_PER_CELL
            - crate::game::types::HALF_FOOTPRINT,
        y: Position::center(cell).y,
    }
}

#[test]
fn contact_v2_punches_when_positive_forward_travel_ends_exactly_tangent() {
    let source = CellId::new(5, 5);
    let tangent = tangent_west(source);
    let mut state = state_at(
        Position {
            x: tangent.x - 51,
            ..tangent
        },
        PunchActivationPolicy::ContactV2,
        Tick(0),
    );
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));

    state
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();

    assert_eq!(state.actors[0].position, tangent);
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
    assert_eq!(state.actors[0].last_successful_punch_tick(), Some(Tick(1)));
}

#[test]
fn manual_v1_keeps_contact_inert_and_explicit_equipment_behavior_unchanged() {
    let source = CellId::new(5, 5);
    let tangent = tangent_west(source);
    let mut state = state_at(
        Position {
            x: tangent.x - 51,
            ..tangent
        },
        PunchActivationPolicy::ManualV1,
        Tick(0),
    );
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));

    state
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    state.step(&[input(InputAction::UseEquipment)]).unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
}

#[test]
fn failed_landing_is_latched_until_separation_not_key_or_heading_changes() {
    let source = CellId::new(5, 5);
    let tangent = tangent_west(source);
    let mut state = state_at(tangent, PunchActivationPolicy::ContactV2, Tick(0));
    state.world.arena = arena(blocked_east_landings(source, CellId::new(4, 5)));
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));

    state
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9)])
    );

    state.world.arena = arena(BTreeSet::new());
    state
        .step(&[input(InputAction::Release(Direction::East))])
        .unwrap();
    state.actors[0].heading = Direction::North;
    state.step(&[]).unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9)])
    );

    state
        .step(&[input(InputAction::Press(Direction::West))])
        .unwrap();
    assert!(state.actors[0].contact_punch_latches.is_empty());
    state
        .step(&[
            input(InputAction::Release(Direction::West)),
            input(InputAction::Press(Direction::East)),
        ])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
}

#[test]
fn cooldown_failure_is_one_attempt_for_the_uninterrupted_contact() {
    let source = CellId::new(5, 5);
    let mut state = state_at(
        tangent_west(source),
        PunchActivationPolicy::ContactV2,
        Tick(1),
    );
    state.actors[0].last_successful_punch_tick = Some(Tick(1));
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));

    for _ in 0..12 {
        state
            .step(&[input(InputAction::Press(Direction::East))])
            .unwrap();
    }
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9)])
    );

    state
        .step(&[
            input(InputAction::Release(Direction::East)),
            input(InputAction::Press(Direction::West)),
        ])
        .unwrap();
    state
        .step(&[
            input(InputAction::Release(Direction::West)),
            input(InputAction::Press(Direction::East)),
        ])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
}

#[test]
fn grace_departure_and_turn_correction_do_not_create_contact_punches() {
    let cell = CellId::new(5, 5);
    let mut grace = state_at(
        Position::center(cell),
        PunchActivationPolicy::ContactV2,
        Tick(0),
    );
    grace
        .world
        .bombs
        .insert(EntityId(9), timed(9, cell, Tick(100)));
    grace.actors[0].bomb_grace.insert(EntityId(9));
    for _ in 0..20 {
        grace
            .step(&[input(InputAction::Press(Direction::East))])
            .unwrap();
    }
    assert_eq!(grace.world.bombs[&EntityId(9)].cell(), cell);
    assert_eq!(grace.actors[0].last_successful_punch_tick(), None);

    let source = CellId::new(5, 5);
    let tangent = tangent_west(source);
    let mut correction = state_at(
        Position {
            y: tangent.y - 50,
            ..tangent
        },
        PunchActivationPolicy::ContactV2,
        Tick(0),
    );
    correction
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));
    correction
        .step(&[input(InputAction::Press(Direction::North))])
        .unwrap();
    assert_eq!(correction.world.bombs[&EntityId(9)].cell(), source);
    assert!(correction.actors[0].contact_punch_latches.is_empty());
}

#[test]
fn rolling_bomb_is_contact_punched_and_teleport_clears_a_failed_latch() {
    let source = CellId::new(5, 5);
    let mut rolling = state_at(
        tangent_west(source),
        PunchActivationPolicy::ContactV2,
        Tick(0),
    );
    let mut bomb = timed(9, source, Tick(100));
    bomb.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 3,
        }),
        true,
    );
    rolling.world.bombs.insert(EntityId(9), bomb);
    rolling
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(rolling.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));

    let mut failed = state_at(
        tangent_west(source),
        PunchActivationPolicy::ContactV2,
        Tick(0),
    );
    failed.world.arena = arena(blocked_east_landings(source, CellId::new(4, 5)));
    failed
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));
    failed
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(
        failed.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9)])
    );
    failed.world.arena = arena(BTreeSet::new());
    failed.commit_teleport([true, false], Tick(2));
    assert!(failed.actors[0].contact_punch_latches.is_empty());
}

#[test]
fn obstacle_tie_and_actor_death_do_not_leave_a_contact_attempt() {
    let source = CellId::new(5, 5);
    let tangent = tangent_west(source);
    let mut state = state_at(
        Position {
            y: i32::from(source.row) * crate::game::types::UNITS_PER_CELL,
            ..tangent
        },
        PunchActivationPolicy::ContactV2,
        Tick(0),
    );
    state.world.arena = arena(BTreeSet::from([CellId::new(5, 4)]));
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, Tick(100)));
    state
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert!(state.actors[0].contact_punch_latches.is_empty());

    state.world.arena = arena(blocked_east_landings(source, CellId::new(4, 5)));
    state
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9)])
    );
    state.actors[0].kill();
    assert!(state.actors[0].contact_punch_latches.is_empty());
}
