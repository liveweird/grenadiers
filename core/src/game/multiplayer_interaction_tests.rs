use std::collections::BTreeSet;
use std::num::NonZeroU32;

use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, PlayerCount, Position,
    RoundAssignment, RulesConfig, SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

fn arena(count: PlayerCount) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row == ARENA_HEIGHT - 1 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    let corners = [
        CellId::new(1, 1), CellId::new(13, 1),
        CellId::new(1, 11), CellId::new(13, 11),
    ];
    Arena::new(terrain, BTreeSet::new(), corners[..count.get()].to_vec()).unwrap()
}

fn state(enabled: EnabledItems, placements: [(Position, Direction); 4]) -> State {
    state_with_round(enabled, placements, 1)
}

fn state_with_round(
    enabled: EnabledItems,
    placements: [(Position, Direction); 4],
    round: u32,
) -> State {
    let actors: Vec<_> = SeatId::ALL.into_iter().enumerate().map(|(index, seat)| ActorInit {
        id: EntityId(10 + index as u32), seat,
        position: placements[index].0, heading: placements[index].1, speed: SpeedLevel::Base,
    }).collect();
    State::new_at_tick_with_assignment(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena(PlayerCount::Four)), actors, Tick(0),
        RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    ).unwrap()
}

fn state_three(
    enabled: EnabledItems,
    placements: [(Position, Direction); 3],
    round: u32,
) -> State {
    let actors: Vec<_> = PlayerCount::Three.seats().iter().enumerate().map(|(index, &seat)| {
        ActorInit {
            id: EntityId(10 + index as u32), seat, position: placements[index].0,
            heading: placements[index].1, speed: SpeedLevel::Base,
        }
    }).collect();
    State::new_at_tick_with_assignment(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena(PlayerCount::Three)), actors, Tick(0),
        RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    ).unwrap()
}

fn press(seat: SeatId, direction: Direction) -> SeatInput {
    SeatInput { seat, action: InputAction::Press(direction) }
}

fn use_equipment(seat: SeatId) -> SeatInput {
    SeatInput { seat, action: InputAction::UseEquipment }
}

fn bomb(id: u32, cell: CellId) -> Bomb {
    Bomb::new(EntityId(id), cell, EntityId(50), 2, Tick(0), BombTrigger::Timed {
        detonates_at: Tick(100),
    }).unwrap()
}

#[test]
fn independent_bracelet_pairs_commit_two_forced_segments_together() {
    let mut game = state(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 3_072 }, Direction::East),
        (Position { x: 4_736, y: 3_072 }, Direction::West),
        (Position { x: 4_096, y: 7_168 }, Direction::East),
        (Position { x: 4_736, y: 7_168 }, Direction::West),
    ]);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    game.replace_actor_equipment(SeatId::Three, Equipment::Bracelet);

    let report = game.step(&[
        press(SeatId::Three, Direction::East),
        press(SeatId::One, Direction::East),
    ]).unwrap();

    assert_eq!(report.forced_movements.len(), 2);
    assert_eq!(
        report
            .forced_movements
            .iter()
            .map(|forced| forced.pusher)
            .collect::<Vec<_>>(),
        [SeatId::One, SeatId::Three]
    );
    assert_eq!(report.forced_movements.iter().map(|forced| forced.seat).collect::<Vec<_>>(),
        [SeatId::Two, SeatId::Four]);
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));
    assert_eq!(game.actors[2].last_successful_shove_tick(), Some(Tick(1)));
}

#[test]
fn old_fire_checks_each_independent_forced_segment() {
    let mut game = state(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 3_072 }, Direction::East),
        (Position { x: 4_736, y: 3_072 }, Direction::West),
        (Position { x: 4_096, y: 7_168 }, Direction::East),
        (Position { x: 4_736, y: 7_168 }, Direction::West),
    ]);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    game.replace_actor_equipment(SeatId::Three, Equipment::Bracelet);
    game.insert_test_flame(CellId::new(5, 3), Tick(10));
    game.insert_test_flame(CellId::new(5, 7), Tick(10));

    let report = game.step(&[
        press(SeatId::One, Direction::East), press(SeatId::Three, Direction::East),
    ]).unwrap();

    assert_eq!(report.forced_movements.len(), 2);
    assert!(!game.actors[1].alive);
    assert!(!game.actors[3].alive);
}

