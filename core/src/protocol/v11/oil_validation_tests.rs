use crate::protocol::{Cell, Position, Tick};

use super::super::v8::projection_facts::{OilPhase, ProjectionError, WorldFacts};
use super::{EquipmentId, FlameView, ItemId, ItemSet, PickupKind, PickupView, expanded_items};

fn facts() -> WorldFacts {
    let state = super::oil_tests::oil_state();
    super::capture_expanded(&state, &[]).unwrap().world.0
}

fn validate(facts: WorldFacts) -> Result<(), ProjectionError> {
    crate::protocol::v8::projection_validation::validate_world_for_items(facts, expanded_items())
        .map(|_| ())
}

#[test]
fn phase_block_layer_and_fire_invariants_are_checked_from_private_facts() {
    let control = facts();
    assert_eq!(validate(control.clone()), Ok(()));

    let mut retained_block = control.clone();
    retained_block.oil[2].phase = OilPhase::Pending;
    assert_eq!(validate(retained_block), Err(ProjectionError::InvalidItems));

    let mut missing_block = control.clone();
    missing_block.oil[0].phase = OilPhase::Concealed;
    assert_eq!(validate(missing_block), Err(ProjectionError::InvalidItems));

    let mut duplicate_fire = control.clone();
    duplicate_fire.flames.push(FlameView {
        cell: Cell::new(3, 3).unwrap(),
        expires_at: Tick::new(1).unwrap(),
    });
    assert_eq!(validate(duplicate_fire), Err(ProjectionError::InvalidItems));

    let mut pickup_conflict = control.clone();
    pickup_conflict
        .private_pickup_cells
        .push(Cell::new(4, 1).unwrap());
    assert_eq!(
        validate(pickup_conflict),
        Err(ProjectionError::InvalidItems)
    );

    let mut exposed_oil = control;
    exposed_oil.pickups.push(PickupView {
        cell: Cell::new(2, 1).unwrap(),
        kind: PickupKind::OilBarrel,
    });
    exposed_oil
        .private_pickup_cells
        .push(Cell::new(2, 1).unwrap());
    assert_eq!(validate(exposed_oil), Err(ProjectionError::InvalidPickups));
}

#[test]
fn unlit_overlap_requires_the_actors_own_capability_or_exact_typed_grace() {
    let mut overlap = facts();
    overlap.actors[0].position = Position::new(4 * 1_024 + 512, 1_536).unwrap();
    assert_eq!(
        validate(overlap.clone()),
        Err(ProjectionError::InvalidActors)
    );

    overlap.actors[0].inventory.equipment = Some(EquipmentId::WallPass);
    assert_eq!(validate(overlap.clone()), Ok(()));

    overlap.actors[0].inventory.equipment = None;
    overlap.unlit_oil_grace[0] = vec![Cell::new(4, 1).unwrap()];
    assert_eq!(validate(overlap.clone()), Ok(()));

    overlap.unlit_oil_grace[1] = overlap.unlit_oil_grace[0].clone();
    assert_eq!(validate(overlap), Err(ProjectionError::InvalidActors));
}

#[test]
fn pending_oil_may_remain_private_while_a_finite_flame_blocks_revelation() {
    let mut pending = facts();
    pending.flames.push(FlameView {
        cell: Cell::new(3, 1).unwrap(),
        expires_at: Tick::new(1).unwrap(),
    });
    assert_eq!(validate(pending), Ok(()));
}

#[test]
fn cap_order_duplicates_and_unsupported_private_facts_are_rejected() {
    let control = facts();

    let mut duplicate = control.clone();
    duplicate.oil.push(duplicate.oil[0]);
    assert_eq!(validate(duplicate), Err(ProjectionError::InvalidItems));

    let mut over_cap = control.clone();
    while over_cap.oil.len() <= crate::game::MAX_OIL_BARRELS {
        let column = u8::try_from(over_cap.oil.len() + 1).unwrap();
        over_cap
            .oil
            .push(super::super::v8::projection_facts::OilFacts {
                cell: Cell::new(column, 5).unwrap(),
                phase: OilPhase::Pending,
            });
    }
    over_cap
        .oil
        .sort_by_key(|oil| (oil.cell.row(), oil.cell.column()));
    assert_eq!(validate(over_cap), Err(ProjectionError::InvalidItems));

    let mut unsupported = control;
    unsupported.available_items = ItemSet::new(
        expanded_items()
            .iter()
            .filter(|item| *item != ItemId::OilBarrel),
    )
    .unwrap();
    assert_eq!(validate(unsupported), Err(ProjectionError::InvalidItems));
}
