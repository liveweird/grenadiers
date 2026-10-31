use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    STRATEGIC_COMMITMENT_TICKS, STRATEGIC_PROGRESS_TICKS, StrategicIntent, StrategicObjectiveKind,
    StrategicPhase, StrategicSwitchReason, StrategicTarget, UtilityMemory,
};
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedPickup, UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, EnabledItems, EntityId, PickupKind, Position, Tick};

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
fn v24_farming_from_its_approach_cell_plants_instead_of_staring_at_the_block() {
    let target = CellId::new(8, 7);
    let scene = observed(CellId::new(7, 7), None)
        .map_parts(|parts| parts.arena.blocks.push(target))
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
        &mut CpuRng::from_seed(key, [79; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        planned
            .survival
            .control
            .next_memory
            .strategic_intent()
            .map(|intent| (intent.target, intent.approach_cell)),
        Some((
            StrategicTarget::Block { cell: target },
            Some(CellId::new(7, 7))
        ))
    );
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(matches!(
        planned.survival.execution,
        Some(super::ExecutionCertificate::Plant(_))
    ));
}

#[test]
fn v25_farming_from_its_approach_cell_keeps_a_certified_plant() {
    for lairs in [false, true] {
        let target = CellId::new(8, 7);
        let mut scene = observed(CellId::new(7, 7), None)
            .map_parts(|parts| parts.arena.blocks.push(target))
            .unwrap();
        if lairs {
            scene = super::rules_v9_occupancy_tests::with_lairs(scene, false);
        }
        let key = MemoryKey {
            world: scene.parts().world,
            cpu_seat: scene.parts().recipient,
        };
        let planned = super::plan_respawn_v25_with_origin(
            &scene,
            &UtilityMemory::new(key),
            &ObservedHazardMemory::new(key),
            scene.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [81; 32]),
            &mut WorkBudget::new(super::WORK_ALLOWANCE),
        )
        .unwrap();
        assert_eq!(
            planned.survival.control.intent.manual,
            Some(crate::cpu::ManualIntent::Bomb),
            "lairs={lairs}"
        );
        assert!(
            matches!(
                planned.survival.execution,
                Some(super::ExecutionCertificate::Plant(_))
            ),
            "lairs={lairs}"
        );
    }
}

#[test]
fn v25_repositions_when_teleporter_uncertainty_withholds_a_farming_plant() {
    for lairs in [false, true] {
        let target = CellId::new(8, 7);
        let mut scene = observed(CellId::new(7, 7), None)
            .map_parts(|parts| {
                parts.rules.settings = parts
                    .rules
                    .settings
                    .with_enabled_items(EnabledItems::TELEPORTER);
                parts.arena.blocks.push(target);
                parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                    id: EntityId(91),
                    placed_at: Tick(0),
                    cell: CellId::new(13, 11),
                    owner: EntityId(1),
                    range: 1,
                    trigger: ObservedBombTrigger::Remote,
                    blast: BlastKind::Standard,
                    unknown: UnknownBombFacts::ALL,
                }]);
            })
            .unwrap();
        if lairs {
            scene = super::rules_v9_occupancy_tests::with_lairs(scene, false);
        }
        let key = MemoryKey {
            world: scene.parts().world,
            cpu_seat: scene.parts().recipient,
        };
        let memory = UtilityMemory::new(key);
        let archived = super::plan_respawn_v24_with_origin(
            &scene,
            &memory,
            &ObservedHazardMemory::new(key),
            scene.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [80; 32]),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(archived.survival.control.intent.manual, None);
        assert_eq!(
            archived.survival.control.intent.movement,
            crate::cpu::MovementIntent::Stop
        );

        let recovered = super::plan_respawn_v25_with_origin(
            &scene,
            &memory,
            &ObservedHazardMemory::new(key),
            scene.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [80; 32]),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(recovered.survival.control.intent.manual, None);
        assert_ne!(
            recovered.survival.control.intent.movement,
            crate::cpu::MovementIntent::Stop,
            "lairs={lairs}"
        );
        assert_eq!(
            recovered
                .survival
                .control
                .next_memory
                .failed_target()
                .map(|(target, _)| target),
            Some(StrategicTarget::Block { cell: target }),
            "lairs={lairs}"
        );
        let moved_position = recovered
            .survival
            .lease
            .as_ref()
            .map(|lease| lease.pose_after(lease.hold_ticks()).min())
            .expect("recovery movement must have a certified lease");
        assert_ne!(moved_position, Position::center(CellId::new(7, 7)));
        let elapsed = u64::from(recovered.survival.control.hold_ticks);

        let deferred_memory = recovered.survival.control.next_memory;
        let later = scene
            .clone()
            .map_parts(|parts| {
                parts.tick = Tick(parts.tick.0 + elapsed);
                let recipient = parts.recipient;
                let actor = parts
                    .actors
                    .iter_mut()
                    .find_map(|actor| match actor {
                        ObservedActor::Visible(actor) if actor.seat == recipient => Some(actor),
                        _ => None,
                    })
                    .unwrap();
                actor.position = moved_position;
            })
            .unwrap();
        let mut later_history = ObservedHazardMemory::new(key);
        later_history.observe(&scene).unwrap();
        later_history.observe(&later).unwrap();
        let replanned = super::plan_respawn_v25_with_origin(
            &later,
            &deferred_memory,
            &later_history,
            later.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [80; 32]),
            &mut WorkBudget::new(super::WORK_ALLOWANCE),
        )
        .unwrap();
        let next_intent = replanned
            .survival
            .control
            .next_memory
            .strategic_intent()
            .expect("recovery should select another bounded objective");
        assert_ne!(
            next_intent.target,
            StrategicTarget::Block { cell: target },
            "lairs={lairs}"
        );
        assert_eq!(
            next_intent.last_switch_reason,
            StrategicSwitchReason::Stalled,
            "lairs={lairs}"
        );
    }
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
