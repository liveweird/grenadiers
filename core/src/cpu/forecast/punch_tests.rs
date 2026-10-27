use crate::cpu::{
    BombKnowledge, Observation, ObservedBombTrigger, ObservedFlame, ObservedPickup, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, PickupKind, Tick};

use super::tests::{key, observation, timed};
use super::{ForecastFailure, ForecastLimits, ObservedHazardMemory, forecast_punch_branch};

const SOURCE: EntityId = EntityId(41);
const SOURCE_CELL: CellId = CellId::new(3, 1);
const LANDING: CellId = CellId::new(4, 1);

fn basis() -> Observation {
    let mut bomb = timed(SOURCE.0, SOURCE_CELL, 60, BlastKind::Standard);
    bomb.placed_at = Tick(3);
    bomb.owner = EntityId(10);
    bomb.range = 2;
    observation(10, BombKnowledge::Visible(vec![bomb]))
}

fn branch(
    observed: &Observation,
    work: &mut WorkBudget,
) -> Result<super::punch::PunchForecastBranch, ForecastFailure> {
    forecast_punch_branch(
        observed,
        &ObservedHazardMemory::new(key()),
        SOURCE,
        LANDING,
        Tick(17),
        ForecastLimits::default(),
        work,
    )
}

#[test]
fn punch_branch_relocates_the_source_without_changing_forecast_header() {
    let original = basis();
    let result = branch(&original, &mut WorkBudget::new(65_536)).unwrap();

    assert_eq!(result.forecast().basis_tick(), original.parts().tick);
    assert_eq!(result.forecast().key(), key());
    assert_eq!(result.forecast().horizon_end(), Tick(850));
    assert_eq!(
        result.scenario_basis().parts().bombs,
        BombKnowledge::Visible(Vec::new())
    );
    assert_eq!(
        result
            .forecast()
            .evolution()
            .expected_live_bombs(EntityId(10), Tick(17)),
        Some(1)
    );
    assert_ne!(result.hypothetical_id(), SOURCE);
}

#[test]
fn punch_branch_uses_execution_as_activation_and_original_absolute_fuse() {
    let result = branch(&basis(), &mut WorkBudget::new(65_536)).unwrap();
    let blocking = &result.forecast().cell(LANDING).unwrap().blocking;
    assert!(!blocking.iter().any(|window| window.contains(Tick(17))));
    assert!(blocking.iter().any(|window| window.contains(Tick(18))));
    let blast = result
        .forecast()
        .cell(CellId::new(5, 1))
        .unwrap()
        .lethal
        .as_slice();
    assert!(blast.iter().any(|window| window.contains(Tick(70))));
    assert!(blast.iter().all(|window| !window.contains(Tick(69))));
}

#[test]
fn relocated_evolution_maps_the_synthetic_bomb_back_to_the_actual_identity() {
    let original = basis();
    let result = branch(&original, &mut WorkBudget::new(65_536)).unwrap();

    let mut awaiting = original.clone().into_parts();
    awaiting.tick = Tick(16);
    awaiting.rules.running_updates += 6;
    awaiting.rules.remaining_ticks -= 6;
    let BombKnowledge::Visible(awaiting_bombs) = &mut awaiting.bombs else {
        unreachable!();
    };
    awaiting_bombs[0].trigger = ObservedBombTrigger::Timed {
        remaining_ticks: 54,
        paused: false,
    };
    let awaiting = Observation::new(awaiting).unwrap();
    assert!(
        result
            .forecast()
            .evolution()
            .matches_scene_with_hypothetical(
                result.scenario_basis(),
                &awaiting,
                result.hypothetical_id(),
                SOURCE,
            )
    );

    let mut current = original.clone().into_parts();
    current.tick = Tick(18);
    current.rules.running_updates += 8;
    current.rules.remaining_ticks -= 8;
    let BombKnowledge::Visible(bombs) = &mut current.bombs else {
        unreachable!();
    };
    bombs[0].cell = LANDING;
    bombs[0].trigger = ObservedBombTrigger::Timed {
        remaining_ticks: 52,
        paused: false,
    };
    let current = Observation::new(current).unwrap();

    assert!(
        result
            .forecast()
            .evolution()
            .matches_scene_with_hypothetical(
                result.scenario_basis(),
                &current,
                result.hypothetical_id(),
                SOURCE,
            )
    );
    assert!(
        !result
            .forecast()
            .evolution()
            .matches_scene(result.scenario_basis(), &current,)
    );
}

#[test]
fn punch_branch_rejects_wrong_source_and_restricted_scene_features() {
    assert!(matches!(
        forecast_punch_branch(
            &basis(),
            &ObservedHazardMemory::new(key()),
            EntityId(999),
            LANDING,
            Tick(17),
            ForecastLimits::default(),
            &mut WorkBudget::new(65_536),
        ),
        Err(ForecastFailure::InvalidHypothetical)
    ));

    let mut restricted = basis().into_parts();
    restricted.flames.push(ObservedFlame {
        cell: CellId::new(6, 1),
        remaining_ticks: 3,
    });
    restricted.pickups.push(ObservedPickup {
        cell: CellId::new(7, 1),
        kind: PickupKind::BombCapacity,
    });
    assert!(matches!(
        branch(
            &Observation::new(restricted).unwrap(),
            &mut WorkBudget::new(65_536),
        ),
        Err(ForecastFailure::UnsupportedRules)
    ));
}

#[test]
fn punch_branch_preserves_forecast_capacity_and_work_failures() {
    assert!(matches!(
        forecast_punch_branch(
            &basis(),
            &ObservedHazardMemory::new(key()),
            SOURCE,
            LANDING,
            Tick(17),
            ForecastLimits {
                horizon_ticks: 0,
                max_intervals_per_cell: 1,
            },
            &mut WorkBudget::new(65_536),
        ),
        Err(ForecastFailure::InvalidLimits)
    ));
    assert!(matches!(
        branch(&basis(), &mut WorkBudget::new(0)),
        Err(ForecastFailure::WorkExhausted)
    ));
}
