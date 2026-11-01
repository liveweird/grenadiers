use grenadiers_core::{
    cpu::{
        ManualIntent, MovementIntent,
        policy::{StrategicObjectiveKind, StrategicPhase, StrategicTarget},
        survival::WORK_ALLOWANCE,
    },
    game::{CellId, Direction, Position, Tick},
};

use super::early_farm_owner_support::{plan_counts, run};

#[test]
fn current_hard51_original_top_right_spawn_admits_the_first_farm_plant() {
    let outcome = run(47, 360);
    let counts = plan_counts(&outcome);
    assert_eq!(outcome.initial_blocks, 48);
    assert_eq!(
        outcome.initial_position,
        Position {
            x: 13_824,
            y: 1_536
        }
    );
    assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
    assert_eq!(counts, (5, 0, 2));
    assert_eq!((outcome.worker.failed, outcome.worker.cancelled), (0, 0));
    assert!(
        outcome
            .worker
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    assert!(
        outcome.samples[0]
            .controller
            .contains("stop_admission=Some")
    );
    assert_eq!(outcome.cloud.position, Position { x: 7_680, y: 6_656 });
    assert_eq!(outcome.cloud.direction, Direction::West);
    assert_eq!(outcome.cloud.leg_remaining_ticks, 128);
    assert_eq!(outcome.lairs, [CellId::new(6, 6), CellId::new(8, 6)]);
    assert_eq!(
        outcome.pits,
        [
            CellId::new(5, 5),
            CellId::new(9, 5),
            CellId::new(5, 7),
            CellId::new(9, 7),
        ]
    );
    let first_intent = outcome
        .samples
        .iter()
        .find_map(|sample| sample.intent)
        .unwrap();
    assert_eq!(first_intent.objective, StrategicObjectiveKind::Farm);
    assert_eq!(first_intent.phase, StrategicPhase::Approach);
    assert_eq!(
        first_intent.target,
        StrategicTarget::Block {
            cell: CellId::new(11, 2)
        }
    );
    assert_eq!(first_intent.approach_cell, Some(CellId::new(11, 1)));
    assert_eq!(
        outcome.worker.planned_intents[0].movement,
        MovementIntent::Direction(Direction::West)
    );
    assert!(
        outcome
            .worker
            .planned_intents
            .iter()
            .any(|intent| { intent.manual == Some(ManualIntent::Bomb) })
    );
    assert_eq!(outcome.first_bomb, Some(Tick(232)));
    assert!(outcome.samples.iter().any(|sample| sample.bombs == 1));
    assert!(outcome.samples.iter().any(|sample| sample.blocks < 48));
    assert!(
        outcome
            .samples
            .iter()
            .any(|sample| sample.position != outcome.initial_position)
    );
    assert_eq!(outcome.admission, Default::default());
    assert!(outcome.samples.iter().any(|sample| {
        sample.tick == Tick(183) && sample.controller.contains("settled_stop=Some(Tick(183))")
    }));
}

#[test]
fn current_hard51_fixed_cpu_seed_sweep_all_admit_the_original_spawn_plant() {
    let outcomes = [0, 1, 2, 3, 7, 31, 47, 51].map(|seed| (seed, run(seed, 600)));
    for (seed, outcome) in &outcomes {
        eprintln!(
            "seed={seed} bomb={:?} pose={:?} jobs={}/{} work={:?} plans={:?} admission={:?}",
            outcome.first_bomb,
            outcome.samples.last().unwrap().position,
            outcome.worker.delivered,
            outcome.worker.dequeued,
            outcome.worker.work,
            plan_counts(outcome),
            outcome.admission,
        );
        assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
        assert_eq!(outcome.worker.dequeued, 6);
        assert_eq!((outcome.worker.failed, outcome.worker.cancelled), (0, 0));
        assert!(
            outcome
                .worker
                .work
                .iter()
                .all(|work| *work <= WORK_ALLOWANCE)
        );
        assert_eq!(
            outcome.worker.work,
            [41_742, 34_401, 57_938, 22_091, 18_223, 5_247]
        );
        assert_eq!(plan_counts(outcome), (6, 0, 3));
        assert_eq!(outcome.first_bomb, Some(Tick(232)));
        assert_eq!(
            outcome.samples.last().unwrap().position,
            Position {
                x: 13_824,
                y: 1_842
            }
        );
        assert_eq!(outcome.admission, Default::default());
    }
}
