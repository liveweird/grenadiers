use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, RoundResult, State};
use crate::game::arena::Arena;
use crate::game::input::Direction;
use crate::game::pickups::PickupKind;
use crate::game::rules::RulesConfig;
use crate::game::test_support::{add_test_bomb_with_deadline, open_border_arena};
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;
use crate::game::{DamagePreventionSource, EnabledItems, Equipment, StepEventKind};
use crate::room::RoomSettings;

const TARGET: CellId = CellId::new(5, 5);

#[path = "heart_damage_tests.rs"]
mod heart_damage_tests;
#[path = "shield_damage_tests.rs"]
mod shield_damage_tests;

fn arena_with_blocks(blocks: BTreeSet<CellId>) -> Arena {
    open_border_arena(blocks)
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

fn state_in(world: World) -> State {
    State::new(
        RulesConfig::classic_v1(),
        world,
        [
            actor(10, SeatId::One, CellId::new(1, 1)),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap()
}

fn expanded_state_in(world: World, first_cell: CellId) -> State {
    expanded_state_with_items(world, first_cell, EnabledItems::HEART)
}

fn expanded_state_with_items(
    world: World,
    first_cell: CellId,
    enabled_items: EnabledItems,
) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled_items)),
        world,
        [
            actor(10, SeatId::One, first_cell),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap()
}

fn concealed_state() -> State {
    let world = World::with_pickups(
        open_border_arena(BTreeSet::from([TARGET])),
        BTreeMap::from([(TARGET, PickupKind::BombCapacity)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    state_in(world)
}

fn add_bomb(state: &mut State, id: u32, cell: CellId, deadline: u64, range: u16) {
    add_test_bomb_with_deadline(
        state,
        EntityId(id),
        cell,
        EntityId(10),
        range,
        Tick(0),
        Tick(deadline),
    )
    .unwrap();
}

fn step_to(state: &mut State, tick: u64) {
    while state.tick().0 < tick {
        state.step(&[]).unwrap();
    }
}

#[test]
fn revealing_blast_moves_concealed_pickup_to_pending_without_burning_it() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);

    state.step(&[]).unwrap();

    assert!(!state.world().arena().blocks().contains(&TARGET));
    assert!(state.world.pickups.concealed.is_empty());
    assert_eq!(
        state
            .world
            .pickups
            .pending
            .get(&TARGET)
            .map(|item| item.kind()),
        Some(PickupKind::BombCapacity)
    );
    assert!(state.world().exposed_pickups().is_empty());
    assert_eq!(state.world().hazards().get(&TARGET), Some(&Tick(31)));
}

#[test]
fn pending_pickup_reveals_at_the_exclusive_flame_expiry() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);
    state.step(&[]).unwrap();

    step_to(&mut state, 30);
    assert!(state.world().exposed_pickups().is_empty());
    state.step(&[]).unwrap();

    assert!(!state.world().hazards().contains_key(&TARGET));
    assert!(state.world.pickups.pending.is_empty());
    assert_eq!(
        state
            .world()
            .exposed_pickups()
            .get(&TARGET)
            .map(|item| item.kind()),
        Some(PickupKind::BombCapacity)
    );
}

#[test]
fn renewed_overlapping_flame_delays_pending_reveal_to_the_latest_expiry() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);
    add_bomb(&mut state, 31, CellId::new(7, 5), 20, 2);

    step_to(&mut state, 20);
    assert_eq!(state.world().hazards().get(&TARGET), Some(&Tick(50)));
    step_to(&mut state, 49);
    assert!(state.world().exposed_pickups().is_empty());
    assert!(state.world.pickups.pending.contains_key(&TARGET));
    state.step(&[]).unwrap();

    assert!(state.world.pickups.pending.is_empty());
    assert!(state.world().exposed_pickups().contains_key(&TARGET));
}

#[test]
fn reveal_followed_by_a_new_blast_in_the_same_tick_burns_the_pickup() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);
    add_bomb(&mut state, 31, CellId::new(7, 5), 31, 2);

    step_to(&mut state, 31);

    assert!(state.world.pickups.concealed.is_empty());
    assert!(state.world.pickups.pending.is_empty());
    assert!(state.world().exposed_pickups().is_empty());
    assert_eq!(state.world().hazards().get(&TARGET), Some(&Tick(61)));
}

#[test]
fn exposed_pickup_burns_without_shielding_cells_behind_it() {
    let world = World::with_pickups(
        open_border_arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(TARGET, PickupKind::MovementSpeed)]),
    )
    .unwrap();
    let mut state = state_in(world);
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 4);

    state.step(&[]).unwrap();

    assert!(state.world().exposed_pickups().is_empty());
    assert!(state.world().hazards().contains_key(&CellId::new(6, 5)));
    assert!(state.world().hazards().contains_key(&CellId::new(7, 5)));
}

