use std::collections::BTreeMap;

use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::{
    ActorInit, CellId, DamagePreventionSource, Direction, EffectKind, EnabledItems, EntityId,
    Equipment, EventAudience, InputAction, PickupKind, Position, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, StepError, StepEventKind, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn state_at(tick: Tick) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns().to_vec();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::InvisibilityCloak)]),
    )
    .unwrap();
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(
                EnabledItems::from_bits(
                    EnabledItems::INVISIBILITY_CLOAK.bits()
                        | EnabledItems::SHIELD.bits()
                        | EnabledItems::HEART.bits(),
                )
                .unwrap(),
            ),
        ),
        world,
        SeatId::DUEL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 1),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        tick,
    )
    .unwrap()
}

#[test]
fn same_tick_placement_precedes_private_acquisition_and_cloak_preserves_loadout() {
    let mut state = state_at(Tick(0));
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    state.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    assert_eq!(report.events.len(), 2);
    assert!(matches!(
        report.events[0].kind,
        StepEventKind::BombPlaced(_)
    ));
    assert_eq!(report.events[0].audience, EventAudience::Both);
    assert!(matches!(
        report.events[1].kind,
        StepEventKind::PickupCollected {
            kind: PickupKind::InvisibilityCloak,
            ..
        }
    ));
    assert_eq!(report.events[1].audience, EventAudience::One);
    assert_eq!(
        state.actors[0].inventory.equipment(),
        Some(Equipment::Heart)
    );
    assert!(state.actors[0].effects.active(EffectKind::Shield, Tick(1)));
    assert!(
        state.actors[0]
            .effects
            .active(EffectKind::InvisibilityCloak, Tick(1))
    );
}

#[test]
fn cloaked_damage_prevention_is_private_but_elimination_and_result_are_public() {
    let mut shielded = state_at(Tick(0));
    shielded.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    shielded.step(&[]).unwrap();
    let cell = shielded.actors[0].position.cell().unwrap();
    shielded.world.hazards.insert(cell, Tick(4));
    let prevention = shielded.step(&[]).unwrap();
    assert!(prevention.events.iter().any(|event| {
        event.audience == EventAudience::One
            && matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    source: DamagePreventionSource::Shield,
                    ..
                }
            )
    }));

    let mut killed = state_at(Tick(0));
    killed.step(&[]).unwrap();
    let cell = killed.actors[0].position.cell().unwrap();
    killed.world.hazards.insert(cell, Tick(4));
    let death = killed.step(&[]).unwrap();
    assert!(!killed.actors[0].alive);
    assert!(killed.actors[0].effects.observations().next().is_none());
    assert!(death.events.iter().all(|event| {
        !matches!(
            event.kind,
            StepEventKind::ActorEliminated { .. } | StepEventKind::RoundEnded(_)
        ) || event.audience == EventAudience::Both
    }));
}

#[test]
fn cloak_expiry_overflow_rolls_back_the_entire_step() {
    let mut state = state_at(Tick(u64::MAX - 1));
    let before = state.clone();
    assert_eq!(
        state.step(&[]),
        Err(StepError::EffectExpiryOverflow(
            EffectKind::InvisibilityCloak
        ))
    );
    assert_eq!(state, before);
}

#[test]
fn detonation_audience_observes_old_fire_death_but_precedes_final_flame_death() {
    let mut old_fire = state_at(Tick(0));
    old_fire.step(&[]).unwrap();
    let owner_cell = old_fire.actors[0].position.cell().unwrap();
    old_fire.world.hazards.insert(owner_cell, Tick(5));
    add_test_bomb_with_deadline(
        &mut old_fire,
        EntityId(30),
        CellId::new(7, 5),
        EntityId(1),
        1,
        Tick(0),
        Tick(2),
    )
    .unwrap();
    let report = old_fire.step(&[]).unwrap();
    assert!(!old_fire.actors[0].alive);
    assert!(report.events.iter().any(|event| event.audience == EventAudience::Both
        && matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))));

    let mut final_flame = state_at(Tick(0));
    final_flame.step(&[]).unwrap();
    let owner_cell = final_flame.actors[0].position.cell().unwrap();
    add_test_bomb_with_deadline(
        &mut final_flame,
        EntityId(30),
        owner_cell,
        EntityId(1),
        1,
        Tick(0),
        Tick(2),
    )
    .unwrap();
    let report = final_flame.step(&[]).unwrap();
    assert!(!final_flame.actors[0].alive);
    assert!(report.events.iter().any(|event| event.audience == EventAudience::One
        && matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))));
    assert!(
        report
            .events
            .iter()
            .any(|event| event.audience == EventAudience::Both
                && matches!(
                    event.kind,
                    StepEventKind::ActorEliminated {
                        id: EntityId(1),
                        ..
                    }
                ))
    );
}
