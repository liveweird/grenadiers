use std::collections::{BTreeMap, BTreeSet};

use super::super::{ActorInit, InputAction, SeatInput, State, StateError, StepError};
use crate::game::EnabledItems;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::BombTrigger;
use crate::game::effects::{EffectKind, GOLDEN_FLAME_RANGE, ORDINARY_EFFECT_TICKS};
use crate::game::input::Direction;
use crate::game::inventory::Equipment;
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::rules::RulesConfig;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;
use crate::room::RoomSettings;

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

fn state_with(world: World, tick: Tick, one: CellId) -> State {
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL)),
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
fn collection_refreshes_for_600_ticks_without_disturbing_heart_shield_or_short_fuse() {
    let mut state = state_with(exposed(PickupKind::ShortFlame), Tick(10), TARGET);
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    state.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(10))
        .unwrap();
    state.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(10))
        .unwrap();

    state.step(&[]).unwrap();
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::ShortFlame, Tick(610))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::ShortFlame, Tick(611))
    );
    assert!(state.actors[0].effects.active(EffectKind::Shield, Tick(11)));
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::ShortFuse, Tick(11))
    );
    assert_eq!(
        state.actors[0].inventory.equipment(),
        Some(Equipment::Heart)
    );

    state.tick = Tick(20);
    state
        .world
        .pickups
        .exposed
        .insert(TARGET, Pickup::new(PickupKind::ShortFlame));
    state.step(&[]).unwrap();
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::ShortFlame, Tick(620))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::ShortFlame, Tick(621))
    );
}

#[test]
fn placement_precedes_collection_and_bombs_capture_ranges_while_base_upgrades_continue() {
    let mut state = state_with(exposed(PickupKind::ShortFlame), Tick(0), TARGET);
    state.actors[0].capacity = 2;
    state.step(&[place()]).unwrap();
    assert_eq!(state.world.bombs.values().next().unwrap().range(), 2);

    state.actors[0].position = Position::center(CellId::new(6, 5));
    state
        .world
        .pickups
        .exposed
        .insert(CellId::new(6, 5), Pickup::new(PickupKind::BlastRange));
    state.step(&[place()]).unwrap();
    let ranges: BTreeSet<_> = state
        .world
        .bombs
        .values()
        .map(|bomb| bomb.range())
        .collect();
    assert_eq!(ranges, BTreeSet::from([1, 2]));
    assert_eq!(state.actors[0].bomb_range(), 3);

    state.world.bombs.clear();
    state.actors[0].bomb_grace.clear();
    state.tick = Tick(600);
    state.step(&[place()]).unwrap();
    assert_eq!(state.world.bombs.values().next().unwrap().range(), 3);
}

#[test]
fn short_flame_overrides_golden_then_returns_to_the_surviving_or_base_range() {
    let mut short_expires_first =
        state_with(World::new(open_arena(BTreeSet::new())), Tick(0), TARGET);
    short_expires_first.actors[0]
        .effects
        .apply(EffectKind::ShortFlame, Tick(0))
        .unwrap();
    short_expires_first.actors[0]
        .effects
        .apply(EffectKind::GoldenFlame, Tick(100))
        .unwrap();
    assert_eq!(
        short_expires_first.actors[0].effective_bomb_range(Tick(599)),
        1
    );
    assert_eq!(
        short_expires_first.actors[0].effective_bomb_range(Tick(600)),
        GOLDEN_FLAME_RANGE
    );
    assert_eq!(
        short_expires_first.actors[0].effective_bomb_range(Tick(700)),
        2
    );

    let mut golden_expires_first =
        state_with(World::new(open_arena(BTreeSet::new())), Tick(0), TARGET);
    golden_expires_first.actors[0]
        .effects
        .apply(EffectKind::GoldenFlame, Tick(0))
        .unwrap();
    golden_expires_first.actors[0]
        .effects
        .apply(EffectKind::ShortFlame, Tick(100))
        .unwrap();
    assert_eq!(
        golden_expires_first.actors[0].effective_bomb_range(Tick(600)),
        1
    );
    assert_eq!(
        golden_expires_first.actors[0].effective_bomb_range(Tick(700)),
        2
    );
}

