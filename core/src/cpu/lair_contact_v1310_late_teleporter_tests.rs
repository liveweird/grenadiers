use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::policy::{ControlPlan, StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::survival::{
    LairSurvivalPlan, SurvivalPlan, TeleporterRecoveryChoice, WORK_ALLOWANCE,
    plan_hard_blocked_births_v43_with_origin, select_teleporter_recovery,
};
use crate::cpu::{CpuRng, Intent, MemoryKey, MovementIntent, WorkBudget};
use crate::game::{CellId, Direction, PickupKind, Position, Tick};

use super::v1310_late_teleporter_fixture::{CPU, captured_tick_5461};

// The public capture has no retained admitted Stop, physical Stop confirmation,
// private controller memory or RNG residual. These controlled characterizations
// explicitly assume a fresh memory/seed and a Stop basis at the captured tick;
// none of those premises is claimed as captured match history.
const ASSUMED_SETTLED_STOP_TICK: Tick = Tick(5_461);

fn basis() -> (crate::cpu::Observation, UtilityMemory, ObservedHazardMemory) {
    let observation = captured_tick_5461();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let memory = UtilityMemory::new(key);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    (observation, memory, history)
}

fn fallback(memory: UtilityMemory) -> LairSurvivalPlan {
    LairSurvivalPlan {
        survival: SurvivalPlan {
            control: ControlPlan {
                intent: Intent::moving(Direction::West),
                hold_ticks: 1,
                next_memory: memory,
            },
            lease: None,
            execution: None,
        },
        contact: None,
    }
}

#[test]
fn captured_public_scene_preserves_the_late_teleporter_observations() {
    let observation = captured_tick_5461();
    let parts = observation.parts();
    assert_eq!((parts.recipient, parts.tick), (CPU, Tick(5_461)));
    assert_eq!(
        (parts.rules.running_updates, parts.rules.remaining_ticks),
        (5_256, 5_544)
    );
    assert_eq!(parts.arena.blocks.len(), 48);
    assert!(matches!(&parts.bombs, crate::cpu::BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(parts.flames.is_empty() && parts.oil.is_empty());
    assert_eq!(parts.pickups.len(), 1);
    assert_eq!(
        (parts.pickups[0].cell, parts.pickups[0].kind),
        (CellId::new(3, 2), PickupKind::Teleporter)
    );
    let actor = parts
        .actors
        .iter()
        .find_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor) if actor.seat == CPU => Some(actor),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (actor.position, actor.heading, actor.alive),
        (Position { x: 1_536, y: 2_724 }, Direction::South, true)
    );
    assert_eq!(
        (actor.bomb_capacity, actor.live_bomb_count, actor.bomb_range),
        (1, 0, 2)
    );
    assert!(actor.equipment.is_none() && actor.weapon.is_none() && actor.effects.is_empty());
    assert!(actor.unknown == crate::cpu::UnknownActorFacts::ALL);
}

#[test]
fn controlled_hard47_worker_path_reports_shared_work_and_selection() {
    let (observation, memory, history) = basis();
    let key = memory.key();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let ordinary = plan_hard_blocked_births_v43_with_origin(
        &observation,
        &memory,
        &history,
        ASSUMED_SETTLED_STOP_TICK,
        None,
        &mut CpuRng::from_seed(key, [47; 32]),
        &mut work,
    )
    .unwrap();
    let fallback_spent = work.spent();
    let selected = select_teleporter_recovery(
        &observation,
        &history,
        ASSUMED_SETTLED_STOP_TICK,
        &memory,
        None,
        ordinary,
        &mut work,
    );
    assert_eq!(fallback_spent, 3_804);
    let TeleporterRecoveryChoice::Step(step) = selected else {
        panic!("the controlled Hard47 worker path should select the typed step")
    };
    let _typed_authority = step;
    assert_eq!(work.spent(), 14_176);

    let guidance = crate::cpu::policy::teleporter_guidance(
        &observation,
        &memory,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert_eq!(
        guidance.candidates[0].unwrap().movement,
        MovementIntent::Direction(Direction::North)
    );
    let intent = guidance.next_memory.strategic_intent().unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Collect);
    assert_eq!(
        intent.target,
        StrategicTarget::Pickup {
            cell: CellId::new(3, 2),
            kind: PickupKind::Teleporter
        }
    );

    let forecast = forecast(
        &observation,
        &history,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let certificate = crate::cpu::routing::certify_teleporter_step(
        &observation,
        &forecast,
        crate::cpu::routing::TeleporterStepRequest {
            target: CellId::new(3, 2),
            movement: MovementIntent::Direction(Direction::North),
            settled_stop_tick: ASSUMED_SETTLED_STOP_TICK,
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert_eq!(
        certificate.phase(),
        crate::cpu::routing::TeleporterStepPhase::Approach
    );
    assert_eq!(certificate.settled_stop_tick(), ASSUMED_SETTLED_STOP_TICK);
}

#[test]
fn controlled_direct_selector_reports_exact_boundary() {
    let (observation, memory, history) = basis();
    let mut measured = WorkBudget::new(WORK_ALLOWANCE);
    let choice = select_teleporter_recovery(
        &observation,
        &history,
        ASSUMED_SETTLED_STOP_TICK,
        &memory,
        None,
        fallback(memory),
        &mut measured,
    );
    assert!(matches!(choice, TeleporterRecoveryChoice::Step(_)));
    assert_eq!(measured.spent(), 10_372);

    let mut exact = WorkBudget::new(10_372);
    assert!(matches!(
        select_teleporter_recovery(
            &observation,
            &history,
            ASSUMED_SETTLED_STOP_TICK,
            &memory,
            None,
            fallback(memory),
            &mut exact,
        ),
        TeleporterRecoveryChoice::Step(_)
    ));
    assert_eq!(exact.spent(), 10_372);

    let mut one_short = WorkBudget::new(10_371);
    let short_choice = select_teleporter_recovery(
        &observation,
        &history,
        ASSUMED_SETTLED_STOP_TICK,
        &memory,
        None,
        fallback(memory),
        &mut one_short,
    );
    let TeleporterRecoveryChoice::Ordinary(short_fallback) = short_choice else {
        panic!("one-short selector work must preserve the ordinary fallback")
    };
    assert_eq!(
        short_fallback.survival.control,
        fallback(memory).survival.control
    );
    assert!(short_fallback.survival.lease.is_none());
    assert!(short_fallback.survival.execution.is_none());
    assert!(short_fallback.contact.is_none());
    assert!(one_short.spent() < 10_372);
}
