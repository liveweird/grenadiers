use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, InputAction, SeatInput, State, StepError};
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
fn collection_refreshes_for_600_ticks_and_coexists_with_shield_heart_and_grace() {
    let mut state = state_with(exposed(PickupKind::GoldenFlame), Tick(10), TARGET);
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    state.step(&[]).unwrap();
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(610))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(611))
    );
    state.world.hazards.insert(TARGET, Tick(100));
    state.step(&[]).unwrap();
    assert_eq!(state.actors[0].inventory.equipment(), None);
    assert!(
        state.actors[0]
            .damage_grace
            .is_some_and(|grace| grace.is_active(Tick(12)))
    );

    state.world.hazards.clear();
    state
        .world
        .pickups
        .exposed
        .insert(TARGET, Pickup::new(PickupKind::Shield));
    state.step(&[]).unwrap();
    assert!(state.actors[0].effects.active(EffectKind::Shield, Tick(13)));
    assert!(
        state.actors[0]
            .damage_grace
            .is_some_and(|grace| grace.is_active(Tick(13)))
    );

    state.tick = Tick(20);
    state
        .world
        .pickups
        .exposed
        .insert(TARGET, Pickup::new(PickupKind::GoldenFlame));
    state.step(&[]).unwrap();

    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(620))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(621))
    );
}

#[test]
fn collection_follows_placement_and_bombs_capture_range_while_base_upgrades_continue() {
    let mut state = state_with(exposed(PickupKind::GoldenFlame), Tick(0), TARGET);

    state.step(&[place()]).unwrap();
    assert_eq!(state.world.bombs.values().next().unwrap().range(), 2);
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(1))
    );

    state.world.bombs.clear();
    state.actors[0].bomb_grace.clear();
    state
        .world
        .pickups
        .exposed
        .insert(TARGET, Pickup::new(PickupKind::BlastRange));
    state.step(&[place()]).unwrap();
    let boosted = *state.world.bombs.values().next().unwrap();
    assert_eq!(boosted.range(), GOLDEN_FLAME_RANGE);
    assert_eq!(state.actors[0].bomb_range(), 3);

    state.tick = Tick(600);
    state.world.bombs.clear();
    state.actors[0].bomb_grace.clear();
    state.step(&[place()]).unwrap();
    assert_eq!(boosted.range(), GOLDEN_FLAME_RANGE);
    assert_eq!(state.world.bombs.values().next().unwrap().range(), 3);
}

#[test]
fn captured_range_reaches_chain_bombs_but_still_stops_at_blockers() {
    let blocker = CellId::new(2, 3);
    let chain = CellId::new(7, 5);
    let mut state = state_with(
        World::new(open_arena(BTreeSet::from([blocker]))),
        Tick(0),
        CellId::new(2, 5),
    );
    state.actors[0]
        .effects
        .apply(EffectKind::GoldenFlame, Tick(0))
        .unwrap();
    state.step(&[place()]).unwrap();
    let golden = state.world.bombs.values().next().unwrap().id();
    state.world.bombs.get_mut(&golden).unwrap().trigger = BombTrigger::Timed {
        detonates_at: Tick(2),
    };
    state
        .insert_test_bomb(
            EntityId(90),
            chain,
            EntityId(20),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap();

    state.step(&[]).unwrap();

    assert!(state.world.bombs.is_empty());
    assert!(state.world.hazards.contains_key(&chain));
    assert!(state.world.hazards.contains_key(&blocker));
    assert!(!state.world.hazards.contains_key(&CellId::new(2, 2)));
    assert!(state.world.hazards.contains_key(&CellId::new(9, 5)));
}

#[test]
fn concealed_golden_flame_reveals_then_collects_through_the_ordinary_layers() {
    let world = World::with_pickups(
        open_arena(BTreeSet::from([TARGET])),
        BTreeMap::from([(TARGET, PickupKind::GoldenFlame)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let mut state = state_with(world, Tick(0), CellId::new(1, 1));
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
            .map(|pickup| pickup.kind()),
        Some(PickupKind::GoldenFlame)
    );
    state.tick = Tick(30);
    state.actors[0].position = Position::center(TARGET);
    state.step(&[]).unwrap();

    assert!(state.world.pickups.pending.is_empty());
    assert!(state.world.pickups.exposed.is_empty());
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(31))
    );
}

#[test]
fn golden_expiry_overflow_rolls_back_input_explosion_and_collection_atomically() {
    let mut exact = state_with(
        exposed(PickupKind::GoldenFlame),
        Tick(u64::MAX - ORDINARY_EFFECT_TICKS - 1),
        TARGET,
    );
    exact.step(&[]).unwrap();
    assert!(
        exact.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(u64::MAX - 1))
    );
    assert!(
        !exact.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(u64::MAX))
    );

    let mut state = state_with(
        exposed(PickupKind::GoldenFlame),
        Tick(u64::MAX - ORDINARY_EFFECT_TICKS),
        TARGET,
    );
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
        Err(StepError::EffectExpiryOverflow(EffectKind::GoldenFlame))
    );
    assert_eq!(state, before);
}

#[test]
fn golden_pickups_are_rejected_in_every_layer_unless_enabled() {
    for layer in 0..3 {
        let pickup = BTreeMap::from([(TARGET, PickupKind::GoldenFlame)]);
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
        let result = State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART)),
            world,
            [
                actor(10, SeatId::One, CellId::new(1, 1)),
                actor(20, SeatId::Two, CellId::new(13, 11)),
            ],
        );
        assert_eq!(
            result,
            Err(super::StateError::UnsupportedPickup {
                cell: TARGET,
                kind: PickupKind::GoldenFlame,
            })
        );
    }
}

#[test]
fn elimination_clears_an_active_golden_flame() {
    let mut state = state_with(exposed(PickupKind::GoldenFlame), Tick(0), TARGET);
    state.step(&[]).unwrap();
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::GoldenFlame, Tick(1))
    );
    state.world.hazards.insert(TARGET, Tick(100));

    state.step(&[]).unwrap();

    assert!(!state.actors[0].alive());
    assert_eq!(state.actors[0].effects, Default::default());
}
