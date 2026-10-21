use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use super::{ActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Direction, EffectKind, EnabledItems, EntityId, Equipment, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, StepError, Tick, World,
};
use crate::room::RoomSettings;

fn arena(blocks: BTreeSet<crate::game::CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row == ARENA_HEIGHT - 1 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        blocks,
        [
            crate::game::CellId::new(1, 1),
            crate::game::CellId::new(13, 11),
        ],
    )
    .unwrap()
}

fn state(one: Position, two: Position, blocks: BTreeSet<crate::game::CellId>) -> State {
    let mut state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::BRACELET)),
        World::new(arena(blocks)),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: one,
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: two,
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    state
}

fn press(seat: SeatId, direction: Direction) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::Press(direction),
    }
}

#[test]
fn forward_positive_area_entry_shoves_one_cell_and_records_a_separate_segment() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position {
        x: one.x + 640,
        y: one.y,
    };
    let mut state = state(one, two, BTreeSet::new());

    let report = state.step(&[press(SeatId::One, Direction::East)]).unwrap();

    let forced = report
        .forced_movement
        .expect("the contact produces a shove");
    assert_eq!(forced.seat, SeatId::Two);
    assert_eq!(forced.segment.start(), two);
    assert_eq!(
        forced.segment.end(),
        Position {
            x: two.x + 1_024,
            y: two.y,
        }
    );
    assert_eq!(state.actors[1].position, forced.segment.end());
    assert_eq!(state.actors[0].last_successful_shove_tick(), Some(Tick(1)));
    assert_eq!(report.traces[0].len(), 1);
    assert!(report.traces[1].is_empty());
}

#[test]
fn tangent_stationary_and_correction_only_motion_do_not_shove() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position {
        x: one.x + 640,
        y: one.y,
    };
    let mut stationary = state(one, two, BTreeSet::new());
    assert!(stationary.step(&[]).unwrap().forced_movement.is_none());

    let mut correction = state(
        Position {
            x: one.x,
            y: one.y - 20,
        },
        two,
        BTreeSet::new(),
    );
    correction.actors[0].heading = Direction::North;
    assert!(
        correction
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_none()
    );
}

#[test]
fn transverse_side_entry_does_not_arm_but_longitudinal_leading_face_entry_does() {
    let holder = Position { x: 3_988, y: 4_096 };
    let mut side_entry = state(holder, Position { x: 4_608, y: 4_746 }, BTreeSet::new());
    let side_report = side_entry
        .step(&[
            press(SeatId::One, Direction::East),
            press(SeatId::Two, Direction::North),
        ])
        .unwrap();
    assert!(side_report.forced_movement.is_none());
    assert_eq!(side_entry.actors[0].last_successful_shove_tick(), None);

    let mut leading_entry = state(holder, Position { x: 4_638, y: 4_716 }, BTreeSet::new());
    let leading_report = leading_entry
        .step(&[
            press(SeatId::One, Direction::East),
            press(SeatId::Two, Direction::North),
        ])
        .unwrap();
    assert_eq!(leading_report.forced_movement.unwrap().seat, SeatId::Two);
    assert_eq!(
        leading_entry.actors[0].last_successful_shove_tick(),
        Some(Tick(1))
    );
}

#[test]
fn victim_collision_capability_controls_partial_clamp() {
    let one = Position { x: 4_220, y: 4_608 };
    let two = Position {
        x: one.x + 640,
        y: one.y,
    };
    let block = crate::game::CellId::new(6, 4);
    let mut state = state(one, two, BTreeSet::from([block]));

    let report = state.step(&[press(SeatId::One, Direction::East)]).unwrap();
    let forced = report
        .forced_movement
        .expect("room before the block remains");

    assert!(forced.segment.end().x > two.x);
    assert!(forced.segment.end().x < two.x + 1_024);
    assert_eq!(state.actors[0].last_successful_shove_tick(), Some(Tick(1)));
}

#[test]
fn initial_overlap_must_separate_before_a_later_entry_can_shove() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position {
        x: one.x + 500,
        y: one.y,
    };
    let mut state = state(one, two, BTreeSet::new());
    assert!(
        state
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_none()
    );
}

#[test]
fn all_cardinal_axes_use_the_actual_forward_segment() {
    let center = Position { x: 5_632, y: 5_632 };
    for (direction, victim, expected) in [
        (
            Direction::East,
            Position { x: 6_272, y: 5_632 },
            Position { x: 7_296, y: 5_632 },
        ),
        (
            Direction::West,
            Position { x: 4_992, y: 5_632 },
            Position { x: 3_968, y: 5_632 },
        ),
        (
            Direction::South,
            Position { x: 5_632, y: 6_272 },
            Position { x: 5_632, y: 7_296 },
        ),
        (
            Direction::North,
            Position { x: 5_632, y: 4_992 },
            Position { x: 5_632, y: 3_968 },
        ),
    ] {
        let mut state = state(center, victim, BTreeSet::new());
        let report = state.step(&[press(SeatId::One, direction)]).unwrap();
        assert_eq!(report.forced_movement.unwrap().segment.end(), expected);
    }
}

#[test]
fn competing_entries_choose_round_priority_before_collision_clamping() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position {
        x: one.x + 640,
        y: one.y,
    };
    for (round, expected_victim) in [(1, SeatId::Two), (2, SeatId::One)] {
        let mut state = state(one, two, BTreeSet::new());
        state.assignment = crate::game::RoundAssignment::new(NonZeroU32::new(round).unwrap());
        state.replace_actor_equipment(SeatId::Two, Equipment::Bracelet);
        let report = state
            .step(&[
                press(SeatId::Two, Direction::West),
                press(SeatId::One, Direction::East),
            ])
            .unwrap();
        assert_eq!(report.forced_movement.unwrap().seat, expected_victim);
    }
}

#[test]
fn cooldown_blocks_through_t_plus_29_and_allows_a_new_entry_at_t_plus_30() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position {
        x: one.x + 640,
        y: one.y,
    };
    let mut state = state(one, two, BTreeSet::new());
    assert!(
        state
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_some()
    );
    for tick in 2..=30 {
        state.actors[0].position = one;
        state.actors[1].position = two;
        let report = state.step(&[]).unwrap();
        assert!(
            report.forced_movement.is_none(),
            "tick {tick} is still cooling down"
        );
    }
    state.actors[0].position = one;
    state.actors[1].position = two;
    assert!(state.step(&[]).unwrap().forced_movement.is_some());
    assert_eq!(state.actors[0].last_successful_shove_tick(), Some(Tick(31)));
}

#[test]
fn late_pickup_overflow_rolls_back_shove_history_forced_travel_and_input() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position {
        x: one.x + 640,
        y: one.y,
    };
    let enabled =
        EnabledItems::from_bits(EnabledItems::BRACELET.bits() | EnabledItems::SHIELD.bits())
            .unwrap();
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(crate::game::CellId::new(5, 4), PickupKind::Shield)]),
    )
    .unwrap();
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: one,
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: two,
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(u64::MAX - 1),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    let before = state.clone();

    assert_eq!(
        state.step(&[press(SeatId::One, Direction::East)]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(state, before);
}

#[test]
fn malformed_shove_history_precedes_unsupported_equipment_action() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position { x: 8_192, y: 8_192 };
    let mut state = state(one, two, BTreeSet::new());
    state.actors[0].restore_shove_cooldown(Some(Tick(0)));

    assert_eq!(
        state.step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::UseEquipment,
        }]),
        Err(StepError::UnsupportedShoveHistory(EntityId(1)))
    );
}