#[test]
fn concealed_and_pending_layers_do_not_change_public_state() {
    let blocked = open_border_arena(BTreeSet::from([TARGET]));
    let hidden = state_in(
        World::with_pickups(
            blocked.clone(),
            BTreeMap::from([(TARGET, PickupKind::BlastRange)]),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .unwrap(),
    );
    let no_hidden = state_in(World::new(blocked));
    assert_eq!(hidden.public_view(), no_hidden.public_view());
    assert!(!format!("{:?}", hidden.world()).contains("BlastRange"));

    let open = open_border_arena(BTreeSet::new());
    let pending = state_in(
        World::with_pickups(
            open.clone(),
            BTreeMap::new(),
            BTreeMap::from([(TARGET, PickupKind::BlastRange)]),
            BTreeMap::new(),
        )
        .unwrap(),
    );
    let no_pending = state_in(World::new(open));
    assert_eq!(pending.public_view(), no_pending.public_view());
}

#[test]
fn completed_result_freezes_pending_reveal_and_exposed_burn() {
    let open = open_border_arena(BTreeSet::new());
    let mut world = World::with_pickups(
        open,
        BTreeMap::new(),
        BTreeMap::from([(TARGET, PickupKind::BlastRange)]),
        BTreeMap::from([(CellId::new(6, 5), PickupKind::MovementSpeed)]),
    )
    .unwrap();
    world.hazards.insert(TARGET, Tick(2));
    world.hazards.insert(CellId::new(1, 1), Tick(10));
    let mut state = state_in(world);
    add_bomb(&mut state, 30, CellId::new(4, 5), 2, 2);
    state.step(&[]).unwrap();
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
    let frozen = state.world().clone();

    state.step(&[]).unwrap();

    assert_eq!(state.world(), &frozen);
    assert!(state.world.pickups.pending.contains_key(&TARGET));
    assert!(
        state
            .world()
            .exposed_pickups()
            .contains_key(&CellId::new(6, 5))
    );
}

#[test]
fn ordinary_heart_pickup_replaces_equipment_and_is_consumed_once() {
    let cell = CellId::new(5, 5);
    let world = World::with_pickups(
        open_border_arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cell, PickupKind::Heart)]),
    )
    .unwrap();
    let mut state = expanded_state_in(world, cell);
    state.actors[0].inventory.replace_equipment(Equipment::Kick);

    let collected = state.step(&[]).unwrap();
    assert!(collected.events.iter().any(|event| matches!(
        &event.kind,
        StepEventKind::PickupCollected {
            kind: PickupKind::Heart,
            ..
        }
    )));
    assert_eq!(
        state.actors()[0].inventory().equipment(),
        Some(Equipment::Heart)
    );

    state.world.hazards.insert(cell, Tick(100));
    let absorbed = state.step(&[]).unwrap();
    assert!(state.actors()[0].alive());
    assert_eq!(state.actors()[0].inventory().equipment(), None);
    assert_eq!(
        absorbed
            .events
            .iter()
            .filter(|event| matches!(
                &event.kind,
                StepEventKind::DamagePrevented {
                    source: DamagePreventionSource::Heart,
                    ..
                }
            ))
            .count(),
        1
    );
}

#[test]
fn ordinary_kick_pickup_uses_the_equipment_replacement_path() {
    let cell = CellId::new(5, 5);
    let world = World::with_pickups(
        open_border_arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cell, PickupKind::Kick)]),
    )
    .unwrap();
    let enabled =
        EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::WALL_PASS.bits())
            .unwrap();
    let mut state = expanded_state_with_items(world, cell, enabled);
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);

    let report = state.step(&[]).unwrap();

    assert_eq!(
        state.actors()[0].inventory().equipment(),
        Some(Equipment::Kick)
    );
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::PickupCollected {
            kind: PickupKind::Kick,
            ..
        }
    )));
}

#[test]
fn second_heart_during_grace_does_not_extend_or_repeat_prevention() {
    let cell = CellId::new(5, 5);
    let mut state = expanded_state_in(
        World::with_pickups(
            open_border_arena(BTreeSet::new()),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(cell, PickupKind::Heart)]),
        )
        .unwrap(),
        cell,
    );
    state.step(&[]).unwrap();
    state.world.hazards.insert(cell, Tick(100));
    state.step(&[]).unwrap();
    let original = state.actors[0].damage_grace.unwrap();
    state
        .world
        .pickups
        .exposed
        .insert(cell, crate::game::Pickup::new(PickupKind::Heart));

    let report = state.step(&[]).unwrap();

    assert_eq!(state.actors[0].damage_grace, Some(original));
    assert_eq!(
        state.actors()[0].inventory().equipment(),
        Some(Equipment::Heart)
    );
    assert!(
        !report
            .events
            .iter()
            .any(|event| matches!(&event.kind, StepEventKind::DamagePrevented { .. }))
    );
}
