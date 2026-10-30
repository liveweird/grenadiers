use std::collections::BTreeSet;
use std::num::NonZeroU32;

use super::{ActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, InputAction, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, StateError,
    StormCloudConfig, Tick, World, player_collision,
};
use crate::room::{RespawnLairConfig, RoomSettings};

fn arena(count: usize) -> Arena {
    const SPAWNS: [CellId; 4] = [
        CellId::new(1, 1),
        CellId::new(13, 1),
        CellId::new(1, 11),
        CellId::new(13, 11),
    ];
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
        BTreeSet::new(),
        if count == 2 {
            vec![SPAWNS[0], SPAWNS[3]]
        } else {
            SPAWNS[..count].to_vec()
        },
    )
    .unwrap()
}

fn actor(index: usize, position: Position, heading: Direction) -> ActorInit {
    ActorInit {
        id: EntityId(index as u32 + 1),
        seat: SeatId::ALL[index],
        position,
        heading,
        speed: SpeedLevel::Base,
    }
}

fn rules(items: EnabledItems) -> RulesConfig {
    RulesConfig::environment_v9(
        RoomSettings::DEFAULT.with_enabled_items(items),
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
    )
}

fn state(positions: &[(Position, Direction)], items: EnabledItems) -> State {
    State::new(
        rules(items),
        World::new(arena(positions.len())),
        positions
            .iter()
            .enumerate()
            .map(|(index, &(position, heading))| actor(index, position, heading))
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn press(seat: SeatId, direction: Direction) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::Press(direction),
    }
}

#[test]
fn constructor_rejects_same_cell_and_overlapping_footprints() {
    let same_cell = State::new(
        rules(EnabledItems::EMPTY),
        World::new(arena(2)),
        [
            actor(0, Position { x: 4_100, y: 4_500 }, Direction::East),
            actor(1, Position { x: 4_800, y: 4_500 }, Direction::West),
        ],
    );
    assert_eq!(
        same_cell,
        Err(StateError::ConflictingActorPositions {
            first: SeatId::One,
            second: SeatId::Two,
        })
    );

    let overlap = State::new(
        rules(EnabledItems::EMPTY),
        World::new(arena(2)),
        [
            actor(0, Position { x: 4_800, y: 4_500 }, Direction::East),
            actor(1, Position { x: 5_400, y: 4_500 }, Direction::West),
        ],
    );
    assert!(matches!(
        overlap,
        Err(StateError::ConflictingActorPositions { .. })
    ));
}

#[test]
fn head_on_motion_clips_before_overlap_and_is_input_order_independent() {
    let positions = [
        (Position { x: 1_500, y: 1_500 }, Direction::East),
        (Position { x: 2_141, y: 1_500 }, Direction::West),
    ];
    let mut forward = state(&positions, EnabledItems::EMPTY);
    forward.actors[0].speed = SpeedLevel::Four;
    forward.actors[1].speed = SpeedLevel::Four;
    let mut reversed = forward.clone();
    forward
        .step(&[
            press(SeatId::One, Direction::East),
            press(SeatId::Two, Direction::West),
        ])
        .unwrap();
    reversed
        .step(&[
            press(SeatId::Two, Direction::West),
            press(SeatId::One, Direction::East),
        ])
        .unwrap();

    assert_eq!(forward.actors(), reversed.actors());
    assert_eq!(forward.actors[0].position, Position { x: 1_501, y: 1_500 });
    assert_eq!(forward.actors[1].position, Position { x: 2_141, y: 1_500 });
    assert!(!player_collision::player_positions_conflict(
        forward.actors[0].position,
        forward.actors[1].position,
    ));
}

#[test]
fn vacated_center_cell_remains_reserved_until_the_next_tick() {
    let positions = [
        (Position { x: 2_047, y: 4_500 }, Direction::East),
        (Position { x: 3_071, y: 4_500 }, Direction::East),
    ];
    let mut game = State::new_at_tick_with_assignment(
        rules(EnabledItems::EMPTY),
        World::new(arena(2)),
        positions
            .iter()
            .enumerate()
            .map(|(index, &(position, heading))| actor(index, position, heading))
            .collect::<Vec<_>>(),
        Tick(0),
        RoundAssignment::new(NonZeroU32::new(2).unwrap()),
    )
    .unwrap();

    game.step(&[
        press(SeatId::One, Direction::East),
        press(SeatId::Two, Direction::East),
    ])
    .unwrap();
    assert_eq!(game.actors[0].position.x, 2_047);
    assert_eq!(game.actors[1].position.x, 3_122);

    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position.x, 2_098);
}

#[test]
fn perpendicular_turn_commits_correction_but_clips_forward_leg_at_tangency() {
    let mut game = state(
        &[
            (Position { x: 4_658, y: 4_000 }, Direction::East),
            (Position { x: 4_608, y: 4_640 }, Direction::West),
        ],
        EnabledItems::EMPTY,
    );

    let report = game.step(&[press(SeatId::One, Direction::South)]).unwrap();

    assert_eq!(game.actors[0].position, Position { x: 4_608, y: 4_000 });
    assert_eq!(report.traces[0].len(), 1);
    assert!(!player_collision::player_positions_conflict(
        game.actors[0].position,
        game.actors[1].position,
    ));
}

