use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, Observation, ObservedActor, ObservedEnvironment,
    ObservedPickup, WorkBudget,
};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{CellId, Direction, PickupKind, Position, StormCloudConfig, Tick};

use super::refuge_centering_test_fixture::{CPU, captured_scene};
use super::{
    WORK_ALLOWANCE, plan_blocked_births_v42_with_origin, plan_hard_blocked_births_v43_with_origin,
    plan_hard_pit_avoidance_v53_with_origin, plan_pit_avoidance_v52_with_origin,
};

fn pit_pickup_scene(with_lairs: bool) -> Observation {
    let basis = captured_scene(CPU);
    let pits = basis.spike_pits().unwrap().clone();
    let lairs = with_lairs.then(|| basis.respawn_lairs().unwrap().clone());
    let mut parts = basis.into_parts();
    parts.bombs = BombKnowledge::Visible(Vec::new());
    parts.flames.clear();
    parts.pickups = vec![ObservedPickup {
        cell: CellId::new(5, 5),
        kind: PickupKind::BombCapacity,
    }];
    let actor_count = parts.actors.len();
    let actor = parts
        .actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
            _ => None,
        })
        .unwrap();
    actor.position = Position {
        x: 5 * UNITS_PER_CELL - HALF_FOOTPRINT,
        y: Position::center(CellId::new(5, 5)).y,
    };
    actor.heading = Direction::East;
    actor.live_bomb_count = 0;
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); actor_count]),
        lairs,
        Some(pits),
    )
    .unwrap()
}

fn key(observation: &Observation) -> MemoryKey {
    MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    }
}

#[test]
fn pit_avoidance_profiles_select_a_safe_short_pulse_alternative() {
    let observation = pit_pickup_scene(true);
    let key = key(&observation);
    for (hard, seed) in [(false, 52), (true, 53)] {
        let legacy = if hard {
            plan_hard_blocked_births_v43_with_origin(
                &observation,
                &UtilityMemory::new(key),
                &ObservedHazardMemory::new(key),
                Tick(540),
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
        } else {
            plan_blocked_births_v42_with_origin(
                &observation,
                &UtilityMemory::new(key),
                &ObservedHazardMemory::new(key),
                Tick(540),
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
        }
        .unwrap();
        assert_eq!(
            legacy.survival.control.intent.movement,
            crate::cpu::MovementIntent::Direction(Direction::East),
        );
        assert_eq!(
            crate::cpu::routing::pit_clear(
                &observation,
                legacy.survival.lease.as_ref().expect("legacy pulse"),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            ),
            Ok(false),
        );
        let planned = if hard {
            plan_hard_pit_avoidance_v53_with_origin(
                &observation,
                &UtilityMemory::new(key),
                &ObservedHazardMemory::new(key),
                Tick(540),
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
        } else {
            plan_pit_avoidance_v52_with_origin(
                &observation,
                &UtilityMemory::new(key),
                &ObservedHazardMemory::new(key),
                Tick(540),
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
        }
        .unwrap();
        let lease = planned.survival.lease.as_ref().expect("short pulse");
        assert_eq!(
            crate::cpu::routing::pit_clear(
                &observation,
                lease,
                &mut WorkBudget::new(WORK_ALLOWANCE),
            ),
            Ok(true),
        );
        assert_ne!(
            planned.survival.control.intent.movement,
            crate::cpu::MovementIntent::Direction(Direction::East),
        );
    }
}

#[test]
fn no_lair_fallback_also_rejects_the_pit_entry_pulse() {
    let observation = pit_pickup_scene(false);
    let key = key(&observation);
    let legacy = plan_blocked_births_v42_with_origin(
        &observation,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(540),
        None,
        &mut CpuRng::from_seed(key, [52; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert_eq!(
        legacy.survival.control.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::East),
    );
    assert_eq!(
        crate::cpu::routing::pit_clear(
            &observation,
            legacy.survival.lease.as_ref().expect("legacy pulse"),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ),
        Ok(false),
    );
    let planned = plan_pit_avoidance_v52_with_origin(
        &observation,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        Tick(540),
        None,
        &mut CpuRng::from_seed(key, [52; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    if let Some(lease) = &planned.survival.lease {
        assert_eq!(
            crate::cpu::routing::pit_clear(
                &observation,
                lease,
                &mut WorkBudget::new(WORK_ALLOWANCE),
            ),
            Ok(true),
        );
    }
    assert_ne!(
        planned.survival.control.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::East),
    );
}

#[test]
fn opening_and_pickup_handoff_refuse_the_unsafe_ordinary_pulse() {
    let observation = pit_pickup_scene(true);
    let key = key(&observation);
    let history = ObservedHazardMemory::new(key);
    let ordinary = crate::cpu::policy::plan_farm_v38(
        &observation,
        &UtilityMemory::new(key),
        &mut CpuRng::from_seed(key, [52; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert_eq!(
        ordinary.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::East),
    );
    let hazards = forecast(
        &observation,
        &history,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let window = TickWindow {
        start: Tick(541),
        end: Tick(548),
    };
    assert!(
        super::pickup_handoff::lair(
            &observation,
            &hazards,
            ordinary,
            true,
            window,
            548,
            Tick(540),
            true,
            false,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_some()
    );
    assert!(
        super::pickup_handoff::lair(
            &observation,
            &hazards,
            ordinary,
            true,
            window,
            548,
            Tick(540),
            true,
            true,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_none()
    );
    assert!(
        super::lair_v23::opening(
            &observation,
            &hazards,
            &history,
            Tick(540),
            &UtilityMemory::new(key),
            &ordinary,
            true,
            window,
            548,
            true,
            true,
            false,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_some()
    );
    assert!(
        super::lair_v23::opening(
            &observation,
            &hazards,
            &history,
            Tick(540),
            &UtilityMemory::new(key),
            &ordinary,
            true,
            window,
            548,
            true,
            true,
            true,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_none()
    );
}
