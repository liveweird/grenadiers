use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, InputAction, SeatInput, State, StateError, StepError};
use crate::game::EnabledItems;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::BombTrigger;
use crate::game::effects::{EffectKind, ORDINARY_EFFECT_TICKS};
use crate::game::input::Direction;
use crate::game::inventory::Equipment;
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::rules::RulesConfig;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;
use crate::room::RoomSettings;
use crate::room::settings::MAX_TURN_ASSIST_UNITS;

const TARGET: CellId = CellId::new(5, 5);

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

fn actor(id: u32, seat: SeatId, cell: CellId) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn settings(fuse: u64, enabled: EnabledItems) -> RoomSettings {
    RoomSettings::new(SpeedLevel::Base, MAX_TURN_ASSIST_UNITS, 3, fuse)
        .unwrap()
        .with_enabled_items(enabled)
}

fn state_with(world: World, tick: Tick, fuse: u64) -> State {
    state_with_actor(world, tick, fuse, TARGET)
}

fn state_with_actor(world: World, tick: Tick, fuse: u64, one: CellId) -> State {
    State::new_at_tick(
        RulesConfig::expanded_v5(settings(fuse, EnabledItems::ALL)),
        world,
        [
            actor(10, SeatId::One, one),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
        tick,
    )
    .unwrap()
}

fn exposed(kind: PickupKind) -> World {
    World::with_pickups(
        open_arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(TARGET, kind)]),
    )
    .unwrap()
}

fn place() -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }
}

#[test]
fn collection_orders_after_placement_refreshes_and_expires_before_placement() {
    let mut state = state_with(exposed(PickupKind::ShortFuse), Tick(10), 180);
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    state.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(10))
        .unwrap();
    state.actors[0]
        .effects
        .apply(EffectKind::GoldenFlame, Tick(10))
        .unwrap();

    state.step(&[place()]).unwrap();
    assert_eq!(
        state.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(191))
    );
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(610))
    );
    assert!(state.actors[0].effects.active(EffectKind::Shield, Tick(11)));
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(11))
    );
    assert_eq!(
        state.actors[0].inventory.equipment(),
        Some(Equipment::Heart)
    );

    state.tick = Tick(20);
    state.world.bombs.clear();
    state.actors[0].bomb_grace.clear();
    state
        .world
        .pickups
        .exposed
        .insert(TARGET, Pickup::new(PickupKind::ShortFuse));
    state.step(&[]).unwrap();
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(620))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(621))
    );

    state.tick = Tick(620);
    state.step(&[place()]).unwrap();
    assert_eq!(
        state.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(801))
    );
}

#[test]
fn ordinary_and_short_schedules_coexist_and_survive_later_effect_cleanup() {
    let mut state = state_with(exposed(PickupKind::ShortFuse), Tick(0), 180);
    state.actors[0].capacity = 2;
    state.step(&[place()]).unwrap();
    state.actors[0].position = Position::center(CellId::new(6, 5));
    state.step(&[place()]).unwrap();
    let schedules: BTreeSet<_> = state
        .world
        .bombs
        .values()
        .filter_map(|bomb| bomb.detonates_at())
        .collect();
    assert_eq!(schedules, BTreeSet::from([Tick(92), Tick(181)]));

    state.world.bombs.clear();
    state.actors[0].bomb_grace.clear();
    state.actors[0].position = Position::center(CellId::new(10, 5));
    state.tick = Tick(599);
    state.step(&[place()]).unwrap();
    assert_eq!(
        state.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(690))
    );
    state.step(&[]).unwrap();
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(601))
    );
    assert_eq!(
        state.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(690))
    );

    state.actors[0].position = Position::center(CellId::new(11, 5));
    state.world.hazards.insert(CellId::new(11, 5), Tick(700));
    state.step(&[]).unwrap();
    assert!(!state.actors[0].alive());
    assert_eq!(state.actors[0].effects, Default::default());
    assert_eq!(
        state.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(690))
    );
}