#[test]
fn frozen_cloaked_and_wall_pass_actors_obey_the_same_solid_boundary() {
    let positions = [
        (Position { x: 1_800, y: 4_500 }, Direction::East),
        (Position { x: 2_440, y: 4_500 }, Direction::West),
    ];
    let mut frozen = state(&positions, EnabledItems::STOPWATCH);
    frozen.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));
    frozen.step(&[press(SeatId::One, Direction::East)]).unwrap();
    assert_eq!(frozen.actors[0].position.x, 1_800);

    let mut cloaked = state(&positions, EnabledItems::INVISIBILITY_CLOAK);
    cloaked.apply_test_effect(SeatId::Two, EffectKind::InvisibilityCloak, Tick(0));
    cloaked
        .step(&[press(SeatId::One, Direction::East)])
        .unwrap();
    assert_eq!(cloaked.actors[0].position.x, 1_800);

    let mut wall_pass = state(&positions, EnabledItems::WALL_PASS);
    wall_pass.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    wall_pass
        .step(&[press(SeatId::One, Direction::East)])
        .unwrap();
    assert_eq!(wall_pass.actors[0].position.x, 1_800);
}

#[test]
fn fourth_seat_wins_a_contested_empty_cell_in_round_four_priority() {
    let positions = [
        (Position { x: 4_096, y: 4_500 }, Direction::West),
        (Position { x: 7_000, y: 4_500 }, Direction::West),
        (Position { x: 9_000, y: 4_500 }, Direction::West),
        (Position { x: 3_071, y: 4_500 }, Direction::East),
    ];
    let mut game = State::new_at_tick_with_assignment(
        rules(EnabledItems::EMPTY),
        World::new(arena(4)),
        positions
            .iter()
            .enumerate()
            .map(|(index, &(position, heading))| actor(index, position, heading))
            .collect::<Vec<_>>(),
        Tick(0),
        RoundAssignment::new(NonZeroU32::new(4).unwrap()),
    )
    .unwrap();

    game.step(&[
        press(SeatId::One, Direction::West),
        press(SeatId::Four, Direction::East),
    ])
    .unwrap();

    assert_eq!(game.actors[3].position.x, 3_122);
    assert_eq!(game.actors[0].position.x, 4_096);
}

#[test]
fn dead_actor_does_not_reserve_a_footprint_or_cell() {
    let mut game = state(
        &[
            (Position { x: 1_500, y: 1_500 }, Direction::East),
            (Position { x: 2_140, y: 1_500 }, Direction::West),
        ],
        EnabledItems::EMPTY,
    );
    game.actors[1].alive = false;
    game.step(&[press(SeatId::One, Direction::East)]).unwrap();
    assert_eq!(game.actors[0].position.x, 1_551);
}

#[test]
fn zero_distance_bracelet_contact_blocked_by_third_actor_spends_no_cooldown() {
    let mut game = state(
        &[
            (Position { x: 1_800, y: 1_500 }, Direction::East),
            (Position { x: 2_440, y: 1_500 }, Direction::East),
            (Position { x: 3_080, y: 1_500 }, Direction::West),
        ],
        EnabledItems::BRACELET,
    );
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    let report = game.step(&[press(SeatId::One, Direction::East)]).unwrap();

    assert!(report.forced_movements.is_empty());
    assert_eq!(game.actors[0].position, Position { x: 1_800, y: 1_500 });
    assert_eq!(game.actors[0].last_successful_shove_tick(), None);
}

#[test]
fn shove_keeps_a_higher_priority_actors_vacated_start_cell_reserved() {
    let positions = [
        (Position { x: 2_047, y: 4_500 }, Direction::East),
        (Position { x: 2_687, y: 4_500 }, Direction::East),
        (Position { x: 4_095, y: 4_500 }, Direction::East),
    ];
    let mut game = State::new_at_tick_with_assignment(
        rules(EnabledItems::BRACELET),
        World::new(arena(3)),
        positions
            .iter()
            .enumerate()
            .map(|(index, &(position, heading))| actor(index, position, heading))
            .collect::<Vec<_>>(),
        Tick(0),
        RoundAssignment::new(NonZeroU32::new(3).unwrap()),
    )
    .unwrap();
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);

    let report = game
        .step(&[
            press(SeatId::One, Direction::East),
            press(SeatId::Three, Direction::East),
        ])
        .unwrap();

    assert_eq!(game.actors[2].position.x, 4_146);
    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].seat, SeatId::Two);
    assert_eq!(report.forced_movements[0].segment.end().x, 3_071);
    assert_eq!(game.actors[1].position.x, 3_071);
}

#[test]
fn teleporter_swap_is_atomic_when_a_third_actor_blocks_one_destination() {
    let positions = [
        (Position { x: 1_500, y: 1_500 }, Direction::East),
        (Position { x: 4_500, y: 1_500 }, Direction::West),
        (Position { x: 7_500, y: 1_500 }, Direction::West),
        (Position { x: 9_500, y: 1_500 }, Direction::West),
    ];
    let mut game = State::new_at_tick_with_assignment_and_seed(
        rules(EnabledItems::TELEPORTER),
        World::new(arena(4)),
        positions
            .iter()
            .enumerate()
            .map(|(index, &(position, heading))| actor(index, position, heading))
            .collect::<Vec<_>>(),
        Tick(0),
        RoundAssignment::new(NonZeroU32::new(1).unwrap()),
        SimulationSeed::new([0; 32]),
    )
    .unwrap();
    let before = [game.actors[0].position, game.actors[1].position];
    game.actors[2].position = before[1];

    game.commit_teleport([true, false, false, false], Tick(1));

    assert_eq!([game.actors[0].position, game.actors[1].position], before);
    assert_eq!(game.last_teleport_tick, None);
}
