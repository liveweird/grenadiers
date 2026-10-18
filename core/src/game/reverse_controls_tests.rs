use std::collections::{BTreeMap, BTreeSet};

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::simulation::{
    ActorInit, InputAction, SeatInput, State, StateError, StepError, TurnAssist,
};
use crate::game::types::HALF_FOOTPRINT;
use crate::game::{
    CellId, Direction, EffectKind, EnabledItems, EntityId, Pickup, PickupKind, Position,
    RulesConfig, SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;
use crate::room::settings::MAX_TURN_ASSIST_UNITS;

const CENTER: CellId = CellId::new(7, 6);

fn open_arena(blocks: BTreeSet<CellId>) -> Arena {
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

fn state(pickup: bool, tick: Tick) -> State {
    let world = World::with_pickups(
        open_arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        if pickup {
            BTreeMap::from([(CENTER, PickupKind::ReverseControls)])
        } else {
            BTreeMap::new()
        },
    )
    .unwrap();
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::new(SpeedLevel::Base, MAX_TURN_ASSIST_UNITS, 3, 180)
                .unwrap()
                .with_enabled_items(EnabledItems::REVERSE_CONTROLS),
        ),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CENTER),
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

fn input(action: InputAction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action,
    }
}

#[test]
fn active_effect_inverts_each_raw_direction_and_heading_follows_motion() {
    for (raw, expected, coordinate) in [
        (Direction::North, Direction::South, (0, 1)),
        (Direction::East, Direction::West, (-1, 0)),
        (Direction::South, Direction::North, (0, -1)),
        (Direction::West, Direction::East, (1, 0)),
    ] {
        let mut game = state(false, Tick(0));
        game.actors[0]
            .effects
            .apply(EffectKind::ReverseControls, Tick(0))
            .unwrap();
        let start = game.actors[0].position;
        game.step(&[input(InputAction::Press(raw))]).unwrap();
        let moved = game.actors[0].position;
        assert_eq!(game.actors[0].heading, expected);
        assert_eq!(moved.x.cmp(&start.x) as i8, coordinate.0);
        assert_eq!(moved.y.cmp(&start.y) as i8, coordinate.1);
        assert_eq!(game.actors[0].held.active(), Some(raw));
    }
}

#[test]
fn collection_starts_next_tick_and_expiry_restores_raw_motion_at_boundary() {
    let mut game = state(true, Tick(0));
    let start = game.actors[0].position;
    game.step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    assert!(game.actors[0].position.x > start.x);
    let after_collection = game.actors[0].position;
    game.step(&[]).unwrap();
    assert!(game.actors[0].position.x < after_collection.x);
    game.tick = Tick(600);
    let before_expiry = game.actors[0].position;
    game.step(&[]).unwrap();
    assert!(game.actors[0].position.x > before_expiry.x);
    assert_eq!(game.actors[0].held.active(), Some(Direction::East));
}

#[test]
fn raw_release_fallback_repeated_press_and_clear_remain_canonical() {
    let mut game = state(false, Tick(0));
    game.actors[0]
        .effects
        .apply(EffectKind::ReverseControls, Tick(0))
        .unwrap();
    game.step(&[
        input(InputAction::Press(Direction::North)),
        input(InputAction::Press(Direction::East)),
    ])
    .unwrap();
    assert_eq!(game.actors[0].held.active(), Some(Direction::East));
    game.step(&[
        input(InputAction::Press(Direction::North)),
        input(InputAction::Release(Direction::East)),
    ])
    .unwrap();
    assert_eq!(game.actors[0].held.active(), Some(Direction::North));
    assert_eq!(game.actors[0].heading, Direction::South);
    game.step(&[input(InputAction::Clear)]).unwrap();
    assert_eq!(game.actors[0].held.active(), None);
    assert_eq!(game.actors[0].residual, 0);
}

