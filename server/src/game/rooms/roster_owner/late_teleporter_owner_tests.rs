use grenadiers_core::{
    cpu::{policy::StrategicTarget, survival::WORK_ALLOWANCE},
    game::{PickupKind, Position, Tick},
    room::CpuProfile,
};

use super::late_teleporter_owner_support::{Outcome, Sample, run};

#[test]
fn archived_hard47_records_the_first_dead_roster_admission_refusal() {
    let outcome = run(CpuProfile::HARD_V47, 240);
    assert_scene_and_initial_stop(&outcome);
    assert_eq!(
        outcome.teleporter_trace.first(),
        Some(&(2, Tick(187), Tick(187), true))
    );
    assert!(outcome.teleporter_trace.iter().all(|event| event.0 == 2));
    assert!(
        outcome
            .samples
            .iter()
            .all(|sample| sample.position == outcome.initial_position)
    );
    assert!(outcome.teleporter_remaining);
    assert_eq!(outcome.collection_tick, None);
    assert_eq!(outcome.plant_tick, None);
    assert!(outcome.samples.iter().all(|sample| sample.intent.is_none()));
    assert_worker(&outcome);
}

#[test]
fn current_hard49_collects_then_plants_destroys_and_survives() {
    let outcome = run(CpuProfile::HARD_V49, 1_200);
    assert_scene_and_initial_stop(&outcome);
    assert_worker(&outcome);

    let movements = trace_ticks(&outcome.teleporter_trace, 0);
    let stops = trace_ticks(&outcome.teleporter_trace, 1);
    assert!(outcome.teleporter_trace.iter().all(|event| event.0 != 2));
    assert!(!movements.is_empty());
    assert_eq!(movements.len(), stops.len());
    for (index, (movement, stop)) in movements.iter().zip(&stops).enumerate() {
        assert_eq!(stop.0, movement.0 + 1);
        let next = movements.get(index + 1).copied();
        assert_physical_stop(&outcome.samples, *stop, next);
    }
    assert!(
        outcome
            .samples
            .iter()
            .any(|sample| sample.position != outcome.initial_position)
    );
    assert!(!outcome.teleporter_remaining);
    assert_eq!(outcome.collection_tick, Some(Tick(626)));
    assert_eq!(outcome.plant_tick, Some(Tick(635)));
    assert!(outcome.destroyed_block);
    assert!(outcome.cpu_alive);
    assert!(outcome.samples.iter().any(|sample| {
        sample.intent.is_some_and(|intent| {
            intent.target
                == StrategicTarget::Pickup {
                    cell: super::late_teleporter_owner_support::TELEPORTER,
                    kind: PickupKind::Teleporter,
                }
        })
    }));
    assert!(outcome.samples.last().unwrap().intent.is_some());
}

fn assert_scene_and_initial_stop(outcome: &Outcome) {
    assert_eq!(outcome.initial_position, Position { x: 1_536, y: 2_724 });
    assert_eq!(outcome.initial_block_count, 48);
    assert!(outcome.dead_three);
    assert_eq!(outcome.initial_storm_slow[2], Some(0));
    assert!(outcome.has_lairs && outcome.has_pits);
    assert!(outcome.structural_environment_geometry);
    assert_eq!(outcome.samples.first().unwrap().tick, Tick(182));
    assert!(
        outcome.samples[0]
            .controller
            .contains("stop_admission=Some(Tick(181))")
    );
    assert!(
        outcome
            .samples
            .iter()
            .any(|sample| sample.controller.contains("pending=Some"))
    );
    assert!(
        outcome
            .samples
            .iter()
            .any(|sample| sample.controller.contains("completed=true"))
    );
    assert!(
        outcome
            .samples
            .iter()
            .any(|sample| sample.controller.contains("settled_stop=Some"))
    );
}

fn assert_worker(outcome: &Outcome) {
    assert!(outcome.worker.dequeued > 0);
    assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
    assert_eq!((outcome.worker.failed, outcome.worker.cancelled), (0, 0));
    assert_eq!(outcome.worker.dropped, 0);
    assert_eq!(outcome.worker.work.len(), outcome.worker.dequeued);
    assert!(
        outcome
            .worker
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    assert_eq!(outcome.admission, Default::default());
}

fn assert_physical_stop(samples: &[Sample], stop: Tick, next_movement: Option<Tick>) {
    let moved_pose = samples
        .iter()
        .find(|sample| sample.tick == stop)
        .unwrap()
        .position;
    let confirmed = samples
        .iter()
        .find(|sample| {
            sample.tick > stop
                && next_movement.is_none_or(|next| sample.tick <= next)
                && sample.controller.contains("settled_stop=Some")
        })
        .unwrap_or_else(|| panic!("Stop {stop:?} before {next_movement:?} lacked confirmation"));
    let physical_stop = samples
        .iter()
        .find(|sample| sample.tick.0 == stop.0 + 1)
        .expect("the physical update after Stop admission must be retained");
    assert_eq!(physical_stop.position, moved_pose);
    assert_eq!(confirmed.position, moved_pose);
    assert!(
        samples
            .iter()
            .filter(|sample| { sample.tick >= physical_stop.tick && sample.tick <= confirmed.tick })
            .all(|sample| sample.position == moved_pose)
    );
}

fn trace_ticks(trace: &[(u8, Tick, Tick, bool)], kind: u8) -> Vec<Tick> {
    trace
        .iter()
        .filter_map(|(event, runtime, sealed, accepted)| {
            assert_eq!(runtime, sealed);
            assert!(*accepted);
            (*event == kind).then_some(*runtime)
        })
        .collect()
}