#[test]
fn two_bracelet_pushers_contesting_one_victim_use_round_priority_once() {
    let mut game = state(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 4_096 }, Direction::East),
        (Position { x: 4_736, y: 4_096 }, Direction::West),
        (Position { x: 4_096, y: 4_096 }, Direction::East),
        (Position::center(CellId::new(13, 11)), Direction::West),
    ]);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    game.replace_actor_equipment(SeatId::Three, Equipment::Bracelet);

    let report = game.step(&[
        press(SeatId::Three, Direction::East),
        press(SeatId::One, Direction::East),
    ]).unwrap();

    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].seat, SeatId::Two);
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));
    assert_eq!(game.actors[2].last_successful_shove_tick(), None);
}

#[test]
fn one_bracelet_pusher_uses_victim_priority_for_tied_contact_time() {
    let mut game = state(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 4_096 }, Direction::East),
        (Position { x: 4_736, y: 3_596 }, Direction::West),
        (Position { x: 4_736, y: 4_596 }, Direction::West),
        (Position::center(CellId::new(13, 11)), Direction::West),
    ]);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);

    let report = game.step(&[press(SeatId::One, Direction::East)]).unwrap();

    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].seat, SeatId::Two);
}

#[test]
fn conflicting_independent_victim_sweeps_use_pusher_priority() {
    let mut game = state(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 4_096 }, Direction::East),
        (Position { x: 4_736, y: 4_096 }, Direction::West),
        (Position { x: 5_760, y: 2_432 }, Direction::South),
        (Position { x: 5_760, y: 3_072 }, Direction::North),
    ]);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    game.replace_actor_equipment(SeatId::Three, Equipment::Bracelet);

    let report = game.step(&[
        press(SeatId::One, Direction::East), press(SeatId::Three, Direction::South),
    ]).unwrap();

    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].pusher, SeatId::One);
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));
    assert_eq!(game.actors[2].last_successful_shove_tick(), None);
}

#[test]
fn three_punch_claims_for_one_bomb_choose_actual_round_priority_and_clear_all_grace() {
    let source = CellId::new(5, 5);
    let mut game = state(EnabledItems::PUNCH, [
        (Position::center(CellId::new(4, 5)), Direction::East),
        (Position::center(CellId::new(6, 5)), Direction::West),
        (Position::center(CellId::new(5, 4)), Direction::South),
        (Position::center(CellId::new(13, 11)), Direction::West),
    ]);
    game.world.bombs.insert(EntityId(90), bomb(90, source));
    for actor in &mut game.actors { actor.bomb_grace.insert(EntityId(90)); }
    for seat in [SeatId::One, SeatId::Two, SeatId::Three] {
        game.replace_actor_equipment(seat, Equipment::Punch);
    }

    game.step(&[
        use_equipment(SeatId::Three), use_equipment(SeatId::Two), use_equipment(SeatId::One),
    ]).unwrap();

    assert_eq!(game.world.bombs[&EntityId(90)].cell(), CellId::new(8, 5));
    assert_eq!(game.actors[0].last_successful_punch_tick(), Some(Tick(1)));
    assert!(game.actors[1..3].iter().all(|actor| actor.last_successful_punch_tick().is_none()));
    assert!(game.actors.iter().all(|actor| !actor.bomb_grace.contains(&EntityId(90))));
}

