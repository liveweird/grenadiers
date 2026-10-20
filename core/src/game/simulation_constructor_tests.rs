use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, InputAction, SeatInput, State, StateError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::{Bomb, BombTrigger};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;
use crate::game::{EnabledItems, PickupKind};
use crate::room::RoomSettings;

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

fn open_world() -> World {
    World::new(open_arena(BTreeSet::new()))
}

fn actor(id: u32, seat: SeatId, cell: CellId) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn actors(one: u32, two: u32) -> [ActorInit; 2] {
    [
        actor(one, SeatId::One, CellId::new(2, 5)),
        actor(two, SeatId::Two, CellId::new(12, 5)),
    ]
}

#[test]
fn absolute_tick_constructor_keeps_round_update_clock_at_zero() {
    let mut state = State::new_at_tick(
        RulesConfig::classic_v1(),
        open_world(),
        actors(10, 20),
        Tick(180),
    )
    .unwrap();

    assert_eq!(state.tick(), Tick(180));
    assert_eq!(state.running_updates(), 0);
    assert_eq!(
        state.round_assignment(),
        crate::game::RoundAssignment::first()
    );
    assert_eq!(state.step(&[]).unwrap().tick, Tick(181));
    assert_eq!(state.running_updates(), 1);
}

#[test]
fn cloned_nonempty_world_accepts_known_owner_and_seeds_allocator_above_bombs() {
    let mut source = State::new(RulesConfig::classic_v1(), open_world(), actors(10, 20)).unwrap();
    add_test_bomb(&mut source, EntityId(90), CellId::new(8, 5), EntityId(10)).unwrap();

    let mut rebuilt = State::new(
        RulesConfig::classic_v1(),
        source.world().clone(),
        actors(10, 20),
    )
    .unwrap();
    rebuilt
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    assert!(rebuilt.world().bombs().contains_key(&EntityId(91)));
    assert_eq!(rebuilt.live_bomb_count(SeatId::One), 1);
    assert_eq!(rebuilt.live_bomb_count(SeatId::Two), 1);
}

#[test]
fn constructor_rejects_a_bomb_owned_by_an_unknown_actor() {
    let mut world = open_world();
    let bomb = Bomb::new(
        EntityId(90),
        CellId::new(8, 5),
        EntityId(99),
        2,
        Tick(1),
        BombTrigger::Timed {
            detonates_at: Tick(181),
        },
    )
    .unwrap();
    world.bombs.insert(bomb.id(), bomb);

    assert_eq!(
        State::new(RulesConfig::classic_v1(), world, actors(10, 20)),
        Err(StateError::UnknownBombOwner(EntityId(99)))
    );
}

#[test]
fn maximum_actor_id_does_not_exhaust_lower_bomb_ids() {
    let mut state =
        State::new(RulesConfig::classic_v1(), open_world(), actors(u32::MAX, 1)).unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    let bomb = state.world().bombs().get(&EntityId(0)).unwrap();
    assert_eq!(bomb.owner(), EntityId(u32::MAX));
}

#[test]
fn constructor_rejects_a_bomb_whose_map_key_disagrees_with_its_id() {
    let mut world = open_world();
    world.bombs.insert(
        EntityId(89),
        Bomb::new(
            EntityId(90),
            CellId::new(8, 5),
            EntityId(10),
            2,
            Tick(1),
            BombTrigger::Timed {
                detonates_at: Tick(181),
            },
        )
        .unwrap(),
    );

    assert_eq!(
        State::new(RulesConfig::classic_v1(), world, actors(10, 20)),
        Err(StateError::BombIdMismatch {
            key: EntityId(89),
            id: EntityId(90),
        })
    );
}

#[test]
fn legacy_profiles_reject_enabled_settings_and_opt_in_items_in_every_layer() {
    for enabled in [
        EnabledItems::HEART,
        EnabledItems::SHIELD,
        EnabledItems::HEART_AND_SHIELD,
        EnabledItems::GOLDEN_FLAME,
        EnabledItems::HEART_AND_GOLDEN_FLAME,
        EnabledItems::SHIELD_AND_GOLDEN_FLAME,
        EnabledItems::from_bits(4624).unwrap(),
        EnabledItems::SHORT_FUSE,
        EnabledItems::SHORT_FLAME,
        EnabledItems::ALL,
    ] {
        let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
        assert_eq!(
            State::new(
                RulesConfig::configurable_v4(settings),
                open_world(),
                actors(10, 20),
            ),
            Err(StateError::UnsupportedEnabledItems(enabled))
        );
    }

    for kind in [
        PickupKind::Heart,
        PickupKind::Shield,
        PickupKind::GoldenFlame,
        PickupKind::ShortFuse,
        PickupKind::Diarrhea,
        PickupKind::SlowFoot,
        PickupKind::ShortFlame,
        PickupKind::LongFuse,
        PickupKind::ReverseControls,
    ] {
        for layer in 0..3 {
            let cell = CellId::new(8, 5);
            let pickup = BTreeMap::from([(cell, kind)]);
            let world = World::with_pickups(
                open_arena(if layer == 0 {
                    BTreeSet::from([cell])
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
            for rules in [
                RulesConfig::classic_v1(),
                RulesConfig::classic_v2(),
                RulesConfig::classic_v3(),
                RulesConfig::configurable_v4(RoomSettings::DEFAULT),
            ] {
                assert_eq!(
                    State::new(rules, world.clone(), actors(10, 20)),
                    Err(StateError::UnsupportedPickup { cell, kind })
                );
            }
        }
    }
}

#[test]
fn expanded_profile_rejects_shield_in_every_layer_unless_enabled() {
    for enabled in [EnabledItems::EMPTY, EnabledItems::HEART] {
        for layer in 0..3 {
            let cell = CellId::new(8, 5);
            let shield = BTreeMap::from([(cell, PickupKind::Shield)]);
            let world = World::with_pickups(
                open_arena(if layer == 0 {
                    BTreeSet::from([cell])
                } else {
                    BTreeSet::new()
                }),
                if layer == 0 {
                    shield.clone()
                } else {
                    BTreeMap::new()
                },
                if layer == 1 {
                    shield.clone()
                } else {
                    BTreeMap::new()
                },
                if layer == 2 { shield } else { BTreeMap::new() },
            )
            .unwrap();
            let rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled));
            assert_eq!(
                State::new(rules, world, actors(10, 20)),
                Err(StateError::UnsupportedPickup {
                    cell,
                    kind: PickupKind::Shield,
                })
            );
        }
    }
}
