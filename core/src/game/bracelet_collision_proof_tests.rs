use std::collections::BTreeSet;

use super::super::bracelet_test_support::{arena, press, simple_state, state};
use super::super::{Direction, Position, SeatId};
use super::outcome;
use crate::game::{
    BombTrigger, CellId, EffectKind, EnabledItems, EntityId, Equipment, Tick, World, movement,
};

fn bracelet_and(item: EnabledItems) -> EnabledItems {
    EnabledItems::from_bits(EnabledItems::BRACELET.bits() | item.bits()).unwrap()
}

fn bracelet_wall_pass_and_heart() -> EnabledItems {
    EnabledItems::from_bits(
        EnabledItems::BRACELET.bits() | EnabledItems::WALL_PASS.bits() | EnabledItems::HEART.bits(),
    )
    .unwrap()
}

fn east_claim() -> [super::MovementOutcome; 2] {
    [
        outcome(
            Position { x: 4_000, y: 4_000 },
            51,
            &[(Direction::East, 51)],
            Some(0),
        ),
        outcome(Position { x: 4_640, y: 4_000 }, 0, &[], None),
    ]
}

fn west_claim() -> [super::MovementOutcome; 2] {
    [
        outcome(
            Position { x: 5_000, y: 4_000 },
            51,
            &[(Direction::West, 51)],
            Some(0),
        ),
        outcome(Position { x: 4_360, y: 4_000 }, 0, &[], None),
    ]
}

#[test]
fn priority_winner_blocked_at_zero_does_not_fall_back_to_viable_loser() {
    let mut game = simple_state(
        EnabledItems::BRACELET,
        Position { x: 4_211, y: 4_096 },
        Position { x: 4_851, y: 4_096 },
    );
    game.replace_actor_equipment(SeatId::Two, Equipment::Bracelet);
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(5, 4),
        EntityId(2),
        1,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    game.actors[1].bomb_grace.insert(EntityId(30));

    let report = game
        .step(&[
            press(SeatId::Two, Direction::West),
            press(SeatId::One, Direction::East),
        ])
        .unwrap();

    assert!(report.forced_movement.is_none());
    assert_eq!(game.actors[0].position, Position { x: 4_262, y: 4_096 });
    assert_eq!(game.actors[1].position, Position { x: 4_800, y: 4_096 });
    assert_eq!(game.actors[0].last_successful_shove_tick(), None);
    assert_eq!(game.actors[1].last_successful_shove_tick(), None);
}

#[test]
fn overlapping_pusher_ahead_is_ignored_but_separated_pusher_still_clamps() {
    let mut overlapping = simple_state(
        EnabledItems::BRACELET,
        Position { x: 5_000, y: 4_096 },
        Position { x: 4_800, y: 4_096 },
    );
    let forced = overlapping
        .resolve_shove(&east_claim(), Tick(1))
        .expect("initial overlap permits the victim to depart through the pusher");
    assert_eq!(forced.segment.end(), Position { x: 5_824, y: 4_096 });

    let mut separated = simple_state(
        EnabledItems::BRACELET,
        Position { x: 5_500, y: 4_096 },
        Position { x: 4_800, y: 4_096 },
    );
    let forced = separated
        .resolve_shove(&east_claim(), Tick(1))
        .expect("the non-overlapping actor leaves a positive sixty-unit gap");
    assert_eq!(forced.segment.end(), Position { x: 4_860, y: 4_096 });
}

