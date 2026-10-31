use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    STRATEGIC_COMMITMENT_TICKS, STRATEGIC_PROGRESS_TICKS, StrategicIntent, StrategicObjectiveKind,
    StrategicPhase, StrategicSwitchReason, StrategicTarget, UtilityMemory,
};
use crate::cpu::{CpuRng, MemoryKey, ObservedActor, ObservedPickup, WorkBudget};
use crate::game::{CellId, PickupKind, Position, Tick};

use super::rules_v9_occupancy_tests::observed;

#[test]
fn v24_waiting_phase_cannot_start_an_opportunistic_plant_with_spare_capacity() {
    let scene = observed(CellId::new(7, 7), None)
        .map_parts(|parts| parts.arena.blocks.push(CellId::new(8, 7)))
        .unwrap();
    let key = MemoryKey {
        world: scene.parts().world,
        cpu_seat: scene.parts().recipient,
    };
    let strategic = StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block {
            cell: CellId::new(8, 7),
        },
        approach_cell: Some(CellId::new(7, 7)),
        phase: StrategicPhase::Waiting,
        last_switch_reason: StrategicSwitchReason::SafetyInterrupt,
        observation_tick: scene.parts().tick,
        score: 2,
        committed_until: Tick(scene.parts().tick.0 + STRATEGIC_COMMITMENT_TICKS),
        progress_deadline: Tick(scene.parts().tick.0 + STRATEGIC_PROGRESS_TICKS),
        last_distance: 0,
        last_cpu_position: Position::center(CellId::new(7, 7)),
        deferred_until: None,
    };
    let planned = super::plan_respawn_v24_with_origin(
        &scene,
        &UtilityMemory::new(key).with_strategic_intent(strategic),
        &ObservedHazardMemory::new(key),
        scene.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [78; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(planned.survival.control.intent.manual, None);
    assert_eq!(
        planned.survival.control.intent.movement,
        crate::cpu::MovementIntent::Stop
    );
    assert!(!matches!(
        planned.survival.execution,
        Some(super::ExecutionCertificate::Plant(_))
    ));
    assert_eq!(
        planned
            .survival
            .control
            .next_memory
            .strategic_intent()
            .map(|intent| intent.target),
        Some(strategic.target)
    );
}

#[test]
fn v24_fresh_pickup_commitment_cannot_incidentally_bomb_a_useful_crate() {
    let scene = observed(CellId::new(7, 7), None)
        .map_parts(|parts| {
            parts.pickups.push(ObservedPickup {
                cell: CellId::new(7, 8),
                kind: PickupKind::GoldenFlame,
            });
            parts.arena.blocks.push(CellId::new(8, 7));
        })
        .unwrap();
    let key = MemoryKey {
        world: scene.parts().world,
        cpu_seat: scene.parts().recipient,
    };
    let planned = super::plan_respawn_v24_with_origin(
        &scene,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        scene.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [76; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(planned.survival.control.intent.manual, None);
    assert!(matches!(
        planned.survival.control.next_memory.strategic_intent(),
        Some(crate::cpu::policy::StrategicIntent {
            target: crate::cpu::policy::StrategicTarget::Pickup { cell, kind },
            ..
        }) if cell == CellId::new(7, 8) && kind == PickupKind::GoldenFlame
    ));
}

#[test]
fn v24_opponent_plant_revalidates_visible_alive_target_at_manual_boundary() {
    let scene = observed(CellId::new(9, 7), Some(CellId::new(11, 7)));
    let key = MemoryKey {
        world: scene.parts().world,
        cpu_seat: scene.parts().recipient,
    };
    let planned = super::plan_respawn_v24_with_origin(
        &scene,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        scene.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [77; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let intent = planned
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    let certificate = planned
        .survival
        .execution
        .as_ref()
        .and_then(super::ExecutionCertificate::plant)
        .expect("selected opponent should have a certified plant");
    assert!(super::strategic_plant_still_targets(
        &scene,
        intent,
        certificate,
        &mut WorkBudget::new(65_536),
    ));

    let moved = scene
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.position = Position::center(CellId::new(11, 9));
        })
        .unwrap();
    assert!(!super::strategic_plant_still_targets(
        &moved,
        intent,
        certificate,
        &mut WorkBudget::new(65_536),
    ));

    let hidden = observed(CellId::new(9, 7), None);
    assert!(!super::strategic_plant_still_targets(
        &hidden,
        intent,
        certificate,
        &mut WorkBudget::new(65_536),
    ));

    let dead = hidden
        .map_parts(|parts| {
            let ObservedActor::Hidden { alive, .. } = &mut parts.actors[0] else {
                unreachable!()
            };
            *alive = false;
        })
        .unwrap();
    assert!(!super::strategic_plant_still_targets(
        &dead,
        intent,
        certificate,
        &mut WorkBudget::new(65_536),
    ));
}
