use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{StrategicObjectiveKind, StrategicPhase, StrategicTarget, UtilityMemory};
use crate::cpu::{BombKnowledge, CpuRng, ManualIntent, ObservedActor, ObservedEffect, WorkBudget};
use crate::game::{CellId, EffectKind, EnabledItems, Tick};
use crate::room::RoomSettings;

use super::farm_progress_tests::{captured_tick_7022, key, stalled_memory};
use super::{WORK_ALLOWANCE, plan_farm_origins_v38_with_origin};

fn plan(
    observation: &crate::cpu::Observation,
    memory: &UtilityMemory,
) -> (super::LairSurvivalPlan, u32) {
    let key = key(observation);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_farm_origins_v38_with_origin(
        observation,
        memory,
        &history,
        observation.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [138; 32]),
        &mut work,
    )
    .unwrap();
    (planned, work.spent())
}

#[test]
fn captured_refused_farm_origin_is_deferred_before_the_next_block_ranking() {
    let observed = captured_tick_7022();
    let first = plan(&observed, &stalled_memory(&observed)).0;
    assert_ne!(
        first.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert!(first.survival.execution.is_none());
    assert_eq!(
        first.survival.control.next_memory.failed_farm_approach(),
        Some((CellId::new(3, 9), Tick(7_142)))
    );

    let second = plan(&observed, &first.survival.control.next_memory).0;
    let intent = second
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    assert_eq!(
        intent.target,
        StrategicTarget::Block {
            cell: CellId::new(3, 7)
        }
    );
    assert_eq!(intent.approach_cell, Some(CellId::new(3, 8)));
    assert_ne!(intent.approach_cell, Some(CellId::new(3, 9)));
}

#[test]
fn expired_origin_memory_is_inert_and_a_new_world_starts_empty() {
    let observed = captured_tick_7022();
    let expired =
        UtilityMemory::new(key(&observed)).defer_farm_approach(CellId::new(3, 9), Tick(6_900));
    let planned = plan(&observed, &expired).0;
    assert_eq!(
        planned
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap()
            .approach_cell,
        Some(CellId::new(3, 9))
    );
    let other = crate::cpu::MemoryKey {
        world: crate::cpu::WorldId {
            room_instance: 999,
            physical_round: 2,
        },
        cpu_seat: observed.parts().recipient,
    };
    assert_eq!(UtilityMemory::new(other).failed_farm_approach(), None);
}

#[test]
fn unsupported_global_plant_states_do_not_defer_the_local_farm_origin() {
    let basis = captured_tick_7022();
    let full = basis
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[parts.recipient.index()] else {
                unreachable!()
            };
            cpu.live_bomb_count = cpu.bomb_capacity;
        })
        .unwrap();
    let affected = basis
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[parts.recipient.index()] else {
                unreachable!()
            };
            cpu.effects.push(ObservedEffect {
                kind: EffectKind::Diarrhea,
                remaining_ticks: 60,
            });
        })
        .unwrap();
    for observed in [full, affected] {
        let planned = plan(&observed, &stalled_memory(&observed)).0;
        assert_eq!(
            planned.survival.control.next_memory.failed_farm_approach(),
            None
        );
    }
}

#[test]
fn failed_origin_eligibility_charges_the_complete_block_scan_atomically() {
    let observed = captured_tick_7022();
    let key = key(&observed);
    let ordinary = crate::cpu::policy::plan_farm_v38(
        &observed,
        &stalled_memory(&observed),
        &mut CpuRng::from_seed(key, [140; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let required = u32::try_from(
        1 + observed.parts().pickups.len()
            + observed.parts().flames.len()
            + observed.parts().oil.len()
            + observed.parts().arena.blocks.len()
            + match &observed.parts().actors[observed.parts().recipient.index()] {
                ObservedActor::Visible(actor) => actor.effects.len(),
                _ => unreachable!(),
            },
    )
    .unwrap();
    let mut short = WorkBudget::new(required - 1);
    assert_eq!(
        super::joint_refuge::eligible_failed_farm_approach(
            &observed,
            observed.parts().tick,
            &ordinary,
            super::plant_tactic::PlantSupport::PersistentBurningOil,
            &mut short,
        ),
        None
    );
    assert_eq!(short.spent(), 0);
    let mut exact = WorkBudget::new(required);
    assert_eq!(
        super::joint_refuge::eligible_failed_farm_approach(
            &observed,
            observed.parts().tick,
            &ordinary,
            super::plant_tactic::PlantSupport::PersistentBurningOil,
            &mut exact,
        ),
        Some(CellId::new(3, 9))
    );
    assert_eq!(exact.spent(), required);
}

#[test]
fn failed_origin_eligibility_rejects_unknown_bombs_and_nonfarm_phases() {
    let observed = captured_tick_7022();
    let key = key(&observed);
    let ordinary = crate::cpu::policy::plan_farm_v38(
        &observed,
        &stalled_memory(&observed),
        &mut CpuRng::from_seed(key, [141; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let unknown = observed
        .clone()
        .map_parts(|parts| {
            parts.rules.settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::EMPTY);
            parts.bombs = BombKnowledge::Unavailable;
        })
        .unwrap();
    let mut malformed = ordinary;
    let mut intent = malformed.next_memory.strategic_intent().unwrap();
    intent.objective = StrategicObjectiveKind::Attack;
    malformed.next_memory = malformed.next_memory.with_strategic_intent(intent);
    let mut waiting = ordinary;
    let mut intent = waiting.next_memory.strategic_intent().unwrap();
    intent.phase = StrategicPhase::Waiting;
    waiting.next_memory = waiting.next_memory.with_strategic_intent(intent);

    for (basis, plan) in [
        (&unknown, &ordinary),
        (&observed, &malformed),
        (&observed, &waiting),
    ] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        assert_eq!(
            super::joint_refuge::eligible_failed_farm_approach(
                basis,
                basis.parts().tick,
                plan,
                super::plant_tactic::PlantSupport::PersistentBurningOil,
                &mut work,
            ),
            None
        );
        assert_eq!(work.spent(), 0);
    }
}