#[test]
fn victim_soft_wall_grace_allows_exit_is_pruned_and_cannot_authorize_reentry() {
    let block = CellId::new(6, 4);
    let world = World::new(arena(BTreeSet::from([block])));
    let mut game = state(
        bracelet_wall_pass_and_heart(),
        world,
        Position { x: 4_096, y: 4_608 },
        Position { x: 8_192, y: 4_608 },
        Tick(0),
    );
    game.actors[0].position = Position { x: 5_860, y: 4_608 };
    game.actors[1].position = Position { x: 6_500, y: 4_608 };
    game.replace_actor_equipment(SeatId::Two, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::Two, Equipment::Heart);
    assert_eq!(game.actors[1].soft_wall_grace, BTreeSet::from([block]));

    let exit = game.resolve_shove(&east_claim(), Tick(1)).unwrap();
    assert_eq!(exit.segment.end().x, 7_524);
    assert!(game.actors[1].soft_wall_grace.is_empty());

    game.actors[0].restore_shove_cooldown(None);
    game.actors[0].position = Position { x: 8_124, y: 4_608 };
    let reentry = game.resolve_shove(&west_claim(), Tick(2)).unwrap();
    assert_eq!(reentry.segment.start().x, 7_524);
    assert_eq!(reentry.segment.end().x, 7_488);
}

#[test]
fn initiator_reachable_soft_wall_grace_does_not_transfer_to_victim() {
    let block = CellId::new(6, 4);
    let world = World::new(arena(BTreeSet::from([block])));
    let mut game = state(
        bracelet_wall_pass_and_heart(),
        world,
        Position { x: 4_096, y: 4_608 },
        Position { x: 8_192, y: 4_608 },
        Tick(0),
    );
    game.actors[0].position = Position { x: 6_000, y: 4_608 };
    game.actors[1].position = Position { x: 5_800, y: 4_608 };
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));

    let forced = game.resolve_shove(&east_claim(), Tick(1)).unwrap();
    assert_eq!(forced.segment.end().x, 5_824);
}

fn bomb_game(holder: Position, victim: Position) -> super::super::State {
    let mut game = simple_state(EnabledItems::BRACELET, holder, victim);
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(6, 4),
        EntityId(2),
        1,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    game
}

#[test]
fn victim_bomb_grace_allows_exit_is_pruned_and_cannot_authorize_reentry() {
    let mut game = bomb_game(
        Position { x: 5_860, y: 4_608 },
        Position { x: 6_500, y: 4_608 },
    );
    assert!(movement::position_overlaps_cell(
        game.actors[1].position,
        CellId::new(6, 4)
    ));
    game.actors[1].bomb_grace.insert(EntityId(30));

    let exit = game.resolve_shove(&east_claim(), Tick(1)).unwrap();
    assert_eq!(exit.segment.end().x, 7_524);
    assert!(game.actors[1].bomb_grace.is_empty());

    game.actors[0].restore_shove_cooldown(None);
    game.actors[0].position = Position { x: 8_124, y: 4_608 };
    let reentry = game.resolve_shove(&west_claim(), Tick(2)).unwrap();
    assert_eq!(reentry.segment.start().x, 7_524);
    assert_eq!(reentry.segment.end().x, 7_488);
}

#[test]
fn initiator_bomb_grace_does_not_transfer_to_victim() {
    let mut game = bomb_game(
        Position { x: 6_000, y: 4_608 },
        Position { x: 5_800, y: 4_608 },
    );
    assert!(movement::position_overlaps_cell(
        game.actors[0].position,
        CellId::new(6, 4)
    ));
    game.actors[0].bomb_grace.insert(EntityId(30));

    let forced = game.resolve_shove(&east_claim(), Tick(1)).unwrap();
    assert_eq!(forced.segment.end().x, 5_824);
}

#[test]
fn frozen_victim_can_be_shoved_but_frozen_holder_cannot_initiate() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position { x: 4_736, y: 4_096 };
    let enabled = bracelet_and(EnabledItems::STOPWATCH);
    let mut frozen_victim = simple_state(enabled, one, two);
    frozen_victim.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));
    assert!(
        frozen_victim
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_some()
    );

    let mut frozen_holder = simple_state(enabled, one, two);
    frozen_holder.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
    assert!(
        frozen_holder
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_none()
    );
}
