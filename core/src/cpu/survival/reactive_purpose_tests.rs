use crate::cpu::forecast::{ForecastLimits, forecast};
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason, StrategicTarget,
};
use crate::cpu::{ObservedActor, WorkBudget};
use crate::game::{BlastKind, CellId, EntityId, Position, SeatId, Tick};

use super::WORK_ALLOWANCE;
use super::foreign_bomb_farming_clear_fixture::captured_clear_basis;
use super::reactive_purpose::{ReactivePlantPurpose, ReactivePurposeContext};

fn intent() -> StrategicIntent {
    StrategicIntent {
        objective: StrategicObjectiveKind::Attack,
        target: StrategicTarget::Opponent { seat: SeatId::One },
        approach_cell: Some(CellId::new(2, 1)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: Tick(420),
        score: 17,
        committed_until: Tick(520),
        progress_deadline: Tick(640),
        last_distance: 4,
        last_cpu_position: Position::center(CellId::new(7, 1)),
        deferred_until: Some(Tick(500)),
    }
}

fn bomb(cell: CellId) -> crate::cpu::routing::PlannedBomb {
    crate::cpu::routing::PlannedBomb {
        placed_at: Tick(439),
        owner: EntityId(2),
        cell,
        range: 2,
        blast: BlastKind::Standard,
        detonates_at: Tick(619),
    }
}

#[test]
fn attack_demolition_accepts_forward_block_and_rejects_rearward_geometry() {
    let captured = captured_clear_basis();
    let observation = captured.observation;
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let forecast = forecast(
        &observation,
        &captured.hazard_memory,
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let context = ReactivePurposeContext::new(&observation, intent().target, &mut work).unwrap();
    let forward = ReactivePlantPurpose::select(
        &observation,
        &context,
        bomb(CellId::new(7, 1)),
        forecast.evolution(),
        &mut work,
    )
    .expect("block (5,1) is forward from placement toward opponent (1,1)");
    assert!(
        ReactivePlantPurpose::select(
            &observation,
            &context,
            bomb(CellId::new(3, 1)),
            forecast.evolution(),
            &mut work
        )
        .is_none()
    );
    assert!(forward.still_targets(&observation, intent(), &bomb(CellId::new(7, 1)), &mut work));
}

#[test]
fn attack_demolition_rejects_changed_opponent_identity_cell_and_named_block() {
    let captured = captured_clear_basis();
    let observation = captured.observation;
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let forecast = forecast(
        &observation,
        &captured.hazard_memory,
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let context = ReactivePurposeContext::new(&observation, intent().target, &mut work).unwrap();
    let purpose = ReactivePlantPurpose::select(
        &observation,
        &context,
        bomb(CellId::new(7, 1)),
        forecast.evolution(),
        &mut work,
    )
    .unwrap();
    let changed_id = observation
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            actor.id = EntityId(99);
        })
        .unwrap();
    let changed_cell = observation
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            actor.position = Position::center(CellId::new(1, 3));
        })
        .unwrap();
    let missing_block = observation
        .clone()
        .map_parts(|parts| {
            parts.arena.blocks.retain(|cell| *cell != CellId::new(5, 1));
        })
        .unwrap();
    for changed in [changed_id, changed_cell, missing_block] {
        assert!(!purpose.still_targets(
            &changed,
            intent(),
            &bomb(CellId::new(7, 1)),
            &mut WorkBudget::new(WORK_ALLOWANCE)
        ));
    }
}

#[test]
fn plant_phase_preserves_the_primary_attack_commitment_fields() {
    let captured = captured_clear_basis();
    let original = intent();
    let marked = super::reactive_plant::mark_plant(
        captured.memory.with_strategic_intent(original),
        original,
    )
    .strategic_intent()
    .unwrap();
    assert_eq!(marked.phase, StrategicPhase::Plant);
    assert_eq!(
        StrategicIntent {
            phase: StrategicPhase::Approach,
            ..marked
        },
        original
    );
}