#[test]
fn configured_minimum_ninety_and_maximum_fuses_are_halved_and_detonate_exactly() {
    for (base, shortened) in [(60, 30), (90, 45), (300, 150)] {
        let mut state = state_with(World::new(open_arena(BTreeSet::new())), Tick(0), base);
        state.actors[0]
            .effects
            .apply(EffectKind::ShortFuse, Tick(0))
            .unwrap();
        state.step(&[place()]).unwrap();
        assert_eq!(
            state.world.bombs.values().next().unwrap().detonates_at(),
            Some(Tick(1 + shortened))
        );
    }

    let mut state = state_with(World::new(open_arena(BTreeSet::new())), Tick(0), 90);
    state.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(0))
        .unwrap();
    state.step(&[place()]).unwrap();
    state.actors[0].position = Position::center(CellId::new(1, 1));
    state.tick = Tick(45);
    state.step(&[]).unwrap();
    assert!(state.world.bombs.is_empty());
    assert!(state.world.hazards.contains_key(&TARGET));
}

#[test]
fn placement_deadline_overflow_declines_only_placement_but_collection_overflow_rolls_back() {
    let mut placement = state_with(
        World::new(open_arena(BTreeSet::new())),
        Tick(u64::MAX - 45),
        90,
    );
    placement.actors[0]
        .effects
        .apply(
            EffectKind::ShortFuse,
            Tick(u64::MAX - ORDINARY_EFFECT_TICKS),
        )
        .unwrap();
    placement.step(&[place()]).unwrap();
    assert!(placement.world.bombs.is_empty());
    assert_eq!(placement.next_entity_id, 0);

    let mut exact = state_with(
        exposed(PickupKind::ShortFuse),
        Tick(u64::MAX - ORDINARY_EFFECT_TICKS - 1),
        90,
    );
    exact.step(&[]).unwrap();
    assert!(
        exact.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(u64::MAX - 1))
    );
    assert!(
        !exact.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(u64::MAX))
    );

    let mut collection = state_with(
        exposed(PickupKind::ShortFuse),
        Tick(u64::MAX - ORDINARY_EFFECT_TICKS),
        90,
    );
    collection
        .insert_test_bomb(
            EntityId(90),
            CellId::new(8, 5),
            EntityId(20),
            1,
            collection.tick,
            BombTrigger::Timed {
                detonates_at: Tick(u64::MAX - ORDINARY_EFFECT_TICKS + 1),
            },
        )
        .unwrap();
    let before = collection.clone();
    assert_eq!(
        collection.step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }]),
        Err(StepError::EffectExpiryOverflow(EffectKind::ShortFuse))
    );
    assert_eq!(collection, before);
}

#[test]
fn concealed_short_fuse_uses_ordinary_reveal_and_collection_layers() {
    let world = World::with_pickups(
        open_arena(BTreeSet::from([TARGET])),
        BTreeMap::from([(TARGET, PickupKind::ShortFuse)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let mut state = state_with_actor(world, Tick(0), 180, CellId::new(1, 1));
    state
        .insert_test_bomb(
            EntityId(90),
            CellId::new(3, 5),
            EntityId(10),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap();
    state.step(&[]).unwrap();
    assert_eq!(
        state
            .world
            .pickups
            .pending
            .get(&TARGET)
            .map(|item| item.kind()),
        Some(PickupKind::ShortFuse)
    );
    state.tick = Tick(30);
    state.actors[0].position = Position::center(TARGET);
    state.step(&[]).unwrap();
    assert!(state.world.pickups.pending.is_empty());
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(31))
    );
}

#[test]
fn short_fuse_pickups_are_rejected_in_every_layer_unless_enabled() {
    for layer in 0..3 {
        let pickup = BTreeMap::from([(TARGET, PickupKind::ShortFuse)]);
        let world = World::with_pickups(
            open_arena(if layer == 0 {
                BTreeSet::from([TARGET])
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
                RulesConfig::expanded_v5(settings(180, EnabledItems::HEART)),
                world,
                [
                    actor(10, SeatId::One, CellId::new(1, 1)),
                    actor(20, SeatId::Two, CellId::new(13, 11))
                ],
            ),
            Err(StateError::UnsupportedPickup {
                cell: TARGET,
                kind: PickupKind::ShortFuse
            })
        );
    }
}
