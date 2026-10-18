use std::collections::{BTreeMap, BTreeSet};

use super::{TARGET, add_bomb, arena_with_blocks, expanded_state_in};
use crate::game::{
    CellId, DamagePreventionSource, EventAudience, PickupKind, Position, StepEventKind, Tick, World,
};

fn heart_world() -> World {
    World::with_pickups(
        arena_with_blocks(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(TARGET, PickupKind::Heart)]),
    )
    .unwrap()
}

#[test]
fn a_lethal_explosion_before_collection_cannot_be_undone_by_the_heart_underfoot() {
    let mut state = expanded_state_in(heart_world(), TARGET);
    add_bomb(&mut state, 30, CellId::new(4, 5), 1, 2);

    let report = state.step(&[]).unwrap();

    assert!(!state.actors()[0].alive());
    assert_eq!(state.actors()[0].inventory().equipment(), None);
    assert!(!state.world.exposed_pickups().contains_key(&TARGET));
    assert!(
        report
            .events
            .iter()
            .any(|event| matches!(event.kind, StepEventKind::ActorEliminated { .. }))
    );
    assert!(!report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::PickupCollected { .. } | StepEventKind::DamagePrevented { .. }
    )));
}

#[test]
fn overlapping_cells_and_both_damage_passes_consume_one_heart_until_exact_expiry() {
    let mut state = expanded_state_in(heart_world(), TARGET);
    state.step(&[]).unwrap();
    // The footprint crosses the shared face of two burning floor cells.
    state.actors[0].position = Position {
        x: 5 * 1024,
        y: 5 * 1024 + 512,
    };
    state.world.hazards.insert(CellId::new(4, 5), Tick(100));
    state.world.hazards.insert(TARGET, Tick(100));

    let absorbed = state.step(&[]).unwrap();
    let received_at = state.tick().0;
    let prevented = absorbed
        .events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    source: DamagePreventionSource::Heart,
                    ..
                }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(prevented.len(), 1);
    assert_eq!(prevented[0].audience, EventAudience::Both);
    assert!(state.actors()[0].alive());
    assert_eq!(state.actors()[0].inventory().equipment(), None);

    while state.tick().0 < received_at + 59 {
        let report = state.step(&[]).unwrap();
        assert!(
            !report
                .events
                .iter()
                .any(|event| matches!(event.kind, StepEventKind::DamagePrevented { .. }))
        );
    }
    assert!(state.actors()[0].alive());
    state.step(&[]).unwrap();
    assert_eq!(state.tick().0, received_at + 60);
    assert!(!state.actors()[0].alive());
}
