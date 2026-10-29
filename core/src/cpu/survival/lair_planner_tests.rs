use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::lair_contact::tests::{mob, observation};
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    CpuRng, ManualIntent, MemoryKey, MovementIntent, ObservedActor, ObservedPickup, WorkBudget,
};
use crate::game::{CellId, PickupKind, Position, Tick};

use super::plan_lairs;

fn key(observation: &crate::cpu::Observation) -> MemoryKey {
    MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    }
}

#[test]
fn clear_pickup_navigation_keeps_ordinary_collection_with_a_contact_lease() {
    let observed = observation(Vec::new(), Tick(200))
        .map_parts(|parts| {
            parts.pickups.push(ObservedPickup {
                cell: CellId::new(2, 1),
                kind: PickupKind::MovementSpeed,
            });
        })
        .unwrap();
    let key = key(&observed);
    let mut work = WorkBudget::new(65_536);
    let plan = plan_lairs(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(100),
        &mut CpuRng::from_seed(key, [7; 32]),
        &mut work,
    )
    .unwrap();
    assert_ne!(
        plan.survival.control.intent.movement,
        MovementIntent::Stop,
        "spent {}",
        work.spent()
    );
    assert_eq!(plan.survival.control.intent.manual, None);
    assert!(plan.survival.lease.is_none());
    assert!(plan.contact.is_some());
}

#[test]
fn imminent_stationary_contact_selects_safe_movement_instead_of_waiting() {
    let actor_position = Position::center(CellId::new(3, 3));
    let mob_position = Position {
        x: actor_position.x + 1_050,
        y: actor_position.y,
    };
    let observed = observation(vec![mob(10, mob_position)], Tick(200))
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                unreachable!()
            };
            actor.position = actor_position;
        })
        .unwrap();
    let key = key(&observed);
    let mut work = WorkBudget::new(65_536);
    let plan = plan_lairs(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(100),
        &mut CpuRng::from_seed(key, [9; 32]),
        &mut work,
    )
    .unwrap();
    assert_ne!(
        plan.survival.control.intent.movement,
        MovementIntent::Stop,
        "spent {}",
        work.spent()
    );
    assert_eq!(plan.survival.control.intent.manual, None);
}

#[test]
fn exhausted_lair_planning_never_emits_manual_input() {
    let observed = observation(Vec::new(), Tick(200));
    let key = key(&observed);
    let plan = plan_lairs(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(100),
        &mut CpuRng::from_seed(key, [11; 32]),
        &mut WorkBudget::new(1),
    )
    .unwrap();
    assert_eq!(plan.survival.control.intent.manual, None);
}

fn plant_scene(mob_position: Option<Position>) -> crate::cpu::Observation {
    let mobs = mob_position
        .into_iter()
        .map(|position| mob(10, position))
        .collect();
    observation(mobs, Tick(400))
        .map_parts(|parts| {
            parts.arena.blocks.push(CellId::new(4, 3));
            let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                unreachable!()
            };
            actor.position = Position::center(CellId::new(3, 3));
        })
        .unwrap()
}

fn planned(observed: &crate::cpu::Observation) -> super::LairSurvivalPlan {
    let key = key(observed);
    plan_lairs(
        observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(100),
        &mut CpuRng::from_seed(key, [13; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

#[test]
fn unsafe_optional_plant_is_withheld_while_safe_movement_continues() {
    let actor = Position::center(CellId::new(3, 3));
    let observed = plant_scene(Some(Position {
        x: actor.x + 1_050,
        y: actor.y,
    }));
    let plan = planned(&observed);
    assert_eq!(plan.survival.control.intent.manual, None);
    assert_ne!(plan.survival.control.intent.movement, MovementIntent::Stop);
}

#[test]
fn distant_mob_keeps_a_fully_certified_plant_escape_available() {
    let observed = plant_scene(Some(Position::center(CellId::new(13, 11))));
    let plan = planned(&observed);
    assert_eq!(
        plan.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert!(plan.survival.execution.is_some());
    assert!(plan.contact.is_some());
}

#[test]
fn optional_tactic_work_exhaustion_retains_the_precomputed_fallback() {
    let observed = plant_scene(None);
    let key = key(&observed);
    let plan = plan_lairs(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(100),
        &mut CpuRng::from_seed(key, [15; 32]),
        &mut WorkBudget::new(5_000),
    )
    .unwrap();
    assert_eq!(plan.survival.control.intent.manual, None);
    assert_ne!(plan.survival.control.intent.movement, MovementIntent::Stop);
    assert!(plan.contact.is_some());
}