#[test]
fn inversion_transitions_clear_assist_while_refresh_preserves_it() {
    let mut collection = state(true, Tick(0));
    collection.actors[0].held.press(Direction::East);
    collection.actors[0].pending_turn = Some(TurnAssist {
        direction: Direction::East,
        target: 0,
    });
    collection.step(&[]).unwrap();
    assert_eq!(collection.actors[0].pending_turn, None);

    collection.actors[0].held.clear();
    collection.actors[0].held.press(Direction::South);
    collection.actors[0].pending_turn = Some(TurnAssist {
        direction: Direction::North,
        target: 0,
    });
    collection
        .world
        .pickups
        .exposed
        .insert(CENTER, Pickup::new(PickupKind::ReverseControls));
    collection.step(&[]).unwrap();
    assert!(collection.actors[0].pending_turn.is_some());

    collection.tick = Tick(601);
    collection.actors[0].pending_turn = Some(TurnAssist {
        direction: Direction::West,
        target: 0,
    });
    collection.step(&[]).unwrap();
    assert_eq!(collection.actors[0].pending_turn, None);
}

#[test]
fn overflow_collection_rolls_back_inputs_movement_and_pickup() {
    let mut game = state(true, Tick(u64::MAX - 600));
    let before = game.clone();
    assert_eq!(
        game.step(&[input(InputAction::Press(Direction::East))]),
        Err(StepError::EffectExpiryOverflow(EffectKind::ReverseControls))
    );
    assert_eq!(game, before);
}

#[test]
fn disabled_reverse_controls_is_rejected_in_every_pickup_layer() {
    for layer in 0..3 {
        let pickup = BTreeMap::from([(CENTER, PickupKind::ReverseControls)]);
        let world = World::with_pickups(
            open_arena(if layer == 0 {
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
            if layer == 2 { pickup } else { BTreeMap::new() },
        )
        .unwrap();
        assert_eq!(
            State::new(
                RulesConfig::expanded_v5(RoomSettings::DEFAULT),
                world,
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
                ],
            ),
            Err(StateError::UnsupportedPickup {
                cell: CENTER,
                kind: PickupKind::ReverseControls,
            })
        );
    }
}

#[test]
fn stationary_inversion_preserves_heading_and_fractional_budget() {
    let mut stationary = state(false, Tick(0));
    stationary.actors[0]
        .effects
        .apply(EffectKind::ReverseControls, Tick(0))
        .unwrap();
    stationary.step(&[]).unwrap();
    assert_eq!(stationary.actors[0].heading, Direction::East);
    assert_eq!(stationary.actors[0].position, Position::center(CENTER));

    let mut reversed = state(false, Tick(0));
    reversed.actors[0]
        .effects
        .apply(EffectKind::ReverseControls, Tick(0))
        .unwrap();
    let mut ordinary = state(false, Tick(0));
    for game in [&mut reversed, &mut ordinary] {
        game.actors[0].speed = SpeedLevel::One;
    }
    reversed
        .step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    ordinary
        .step(&[input(InputAction::Press(Direction::West))])
        .unwrap();
    for _ in 0..28 {
        reversed.step(&[]).unwrap();
        ordinary.step(&[]).unwrap();
    }
    assert_eq!(reversed.actors[0].position, ordinary.actors[0].position);
    assert_eq!(reversed.actors[0].residual, ordinary.actors[0].residual);
}

#[test]
fn reversed_motion_obeys_blocking_geometry() {
    let mut game = state(false, Tick(0));
    game.world.arena = open_arena(BTreeSet::from([CellId::new(6, 6)]));
    game.actors[0]
        .effects
        .apply(EffectKind::ReverseControls, Tick(0))
        .unwrap();
    game.step(&[input(InputAction::Press(Direction::East))])
        .unwrap();
    for _ in 0..120 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.actors[0].position.x, 7 * 1_024 + HALF_FOOTPRINT);
    assert_eq!(game.actors[0].heading, Direction::West);
}

#[test]
fn expiry_and_recollection_same_tick_moves_normally_then_reverses() {
    let mut game = state(false, Tick(600));
    game.actors[0]
        .effects
        .apply(EffectKind::ReverseControls, Tick(0))
        .unwrap();
    game.world
        .pickups
        .exposed
        .insert(CENTER, Pickup::new(PickupKind::ReverseControls));
    game.actors[0].held.press(Direction::East);
    let start = game.actors[0].position;
    game.step(&[]).unwrap();
    assert!(game.actors[0].position.x > start.x);
    let collected = game.actors[0].position;
    assert!(game.actors[0].effects.reverses_controls(game.tick()));
    game.step(&[]).unwrap();
    assert!(game.actors[0].position.x < collected.x);
}
