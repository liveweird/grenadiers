use std::collections::{BTreeMap, BTreeSet};

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::events::StepEventKind;
use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction, Pickup,
    PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, StepError, Tick,
    World,
};
use crate::room::RoomSettings;

const CENTER: CellId = CellId::new(7, 6);
const OTHER: CellId = CellId::new(11, 6);

fn arena() -> Arena {
    arena_with_blocks(BTreeSet::new())
}

fn arena_with_blocks(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn state(
    speed: SpeedLevel,
    enabled: EnabledItems,
    pickup: bool,
    shared: bool,
    tick: Tick,
) -> State {
    let exposed = if pickup {
        BTreeMap::from([(CENTER, PickupKind::SlowFoot)])
    } else {
        BTreeMap::new()
    };
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::new(speed, 256, 3, 180)
                .unwrap()
                .with_enabled_items(enabled),
        ),
        World::with_pickups(arena(), BTreeMap::new(), BTreeMap::new(), exposed).unwrap(),
        [
            ActorInit {
                id: EntityId(10),
                seat: SeatId::One,
                position: Position::center(CENTER),
                heading: Direction::East,
                speed,
            },
            ActorInit {
                id: EntityId(20),
                seat: SeatId::Two,
                position: Position::center(if shared { CENTER } else { OTHER }),
                heading: Direction::West,
                speed,
            },
        ],
        tick,
    )
    .unwrap()
}

fn press(direction: Direction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(direction),
    }
}

fn add_stacks(game: &mut State, seat: SeatId, count: usize, at: Tick) {
    for _ in 0..count {
        game.actors[seat.index()]
            .effects
            .apply(EffectKind::SlowFoot, at)
            .unwrap();
    }
}

#[test]
fn every_speed_and_stack_count_moves_the_exact_integer_rate_over_sixty_ticks() {
    for speed in [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        for stacks in 0..=6 {
            let mut game = state(speed, EnabledItems::SLOW_FOOT, false, false, Tick(0));
            add_stacks(&mut game, SeatId::One, stacks, Tick(0));
            let start = game.actors[0].position.x;
            game.step(&[press(Direction::East)]).unwrap();
            for _ in 1..60 {
                game.step(&[]).unwrap();
            }
            assert_eq!(
                game.actors[0].position.x - start,
                i32::from(speed.units_per_second() / (1_u16 << stacks))
            );
            assert_eq!(game.actors[0].residual, 0);
        }
    }
}

#[test]
fn collection_changes_following_tick_and_exact_expiry_restores_base_rate() {
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        true,
        false,
        Tick(0),
    );
    let start = game.actors[0].position.x;
    game.step(&[press(Direction::East)]).unwrap();
    assert_eq!(game.actors[0].position.x - start, 51);
    let collected = game.actors[0].position.x;
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position.x - collected, 25);

    game.tick = Tick(600);
    game.actors[0].residual = 0;
    let before_expiry = game.actors[0].position.x;
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position.x - before_expiry, 51);
    assert_eq!(game.actors[0].effects.slow_foot_stacks(Tick(601)), 0);
}

#[test]
fn six_stacks_keep_fractional_progress_and_zero_budget_ticks_are_real_steps() {
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        false,
        false,
        Tick(0),
    );
    add_stacks(&mut game, SeatId::One, 6, Tick(0));
    let start = game.actors[0].position;
    game.step(&[press(Direction::East)]).unwrap();
    assert_eq!(game.actors[0].position, start);
    assert_eq!(game.actors[0].residual, 48);
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position.x, start.x + 1);
    assert_eq!(game.actors[0].residual, 36);

    let mut burning = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        false,
        false,
        Tick(0),
    );
    add_stacks(&mut burning, SeatId::One, 6, Tick(0));
    burning.world.hazards.insert(CENTER, Tick(10));
    burning.step(&[press(Direction::East)]).unwrap();
    assert!(!burning.actors[0].alive);
}

#[test]
fn disabled_slow_foot_state_preserves_historical_base_movement() {
    let mut game = state(SpeedLevel::Base, EnabledItems::EMPTY, false, false, Tick(0));
    add_stacks(&mut game, SeatId::One, 6, Tick(0));
    let start = game.actors[0].position.x;
    game.step(&[press(Direction::East)]).unwrap();
    assert_eq!(game.actors[0].position.x - start, 51);
}

#[test]
fn reverse_controls_and_slow_foot_compose_without_changing_raw_input() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::SLOW_FOOT.bits() | EnabledItems::REVERSE_CONTROLS.bits(),
    )
    .unwrap();
    let mut game = state(SpeedLevel::Base, enabled, false, false, Tick(0));
    game.actors[0]
        .effects
        .apply(EffectKind::SlowFoot, Tick(0))
        .unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::ReverseControls, Tick(0))
        .unwrap();
    let start = game.actors[0].position.x;
    game.step(&[press(Direction::East)]).unwrap();
    assert_eq!(game.actors[0].held.active(), Some(Direction::East));
    assert_eq!(start - game.actors[0].position.x, 25);
    assert_eq!(game.actors[0].heading, Direction::West);
}

