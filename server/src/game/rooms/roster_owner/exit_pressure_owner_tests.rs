use grenadiers_core::{
    cpu::{
        MovementIntent,
        policy::{StrategicObjectiveKind, StrategicTarget},
        survival::WORK_ALLOWANCE,
    },
    game::{CellId, Direction, SeatId, Tick},
    room::CpuProfile,
};

use super::exit_pressure_owner_support::run;

#[test]
fn hard57_records_a_natural_range_upgrade_coordinate_selected_attack_and_escape() {
    let episode = run(CpuProfile::HARD_V57, 500);
    eprintln!(
        "HARD57_EXIT_PRESSURE basis={:?} work={} strategy={:?} plant={:?} admissions={:?}",
        episode.basis.observation.parts().tick,
        episode.basis.work,
        episode.basis.strategy,
        episode.plant,
        episode.admissions
    );
    assert_eq!((episode.initial_range, episode.final_range), (2, 3));
    let strategy = episode.basis.strategy.unwrap();
    assert_eq!(strategy.objective, StrategicObjectiveKind::Attack);
    assert_eq!(
        strategy.target,
        StrategicTarget::Opponent { seat: SeatId::One }
    );
    assert_eq!(strategy.approach_cell, Some(CellId::new(7, 5)));
    assert_eq!(
        episode.basis.intent.movement,
        MovementIntent::Direction(Direction::East)
    );
    assert!(episode.basis.intent.manual.is_none());
    assert!(!episode.basis.execution);
    assert_eq!(episode.first_attack.approach_cell, strategy.approach_cell);
    assert!(episode.basis.stop <= episode.basis.observation.parts().tick);
    assert!(episode.basis.work <= WORK_ALLOWANCE);
    assert!(
        episode
            .worker
            .work
            .iter()
            .all(|spent| *spent <= WORK_ALLOWANCE)
    );
    assert_eq!(episode.worker.dequeued, episode.worker.delivered);
    assert_eq!(
        (
            episode.worker.failed,
            episode.worker.cancelled,
            episode.worker.dropped
        ),
        (0, 0, 0)
    );
    assert_eq!(episode.basis.observation.parts().tick, Tick(183));
    assert_eq!(episode.basis.stop, Tick(183));
    assert_eq!(episode.basis.work, 21_691);
    let plant = episode
        .plant
        .expect("the selected attack must actually plant");
    assert_eq!(plant.range, 3);
    assert_eq!(plant.cell, CellId::new(7, 5));
    assert_eq!(
        (plant.placed_at, plant.detonates_at, plant.flame_end),
        (Tick(232), Tick(412), Tick(442))
    );
    assert_eq!(episode.rival_death, Some(plant.detonates_at));
    assert!(episode.resolved);
    assert!(episode.rival_flame_observed);
    assert!(episode.cpu_death.is_none());
    let a = episode.admissions;
    assert_eq!(
        [
            a.expired_or_wrong_decision,
            a.missed_manual_tick,
            a.stale_scene,
            a.invalid_hold,
            a.contact,
            a.strategic_plant,
            a.invalid_shape,
            a.pulse,
            a.route,
            a.duplicate_decision,
            a.refuge_handoff
        ],
        [0; 11]
    );
}

#[test]
fn hard57_already_prefers_west_when_active_pits_make_north_farther() {
    use super::exit_pressure_owner_support::run_with_environment;
    use super::queued_scenario_support::Environment;
    let episode = run_with_environment(CpuProfile::HARD_V57, 500, Environment::Active);
    let strategy = episode.basis.strategy.unwrap();
    assert_eq!(strategy.approach_cell, Some(CellId::new(5, 7)));
    assert_eq!(episode.basis.work, 55_822);
    let plant = episode.plant.expect("pit-aware attack must plant");
    assert_eq!(plant.cell, CellId::new(5, 7));
    assert_eq!(
        (plant.placed_at, plant.detonates_at, plant.flame_end),
        (Tick(232), Tick(412), Tick(442))
    );
    assert_eq!(episode.rival_death, Some(plant.detonates_at));
    assert!(episode.cpu_death.is_none());
    assert!(episode.resolved);
    assert!(episode.rival_flame_observed);
}