#[test]
fn short_flame_and_short_fuse_independently_shape_a_new_bomb() {
    let mut state = state_with(World::new(open_arena(BTreeSet::new())), Tick(0), TARGET);
    state.actors[0]
        .effects
        .apply(EffectKind::ShortFlame, Tick(0))
        .unwrap();
    state.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(0))
        .unwrap();
    state.step(&[place()]).unwrap();
    let bomb = state.world.bombs.values().next().unwrap();
    assert_eq!(bomb.range(), 1);
    assert_eq!(bomb.detonates_at(), Some(Tick(91)));
}

#[test]
fn a_short_bomb_can_trigger_a_chain_bomb_that_keeps_its_own_range() {
    let blocker = CellId::new(3, 3);
    let chain = CellId::new(3, 5);
    let mut state = state_with(
        World::new(open_arena(BTreeSet::from([blocker]))),
        Tick(0),
        CellId::new(2, 5),
    );
    state.actors[0]
        .effects
        .apply(EffectKind::ShortFlame, Tick(0))
        .unwrap();
    state.step(&[place()]).unwrap();
    let short = state.world.bombs.values().next().unwrap().id();
    state.world.bombs.get_mut(&short).unwrap().trigger = BombTrigger::Timed {
        detonates_at: Tick(2),
    };
    state
        .insert_test_bomb(
            EntityId(90),
            chain,
            EntityId(20),
            3,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap();
    state.actors[0].position = Position::center(CellId::new(1, 1));

    state.step(&[]).unwrap();
    assert!(state.world.bombs.is_empty());
    assert!(state.world.hazards.contains_key(&CellId::new(6, 5)));
    assert!(state.world.hazards.contains_key(&blocker));
    assert!(!state.world.hazards.contains_key(&CellId::new(3, 2)));
}

#[test]
fn near_max_collection_is_exact_and_later_failure_restores_every_pickup_layer() {
    let mut exact = state_with(
        exposed(PickupKind::ShortFlame),
        Tick(u64::MAX - ORDINARY_EFFECT_TICKS - 1),
        TARGET,
    );
    exact.step(&[]).unwrap();
    assert!(
        exact.actors[0]
            .effects
            .active(EffectKind::ShortFlame, Tick(u64::MAX - 1))
    );
    assert!(
        !exact.actors[0]
            .effects
            .active(EffectKind::ShortFlame, Tick(u64::MAX))
    );

    let concealed = CellId::new(4, 5);
    let pending = CellId::new(6, 5);
    let world = World::with_pickups(
        open_arena(BTreeSet::from([concealed])),
        BTreeMap::from([(concealed, PickupKind::ShortFlame)]),
        BTreeMap::from([(pending, PickupKind::ShortFlame)]),
        BTreeMap::from([(TARGET, PickupKind::ShortFlame)]),
    )
    .unwrap();
    let mut state = state_with(world, Tick(u64::MAX - ORDINARY_EFFECT_TICKS), TARGET);
    state
        .insert_test_bomb(
            EntityId(90),
            CellId::new(8, 5),
            EntityId(20),
            1,
            state.tick,
            BombTrigger::Timed {
                detonates_at: Tick(u64::MAX - ORDINARY_EFFECT_TICKS + 1),
            },
        )
        .unwrap();
    let before = state.clone();
    assert_eq!(
        state.step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }]),
        Err(StepError::EffectExpiryOverflow(EffectKind::ShortFlame))
    );
    assert_eq!(state, before);
}

#[test]
fn disabled_short_flame_is_rejected_in_every_pickup_layer() {
    for layer in 0..3 {
        let pickup = BTreeMap::from([(TARGET, PickupKind::ShortFlame)]);
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
                RulesConfig::expanded_v5(RoomSettings::DEFAULT),
                world,
                [
                    actor(10, SeatId::One, CellId::new(1, 1)),
                    actor(20, SeatId::Two, CellId::new(13, 11)),
                ],
            ),
            Err(StateError::UnsupportedPickup {
                cell: TARGET,
                kind: PickupKind::ShortFlame,
            })
        );
    }
}

#[test]
fn death_clears_short_flame_and_a_new_round_starts_without_it() {
    let mut state = state_with(exposed(PickupKind::ShortFlame), Tick(0), TARGET);
    state.step(&[]).unwrap();
    state.world.hazards.insert(TARGET, Tick(100));
    state.step(&[]).unwrap();
    assert!(!state.actors[0].alive());
    assert_eq!(state.actors[0].effects, Default::default());

    let next = state_with(
        World::new(open_arena(BTreeSet::new())),
        Tick(2),
        CellId::new(1, 1),
    );
    assert_eq!(next.actors[0].effects, Default::default());
}
