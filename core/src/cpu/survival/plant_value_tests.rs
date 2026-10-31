use crate::cpu::policy::StrategicTarget;
use crate::cpu::policy::tests::observation;
use crate::cpu::{Observation, ObservedActor, ObservedOil, WorkBudget};
use crate::game::{
    BlastKind, CellId, EntityId, OilBarrelState, PickupKind, Position, SeatId, Tick,
};

use super::{ARENA_CELL_COUNT, could_be_useful, useful_plant, useful_plant_for};
use crate::cpu::routing::PlannedBomb;

fn with_rival_and_oil(rival: CellId, oil: Option<CellId>) -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(mut actor) = parts.actors[1].clone() else {
        unreachable!()
    };
    actor.id = EntityId(1);
    actor.seat = SeatId::One;
    actor.position = Position::center(rival);
    parts.actors[0] = ObservedActor::Visible(actor);
    if let Some(cell) = oil {
        parts.oil.push(ObservedOil {
            cell,
            state: OilBarrelState::Unlit,
        });
    }
    Observation::new(parts).unwrap()
}

fn bomb(blast: BlastKind) -> PlannedBomb {
    PlannedBomb {
        placed_at: Tick(17),
        owner: EntityId(2),
        cell: CellId::new(7, 7),
        range: 3,
        blast,
        detonates_at: Tick(197),
    }
}

#[test]
fn shared_geometry_values_piercing_past_oil_and_power_diagonals() {
    let piercing = with_rival_and_oil(CellId::new(9, 7), Some(CellId::new(8, 7)));
    assert_eq!(
        useful_plant(
            &piercing,
            &bomb(BlastKind::Standard),
            &mut WorkBudget::new(1_000)
        ),
        Some(false)
    );
    assert_eq!(
        useful_plant(
            &piercing,
            &bomb(BlastKind::Piercing),
            &mut WorkBudget::new(1_000)
        ),
        Some(true)
    );

    let diagonal = with_rival_and_oil(CellId::new(8, 8), None);
    assert!(could_be_useful(&diagonal, &mut WorkBudget::new(100)));
    assert_eq!(
        useful_plant(
            &diagonal,
            &bomb(BlastKind::Standard),
            &mut WorkBudget::new(1_000)
        ),
        Some(false)
    );
    assert_eq!(
        useful_plant(
            &diagonal,
            &bomb(BlastKind::Power),
            &mut WorkBudget::new(1_000)
        ),
        Some(true)
    );
}

#[test]
fn power_full_arena_scan_is_charged_before_value_can_succeed() {
    let diagonal = with_rival_and_oil(CellId::new(8, 8), None);
    assert_eq!(
        useful_plant(
            &diagonal,
            &bomb(BlastKind::Power),
            &mut WorkBudget::new(ARENA_CELL_COUNT as u32)
        ),
        None
    );
}

#[test]
fn strategic_plant_hits_only_the_selected_exact_subject() {
    let mut parts = with_rival_and_oil(CellId::new(9, 7), None).into_parts();
    parts
        .arena
        .blocks
        .extend([CellId::new(5, 7), CellId::new(6, 7), CellId::new(8, 7)]);
    let observation = Observation::new(parts).unwrap();
    assert_eq!(
        useful_plant_for(
            &observation,
            &bomb(BlastKind::Standard),
            StrategicTarget::Block {
                cell: CellId::new(8, 7)
            },
            &mut WorkBudget::new(1_000),
        ),
        Some(true)
    );
    assert_eq!(
        useful_plant_for(
            &observation,
            &bomb(BlastKind::Standard),
            StrategicTarget::Block {
                cell: CellId::new(5, 7)
            },
            &mut WorkBudget::new(1_000),
        ),
        Some(false),
        "destroying a different block is not target-directed value"
    );
    assert_eq!(
        useful_plant_for(
            &observation,
            &bomb(BlastKind::Standard),
            StrategicTarget::Opponent { seat: SeatId::One },
            &mut WorkBudget::new(1_000),
        ),
        Some(false),
        "the selected opponent is shielded by the exact intervening block"
    );
    assert_eq!(
        useful_plant_for(
            &observation,
            &bomb(BlastKind::Standard),
            StrategicTarget::Pickup {
                cell: CellId::new(7, 7),
                kind: PickupKind::BombCapacity,
            },
            &mut WorkBudget::new(1_000),
        ),
        Some(false),
        "incidental geometry cannot interrupt a collection commitment"
    );
}