#[test]
fn three_distinct_punches_with_one_common_landing_are_all_rejected() {
    let mut game = state(EnabledItems::PUNCH, [
        (Position::center(CellId::new(1, 5)), Direction::East),
        (Position::center(CellId::new(9, 5)), Direction::West),
        (Position::center(CellId::new(5, 1)), Direction::South),
        (Position::center(CellId::new(13, 11)), Direction::West),
    ]);
    for (id, cell) in [(90, CellId::new(2, 5)), (91, CellId::new(8, 5)), (92, CellId::new(5, 2))] {
        game.world.bombs.insert(EntityId(id), bomb(id, cell));
    }
    for seat in [SeatId::One, SeatId::Two, SeatId::Three] {
        game.replace_actor_equipment(seat, Equipment::Punch);
    }

    game.step(&[use_equipment(SeatId::One), use_equipment(SeatId::Two), use_equipment(SeatId::Three)]).unwrap();

    assert_eq!(game.world.bombs[&EntityId(90)].cell(), CellId::new(2, 5));
    assert_eq!(game.world.bombs[&EntityId(91)].cell(), CellId::new(8, 5));
    assert_eq!(game.world.bombs[&EntityId(92)].cell(), CellId::new(5, 2));
    assert!(game.actors.iter().all(|actor| actor.last_successful_punch_tick().is_none()));
}

#[test]
fn three_player_step_integrates_punch_and_bracelet_in_one_update() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::PUNCH.bits() | EnabledItems::BRACELET.bits(),
    ).unwrap();
    let mut game = state_three(enabled, [
        (Position::center(CellId::new(4, 5)), Direction::East),
        (Position { x: 4_096, y: 7_168 }, Direction::East),
        (Position { x: 4_736, y: 7_168 }, Direction::West),
    ], 1);
    game.world.bombs.insert(EntityId(90), bomb(90, CellId::new(5, 5)));
    game.replace_actor_equipment(SeatId::One, Equipment::Punch);
    game.replace_actor_equipment(SeatId::Two, Equipment::Bracelet);

    let report = game.step(&[
        use_equipment(SeatId::One), press(SeatId::Two, Direction::East),
    ]).unwrap();

    assert_eq!(game.world.bombs[&EntityId(90)].cell(), CellId::new(8, 5));
    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].pusher, SeatId::Two);
    assert_eq!(report.forced_movements[0].seat, SeatId::Three);
}

#[test]
fn nonfirst_round_priority_controls_three_player_punch_and_bracelet_claims() {
    let mut punch = state_three(EnabledItems::PUNCH, [
        (Position::center(CellId::new(4, 5)), Direction::East),
        (Position::center(CellId::new(6, 5)), Direction::West),
        (Position::center(CellId::new(5, 4)), Direction::South),
    ], 2);
    punch.world.bombs.insert(EntityId(90), bomb(90, CellId::new(5, 5)));
    for seat in PlayerCount::Three.seats() {
        punch.replace_actor_equipment(*seat, Equipment::Punch);
    }
    punch.step(&[
        use_equipment(SeatId::One), use_equipment(SeatId::Two), use_equipment(SeatId::Three),
    ]).unwrap();
    assert_eq!(punch.world.bombs[&EntityId(90)].cell(), CellId::new(2, 5));
    assert_eq!(punch.actors[1].last_successful_punch_tick(), Some(Tick(1)));

    let mut bracelet = state_three(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 4_096 }, Direction::East),
        (Position { x: 4_736, y: 4_096 }, Direction::West),
        (Position { x: 4_096, y: 4_096 }, Direction::East),
    ], 3);
    bracelet.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    bracelet.replace_actor_equipment(SeatId::Three, Equipment::Bracelet);
    let report = bracelet.step(&[
        press(SeatId::One, Direction::East), press(SeatId::Three, Direction::East),
    ]).unwrap();
    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].pusher, SeatId::Three);
}

#[test]
fn unequal_contact_times_choose_earliest_victim_before_round_priority() {
    let mut game = state_three(EnabledItems::BRACELET, [
        (Position { x: 4_096, y: 4_096 }, Direction::East),
        (Position { x: 4_736, y: 3_696 }, Direction::West),
        (Position { x: 4_766, y: 4_496 }, Direction::West),
    ], 3);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);

    let report = game.step(&[press(SeatId::One, Direction::East)]).unwrap();

    assert_eq!(report.forced_movements.len(), 1);
    assert_eq!(report.forced_movements[0].seat, SeatId::Two);
}
