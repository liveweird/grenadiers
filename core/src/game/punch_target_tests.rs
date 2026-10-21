use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, SeatInput, State, StepError};
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

fn state(position: Position, heading: Direction, enabled: EnabledItems) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena()),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position,
                heading,
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
    )
    .unwrap()
}

fn bomb(cell: CellId) -> Bomb {
    Bomb::new(
        EntityId(9),
        cell,
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap()
}

fn equipment() -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::UseEquipment,
    }
}

#[test]
fn every_direction_uses_the_fractional_actor_center_cell_and_signed_wrapped_stride() {
    let center = CellId::new(6, 6);
    let fractional = Position {
        x: Position::center(center).x + 111,
        y: Position::center(center).y - 97,
    };
    for (heading, source, landing) in [
        (
            Direction::North,
            CellId::new(6, 5),
            CellId::new(6, 2),
        ),
        (
            Direction::East,
            CellId::new(7, 6),
            CellId::new(10, 6),
        ),
        (
            Direction::South,
            CellId::new(6, 7),
            CellId::new(6, 10),
        ),
        (
            Direction::West,
            CellId::new(5, 6),
            CellId::new(2, 6),
        ),
    ] {
        let mut state = state(fractional, heading, EnabledItems::PUNCH);
        state.replace_actor_equipment(SeatId::One, Equipment::Punch);
        state.world.bombs.insert(EntityId(9), bomb(source));

        state.step(&[equipment()]).unwrap();

        assert_eq!(state.world.bombs[&EntityId(9)].cell(), landing);
    }
}

#[test]
fn movement_updates_authoritative_heading_before_equipment_resolution() {
    let actor = CellId::new(5, 5);
    let mut state = state(
        Position::center(actor),
        Direction::East,
        EnabledItems::PUNCH,
    );
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(CellId::new(5, 4)));

    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::North),
            },
            equipment(),
        ])
        .unwrap();

    assert_eq!(state.actors[0].heading(), Direction::North);
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 1));
}

#[test]
fn reverse_controls_changes_movement_heading_once_and_equipment_does_not_invert_again() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::PUNCH.bits() | EnabledItems::REVERSE_CONTROLS.bits(),
    )
    .unwrap();
    let mut state = state(
        Position::center(CellId::new(5, 5)),
        Direction::East,
        enabled,
    );
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.apply_test_effect(SeatId::One, EffectKind::ReverseControls, Tick(0));
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(CellId::new(4, 5)));

    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
            equipment(),
        ])
        .unwrap();

    assert_eq!(state.actors[0].heading(), Direction::West);
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(1, 5));
}

#[test]
fn wrong_gear_and_stopwatch_freeze_are_gameplay_no_ops() {
    let cases = [
        (EnabledItems::PUNCH, Some(Equipment::Kick), false),
        (
            EnabledItems::from_bits(
                EnabledItems::PUNCH.bits() | EnabledItems::STOPWATCH.bits(),
            )
            .unwrap(),
            Some(Equipment::Punch),
            true,
        ),
    ];
    for (enabled, equipment_kind, frozen) in cases {
        let mut state = state(
            Position::center(CellId::new(4, 5)),
            Direction::East,
            enabled,
        );
        if let Some(equipment_kind) = equipment_kind {
            state.replace_actor_equipment(SeatId::One, equipment_kind);
        }
        if frozen {
            state.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
        }
        state
            .world
            .bombs
            .insert(EntityId(9), bomb(CellId::new(5, 5)));

        state.step(&[equipment()]).unwrap();

        assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 5));
        assert_eq!(state.actors[0].last_successful_punch_tick(), None);
    }
}

#[test]
fn raw_equipment_input_is_rejected_when_punch_is_disabled() {
    let mut state = state(
        Position::center(CellId::new(4, 5)),
        Direction::East,
        EnabledItems::EMPTY,
    );
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(CellId::new(5, 5)));
    let before = state.clone();

    assert_eq!(
        state.step(&[equipment()]),
        Err(StepError::UnsupportedEquipmentAction)
    );
    assert_eq!(state, before);
}

#[test]
fn target_selection_does_not_wrap_at_the_arena_edge() {
    let mut state = state(
        Position::center(CellId::new(13, 5)),
        Direction::East,
        EnabledItems::PUNCH,
    );
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(CellId::new(1, 5)));

    state.step(&[equipment()]).unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(1, 5));
}

#[test]
fn exhausting_the_stride_cycle_and_returning_to_source_fails_unchanged() {
    let mut state = state(
        Position::center(CellId::new(4, 5)),
        Direction::East,
        EnabledItems::PUNCH,
    );
    let blocked = [1, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13]
        .map(|column| CellId::new(column, 5));
    state.world.arena = {
        let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
        for row in 0..ARENA_HEIGHT {
            for column in 0..ARENA_WIDTH {
                if column == 0
                    || row == 0
                    || column == ARENA_WIDTH - 1
                    || row == ARENA_HEIGHT - 1
                {
                    terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                        Terrain::Wall;
                }
            }
        }
        Arena::new(
            terrain,
            BTreeSet::from(blocked),
            [CellId::new(1, 1), CellId::new(13, 11)],
        )
        .unwrap()
    };
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(CellId::new(5, 5)));

    state.step(&[equipment()]).unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 5));
    assert_eq!(state.actors[0].last_successful_punch_tick(), None);
}

#[test]
fn positive_living_footprint_overlap_skips_a_landing_candidate() {
    let mut state = state(
        Position::center(CellId::new(11, 5)),
        Direction::East,
        EnabledItems::PUNCH,
    );
    state.world.arena = {
        let mut open = arena();
        open.blocks.insert(CellId::new(2, 5));
        open
    };
    state.world.bombs.insert(
        EntityId(10),
        Bomb::new(
            EntityId(10),
            CellId::new(5, 5),
            EntityId(2),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap(),
    );
    let candidate = Position::center(CellId::new(8, 5));
    state.actors[1].position = Position {
        x: candidate.x + crate::game::types::UNITS_PER_CELL / 2
            + crate::game::types::HALF_FOOTPRINT
            - 1,
        y: candidate.y,
    };
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(CellId::new(12, 5)));

    state.step(&[equipment()]).unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(1, 5));
}
