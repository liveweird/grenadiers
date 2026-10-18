use std::collections::{BTreeMap, BTreeSet};

use super::{TARGET, add_bomb, arena_with_blocks, expanded_state_with_items, step_to};
use crate::game::effects::{EffectKind, SHIELD_TICKS};
use crate::game::{
    CellId, DamagePreventionSource, EnabledItems, Equipment, EventAudience, Pickup, PickupKind,
    Position, State, StepError, StepEventKind, Tick, World,
};

fn shield_world(layer: usize) -> World {
    let shield = BTreeMap::from([(TARGET, PickupKind::Shield)]);
    World::with_pickups(
        arena_with_blocks(if layer == 0 {
            BTreeSet::from([TARGET])
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
    .unwrap()
}

fn shield_state(world: World, tick: Tick) -> State {
    State::new_at_tick(
        crate::game::RulesConfig::expanded_v5(
            crate::room::RoomSettings::DEFAULT.with_enabled_items(EnabledItems::SHIELD),
        ),
        world,
        [
            super::actor(10, crate::game::SeatId::One, TARGET),
            super::actor(20, crate::game::SeatId::Two, CellId::new(13, 11)),
        ],
        tick,
    )
    .unwrap()
}

#[test]
fn concealed_shield_follows_the_ordinary_reveal_and_collection_path() {
    let mut state =
        expanded_state_with_items(shield_world(0), CellId::new(1, 1), EnabledItems::SHIELD);
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);

    state.step(&[]).unwrap();
    assert_eq!(
        state
            .world
            .pickups
            .pending
            .get(&TARGET)
            .map(|pickup| pickup.kind()),
        Some(PickupKind::Shield)
    );
    step_to(&mut state, 30);
    state.actors[0].position = Position::center(TARGET);

    let report = state.step(&[]).unwrap();

    assert!(state.world.pickups.pending.is_empty());
    assert!(state.world().exposed_pickups().is_empty());
    assert!(
        state.actors()[0]
            .effects()
            .active(EffectKind::Shield, Tick(31))
    );
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::PickupCollected {
            kind: PickupKind::Shield,
            ..
        }
    )));
}

#[test]
fn collecting_again_refreshes_shield_from_the_collection_tick() {
    let mut state = shield_state(shield_world(2), Tick(10));
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    state.step(&[]).unwrap();
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(310))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(311))
    );

    step_to(&mut state, 20);
    state
        .world
        .pickups
        .exposed
        .insert(TARGET, Pickup::new(PickupKind::Shield));
    state.step(&[]).unwrap();

    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(320))
    );
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(321))
    );
    assert_eq!(
        state.actors[0].inventory().equipment(),
        Some(Equipment::Heart)
    );
}

#[test]
fn shield_reports_prevention_once_across_cells_and_both_damage_passes() {
    let mut state = shield_state(shield_world(2), Tick(0));
    state.step(&[]).unwrap();
    state.actors[0].position = Position {
        x: 5 * 1024,
        y: 5 * 1024 + 512,
    };
    state.world.hazards.insert(CellId::new(4, 5), Tick(100));
    state.world.hazards.insert(TARGET, Tick(100));

    let report = state.step(&[]).unwrap();
    let prevented = report
        .events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    source: DamagePreventionSource::Shield,
                    ..
                }
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(prevented.len(), 1);
    assert_eq!(prevented[0].audience, EventAudience::Both);
    assert!(state.actors()[0].alive());
}

#[test]
fn shield_expires_before_boundary_damage_then_heart_preserves_life_once() {
    let mut state = shield_state(shield_world(2), Tick(0));
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    state.step(&[]).unwrap();
    state.tick = Tick(SHIELD_TICKS);
    state.world.hazards.insert(TARGET, Tick(400));

    let report = state.step(&[]).unwrap();

    assert!(state.actors()[0].alive());
    assert!(
        !state.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(SHIELD_TICKS + 1))
    );
    assert_eq!(state.actors()[0].inventory().equipment(), None);
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::DamagePrevented {
            source: DamagePreventionSource::Heart,
            ..
        }
    )));

    state.tick = Tick(SHIELD_TICKS + 60);
    state.step(&[]).unwrap();
    assert!(!state.actors()[0].alive());
    assert_eq!(state.actors()[0].effects(), &Default::default());
    assert_eq!(state.actors()[0].inventory(), &Default::default());
}

#[test]
fn shield_expiry_overflow_rejects_the_whole_step_without_saturation() {
    let mut exact = shield_state(shield_world(2), Tick(u64::MAX - SHIELD_TICKS - 1));
    exact.step(&[]).unwrap();
    assert!(
        exact.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(u64::MAX - 1))
    );
    assert!(
        !exact.actors[0]
            .effects
            .active(EffectKind::Shield, Tick(u64::MAX))
    );

    let mut state = shield_state(shield_world(2), Tick(u64::MAX - SHIELD_TICKS));
    let before = state.clone();

    assert_eq!(
        state.step(&[]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(state, before);
}
