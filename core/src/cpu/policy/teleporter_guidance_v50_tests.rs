use super::*;
use crate::cpu::{ObservedActor, ObservedPickup, WorkBudget};
use crate::game::{CellId, PickupKind, Position};

fn captured() -> (Observation, UtilityMemory) {
    let observation = crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121();
    let key = crate::cpu::MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    (observation, UtilityMemory::new(key))
}

fn multiple() -> (Observation, UtilityMemory) {
    let (observation, memory) = captured();
    let observation = observation
        .map_parts(|parts| {
            parts.arena.blocks.clear();
            let recipient = parts.recipient;
            for actor in &mut parts.actors {
                let ObservedActor::Visible(actor) = actor else {
                    continue;
                };
                if actor.seat == recipient {
                    actor.position = Position::center(CellId::new(7, 7));
                } else {
                    actor.alive = false;
                }
            }
            parts.pickups.clear();
            parts.pickups.extend([
                ObservedPickup {
                    cell: CellId::new(1, 7),
                    kind: PickupKind::Teleporter,
                },
                ObservedPickup {
                    cell: CellId::new(13, 7),
                    kind: PickupKind::Teleporter,
                },
            ]);
        })
        .unwrap();
    (observation, memory)
}

#[test]
fn multiple_teleporters_choose_the_closest_reachable_approach_then_target_cell_order() {
    let (observation, memory) = multiple();
    let guidance =
        teleporter_guidance_v50(&observation, &memory, &mut WorkBudget::new(32_768)).unwrap();
    assert_eq!(guidance.target, CellId::new(1, 7));
    assert_eq!(guidance.approach, CellId::new(2, 7));
}

#[test]
fn valid_existing_target_is_stable_even_when_another_teleporter_becomes_closer() {
    let (basis, memory) = multiple();
    let first = teleporter_guidance_v50(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let moved = basis
        .map_parts(|parts| {
            let recipient = parts.recipient;
            let actor = parts
                .actors
                .iter_mut()
                .find_map(|actor| match actor {
                    ObservedActor::Visible(actor) if actor.seat == recipient => Some(actor),
                    _ => None,
                })
                .unwrap();
            actor.position = Position::center(CellId::new(11, 7));
        })
        .unwrap();
    let retained =
        teleporter_guidance_v50(&moved, &first.next_memory, &mut WorkBudget::new(32_768)).unwrap();
    assert_eq!(retained.target, first.target);
}

#[test]
fn removed_retained_target_selects_the_remaining_reachable_teleporter() {
    let (basis, memory) = multiple();
    let first = teleporter_guidance_v50(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let remaining = basis
        .map_parts(|parts| parts.pickups.retain(|pickup| pickup.cell != first.target))
        .unwrap();
    let replacement =
        teleporter_guidance_v50(&remaining, &first.next_memory, &mut WorkBudget::new(32_768))
            .unwrap();
    assert_eq!(replacement.target, CellId::new(13, 7));
}

#[test]
fn legacy_rejects_two_teleporters_while_v50_freezes_one_target() {
    let (observation, memory) = multiple();
    assert!(
        crate::cpu::policy::teleporter_guidance(
            &observation,
            &memory,
            &mut WorkBudget::new(32_768)
        )
        .is_none()
    );
    assert!(teleporter_guidance_v50(&observation, &memory, &mut WorkBudget::new(32_768)).is_some());
}

#[test]
fn one_teleporter_preserves_legacy_guidance_and_work() {
    let (observation, memory) = captured();
    let mut legacy_work = WorkBudget::new(32_768);
    let legacy = crate::cpu::policy::teleporter_guidance(&observation, &memory, &mut legacy_work);
    let mut v50_work = WorkBudget::new(32_768);
    let v50 = teleporter_guidance_v50(&observation, &memory, &mut v50_work);
    assert_eq!(v50, legacy);
    assert_eq!(v50_work.spent(), legacy_work.spent());
}

#[test]
fn zero_work_declines_without_charging() {
    let (observation, memory) = multiple();
    let mut work = WorkBudget::new(0);
    assert!(teleporter_guidance_v50(&observation, &memory, &mut work).is_none());
    assert_eq!(work.spent(), 0);
}
