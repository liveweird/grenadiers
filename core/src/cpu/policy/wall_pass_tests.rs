use crate::cpu::{CpuRng, Observation, ObservedActor, ObservedPickup, WorkBudget};
use crate::game::{CellId, EnabledItems, Equipment, PickupKind};

use super::goals::{legal_opening, select, transit_allowed};
use super::hazards::planting_supported;
use super::tests::{key, observation};

fn with_wall_pass(position: CellId) -> Observation {
    let mut parts = observation(position).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::WALL_PASS);
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.equipment = Some(Equipment::WallPass);
    Observation::new(parts).unwrap()
}

fn actor(observation: &Observation) -> &crate::cpu::VisibleActor {
    let ObservedActor::Visible(actor) = &observation.parts().actors[1] else {
        unreachable!()
    };
    actor
}

#[test]
fn holder_crosses_blocks_but_opening_goal_stays_on_plantable_floor() {
    let current = CellId::new(7, 7);
    let block = CellId::new(8, 7);
    let mut parts = with_wall_pass(current).into_parts();
    parts.arena.blocks.push(block);
    let observed = Observation::new(parts).unwrap();
    assert!(transit_allowed(&observed, actor(&observed), block));
    assert!(!planting_supported(&observed));

    assert!(!legal_opening(&observed, block));
    assert!(legal_opening(&observed, CellId::new(8, 6)));
}

#[test]
fn wall_pass_is_sought_once_and_then_preserved_from_other_equipment() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::WALL_PASS);
    parts.pickups.push(ObservedPickup {
        cell: CellId::new(8, 7),
        kind: PickupKind::WallPass,
    });
    let available = Observation::new(parts).unwrap();
    assert!(transit_allowed(
        &available,
        actor(&available),
        CellId::new(8, 7)
    ));
    let mut rng = CpuRng::from_seed(key(), [43; 32]);
    assert_eq!(
        select(
            &available,
            actor(&available),
            CellId::new(7, 7),
            &mut rng,
            &mut WorkBudget::new(100),
        )
        .unwrap()
        .first_step,
        CellId::new(8, 7)
    );

    let mut held = with_wall_pass(CellId::new(7, 7)).into_parts();
    for (column, kind) in [
        (8, PickupKind::WallPass),
        (9, PickupKind::Heart),
        (10, PickupKind::Kick),
    ] {
        held.pickups.push(ObservedPickup {
            cell: CellId::new(column, 7),
            kind,
        });
    }
    let held = Observation::new(held).unwrap();
    assert!(transit_allowed(&held, actor(&held), CellId::new(8, 7)));
    assert!(!transit_allowed(&held, actor(&held), CellId::new(9, 7)));
    assert!(!transit_allowed(&held, actor(&held), CellId::new(10, 7)));
}
