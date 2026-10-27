use crate::cpu::{ObservedActor, ObservedPickup};
use crate::game::{CellId, Equipment, PickupKind, SpecialWeapon};

use super::goals::{supported_weapon, transit_allowed};
use super::tests::observation;

fn actor(observation: &crate::cpu::Observation) -> &crate::cpu::VisibleActor {
    let ObservedActor::Visible(actor) = &observation.parts().actors[1] else {
        unreachable!()
    };
    actor
}

#[test]
fn supported_weapon_goals_replace_unusable_charges_and_allow_zero_value_transit() {
    for (held, pickup, expected) in [
        (None, PickupKind::PiercingBomb, (Some(7), true)),
        (
            Some(SpecialWeapon::Remote),
            PickupKind::PowerBomb,
            (Some(7), true),
        ),
        (
            Some(SpecialWeapon::Landmine),
            PickupKind::PiercingBomb,
            (Some(7), true),
        ),
        (
            Some(SpecialWeapon::Piercing),
            PickupKind::PiercingBomb,
            (None, true),
        ),
        (
            Some(SpecialWeapon::Power),
            PickupKind::PiercingBomb,
            (None, true),
        ),
    ] {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
            unreachable!()
        };
        cpu.weapon = held;
        parts.pickups.push(ObservedPickup {
            cell: CellId::new(8, 7),
            kind: pickup,
        });
        let observed = crate::cpu::Observation::new(parts).unwrap();
        assert_eq!(
            transit_allowed(&observed, actor(&observed), CellId::new(8, 7)),
            expected.1
        );
        assert_eq!(supported_weapon(actor(&observed), pickup), Some(expected));
    }
}

#[test]
fn heart_slot_does_not_block_supported_weapon_collection() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.weapon = Some(SpecialWeapon::Remote);
    cpu.equipment = Some(Equipment::Heart);
    parts.pickups.push(ObservedPickup {
        cell: CellId::new(8, 7),
        kind: PickupKind::PowerBomb,
    });
    let observed = crate::cpu::Observation::new(parts).unwrap();
    assert!(transit_allowed(
        &observed,
        actor(&observed),
        CellId::new(8, 7)
    ));
}