#[test]
fn blocked_whole_budgets_are_not_released_after_the_obstacle_disappears() {
    let blocker = CellId::new(8, 6);
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        false,
        false,
        Tick(0),
    );
    game.world.arena = arena_with_blocks(BTreeSet::from([blocker]));
    add_stacks(&mut game, SeatId::One, 1, Tick(0));
    game.step(&[press(Direction::East)]).unwrap();
    for _ in 1..60 {
        game.step(&[]).unwrap();
    }
    game.world.arena = arena();
    let released = game.actors[0].position.x;
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position.x - released, 25);
}

#[test]
fn speed_upgrade_and_stack_changes_preserve_fractional_remainder() {
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        false,
        false,
        Tick(0),
    );
    add_stacks(&mut game, SeatId::One, 6, Tick(0));
    game.world
        .pickups
        .exposed
        .insert(CENTER, Pickup::new(PickupKind::MovementSpeed));
    game.step(&[press(Direction::East)]).unwrap();
    assert_eq!(game.actors[0].residual, 48);
    assert_eq!(game.actors[0].speed, SpeedLevel::One);
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].residual, 44);
}

#[test]
fn capped_actor_is_excluded_before_distance_and_tie_arbitration() {
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        true,
        true,
        Tick(0),
    );
    add_stacks(&mut game, SeatId::One, 6, Tick(0));
    game.actors[1].position.x += 100;
    let report = game.step(&[]).unwrap();
    assert_eq!(game.actors[0].effects.slow_foot_stacks(Tick(1)), 6);
    assert_eq!(game.actors[1].effects.slow_foot_stacks(Tick(1)), 1);
    assert!(game.world.pickups.exposed.is_empty());
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::PickupCollected {
            seat: SeatId::Two,
            kind: PickupKind::SlowFoot,
            ..
        }
    )));
}

#[test]
fn two_capped_actors_retain_pickup_without_event_or_refresh() {
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        true,
        true,
        Tick(0),
    );
    add_stacks(&mut game, SeatId::One, 6, Tick(0));
    add_stacks(&mut game, SeatId::Two, 6, Tick(0));
    let before = game
        .actors
        .iter()
        .map(|actor| actor.effects.clone())
        .collect::<Vec<_>>();
    let report = game.step(&[]).unwrap();
    assert!(game.world.pickups.exposed.contains_key(&CENTER));
    assert!(report.events.is_empty());
    assert_eq!(
        game.actors
            .iter()
            .map(|actor| actor.effects.clone())
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn expiry_frees_capacity_before_collection_and_replacement_slows_next_tick() {
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        true,
        false,
        Tick(599),
    );
    add_stacks(&mut game, SeatId::One, 6, Tick(0));
    let start = game.actors[0].position.x;
    game.step(&[press(Direction::East)]).unwrap();
    assert_eq!(game.actors[0].position.x - start, 51);
    assert_eq!(game.actors[0].effects.slow_foot_stacks(Tick(600)), 1);
    let collected = game.actors[0].position.x;
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position.x - collected, 25);
}

#[test]
fn collection_overflow_rolls_back_movement_input_and_pickup() {
    let start_tick = Tick(u64::MAX - 600);
    let mut game = state(
        SpeedLevel::Base,
        EnabledItems::SLOW_FOOT,
        true,
        false,
        start_tick,
    );
    let before = game.clone();
    assert_eq!(
        game.step(&[press(Direction::East)]),
        Err(StepError::EffectExpiryOverflow(EffectKind::SlowFoot))
    );
    assert_eq!(game, before);
}

#[test]
fn slow_foot_is_rejected_from_every_pickup_layer_when_disabled() {
    for layer in 0..3 {
        let pickup = BTreeMap::from([(CENTER, PickupKind::SlowFoot)]);
        let world = World::with_pickups(
            arena_with_blocks(if layer == 0 {
                BTreeSet::from([CENTER])
            } else {
                BTreeSet::new()
            }),
            if layer == 0 {
                pickup.clone()
            } else {
                BTreeMap::new()
            },
            if layer == 1 {
                pickup.clone()
            } else {
                BTreeMap::new()
            },
            if layer == 2 {
                pickup.clone()
            } else {
                BTreeMap::new()
            },
        );
        let world = world.unwrap();
        assert!(
            State::new(
                RulesConfig::expanded_v5(RoomSettings::default()),
                world,
                [
                    ActorInit {
                        id: EntityId(1),
                        seat: SeatId::One,
                        position: Position::center(if layer == 0 {
                            CellId::new(7, 5)
                        } else {
                            CENTER
                        }),
                        heading: Direction::East,
                        speed: SpeedLevel::Base,
                    },
                    ActorInit {
                        id: EntityId(2),
                        seat: SeatId::Two,
                        position: Position::center(OTHER),
                        heading: Direction::West,
                        speed: SpeedLevel::Base,
                    },
                ],
            )
            .is_err()
        );
    }
}
