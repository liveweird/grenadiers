use grenadiers_core::{
    cpu::{
        MovementIntent,
        policy::{StrategicObjectiveKind, StrategicTarget},
        survival::WORK_ALLOWANCE,
    },
    game::{CellId, Direction},
};

use super::farm_yield_owner_support::run;

#[test]
fn current_hard58_records_the_low_yield_farm_choice_through_real_owner_execution() {
    let outcome = run(400);
    eprintln!("HARD58_FARM_YIELD {outcome:#?}");
    assert_eq!((outcome.initial_range, outcome.final_range), (2, 3));
    assert_eq!(outcome.strategy.objective, StrategicObjectiveKind::Farm);
    assert_eq!(
        outcome.strategy.target,
        StrategicTarget::Block {
            cell: CellId::new(3, 7),
        }
    );
    assert_eq!(outcome.strategy.approach_cell, Some(CellId::new(3, 6)));
    assert_eq!(
        outcome.intent.movement,
        MovementIntent::Direction(Direction::West)
    );
    assert!(outcome.intent.manual.is_none());
    assert!(!outcome.execution);
    assert_eq!(outcome.basis_tick.0, 183);
    assert_eq!(outcome.settled_stop_tick, outcome.basis_tick);
    assert_eq!(outcome.work, 21_485);
    assert_eq!(outcome.plant.cell, CellId::new(3, 5));
    assert_eq!(outcome.plant.range, 3);
    assert_eq!(
        (
            outcome.plant.placed_at.0,
            outcome.plant.detonates_at.0,
            outcome.plant.flame_end.0,
        ),
        (232, 412, 442)
    );
    assert_eq!(outcome.destroyed_blocks, 1);
    assert!(outcome.target_destroyed);
    assert!(!outcome.second_block_destroyed);
    assert!(outcome.open_flame_observed);
    assert!(outcome.resolved);
    assert!(outcome.cpu_death.is_none());
    assert_eq!((outcome.worker.dequeued, outcome.worker.delivered), (4, 4));
    assert_eq!(
        (
            outcome.worker.failed,
            outcome.worker.cancelled,
            outcome.worker.dropped,
        ),
        (0, 0, 0)
    );
    assert!(
        outcome
            .worker
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    let admissions = outcome.admissions;
    assert_eq!(
        [
            admissions.expired_or_wrong_decision,
            admissions.missed_manual_tick,
            admissions.stale_scene,
            admissions.invalid_hold,
            admissions.contact,
            admissions.strategic_plant,
            admissions.invalid_shape,
            admissions.pulse,
            admissions.route,
            admissions.duplicate_decision,
            admissions.refuge_handoff,
        ],
        [0; 11]
    );
}
