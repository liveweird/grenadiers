use grenadiers_core::{
    cpu::{
        BombKnowledge, ObservedBombTrigger, forecast::ObservedHazardMemory,
        survival::WORK_ALLOWANCE,
    },
    game::{CellId, Direction, Position, Tick},
    room::{CpuPolicy, CpuProfile},
};

use super::early_farm_owner_interaction_support::{run, run_for_profile};

#[test]
fn captured_foreign_bomb_history_is_exactly_reconstructible_without_a_private_constructor() {
    let episode = run(600);
    let capture = &episode.basis_probe.capture;
    let mut previous = capture.observation.clone();
    previous = previous
        .map_parts(|parts| {
            parts.tick.0 -= 3;
            parts.rules.running_updates -= 3;
            parts.rules.remaining_ticks += 3;
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                panic!("captured bombs must be visible");
            };
            for bomb in bombs {
                let ObservedBombTrigger::Timed {
                    remaining_ticks,
                    paused: false,
                } = &mut bomb.trigger
                else {
                    panic!("captured bombs must have ordinary running fuses");
                };
                *remaining_ticks += 3;
            }
        })
        .unwrap();

    // This is a history-only carrier of the captured key, tick, IDs and cells,
    // not a restoration of the complete tick-231 publication. The memory reads
    // those facts only; planning always receives the unchanged actual tick 234.
    let mut rebuilt = ObservedHazardMemory::new(capture.hazard_memory.key());
    rebuilt.observe(&previous).unwrap();
    rebuilt.observe(&capture.observation).unwrap();
    assert_eq!(rebuilt, capture.hazard_memory);
}

#[test]
fn current_hard55_records_foreign_bomb_farming_on_an_actual_queued_basis() {
    let episode = run_for_profile(CpuProfile::HARD_V55, 600);
    eprintln!(
        "CURRENT_HARD55_FOREIGN_FARM plants={:?} jobs={}/{} admissions={:?} basis={:?}",
        episode.first_plants,
        episode.worker.delivered,
        episode.worker.dequeued,
        episode.admissions,
        episode.basis_probe,
    );
    eprintln!(
        "CURRENT_HARD55_POST_FOREIGN {:#?}",
        episode.post_foreign_probe
    );
    eprintln!(
        "CURRENT_HARD55_CLEAR_AFTER_FOREIGN {:#?}",
        episode.clear_after_foreign_probe
    );
    eprintln!(
        "CURRENT_HARD55_APPROACH_AFTER_FOREIGN {:#?}",
        episode.farm_approach_probe
    );
    for job in &episode.post_jobs {
        eprintln!("CURRENT_HARD55_POST_JOB {job:?}");
        assert!(job.tick >= Tick(415) && job.tick < Tick(780) && job.stop <= job.tick);
        assert!(job.position.cell().is_some());
        assert!([0, 2].contains(&job.bombs) && [0, 12].contains(&job.flames));
        assert!(job.strategy.is_some() && !job.had_origin);
        assert!(job.work <= WORK_ALLOWANCE && job.intent.is_some());
    }
    assert_eq!(episode.post_jobs.len(), 39);
    assert_eq!(episode.post_jobs.last().unwrap().tick, Tick(774));
    assert_eq!(
        episode
            .post_jobs
            .iter()
            .filter(|job| job.bombs == 0 && job.flames == 0)
            .map(|job| job.tick)
            .collect::<Vec<_>>(),
        [Tick(417), Tick(432), Tick(651)],
    );
    let clear = episode.clear_after_foreign_probe.as_ref().unwrap();
    assert_eq!(
        (clear.tick, clear.settled_stop_tick),
        (Tick(417), Tick(417))
    );
    assert_eq!(
        clear.position,
        Position {
            x: 10_752,
            y: 1_497
        }
    );
    assert_eq!(
        (clear.visible_bombs, clear.visible_flames, clear.blocks),
        (0, 0, 46)
    );
    assert_eq!(
        (clear.planner_work, clear.dispatched_work),
        (41_697, 41_809)
    );
    assert_eq!(
        clear.planner_intent,
        grenadiers_core::cpu::Intent::moving(Direction::East)
    );
    assert!(!clear.planner_execution);
    let approach = episode.farm_approach_probe.as_ref().unwrap();
    assert_eq!(
        (approach.tick, approach.settled_stop_tick),
        (Tick(432), Tick(432))
    );
    assert_eq!(
        approach.position,
        Position {
            x: 11_327,
            y: 1_536
        }
    );
    assert_eq!(
        (
            approach.visible_bombs,
            approach.visible_flames,
            approach.blocks
        ),
        (0, 0, 46)
    );
    assert_eq!(
        (approach.planner_work, approach.dispatched_work),
        (58_041, 58_153)
    );
    assert_eq!(
        approach.planner_intent,
        grenadiers_core::cpu::Intent::moving(Direction::East)
    );
    assert!(!approach.planner_execution);
    assert_eq!(
        episode.first_plants,
        [None, Some(Tick(205)), Some(Tick(205))]
    );
    assert_eq!(episode.worker.dequeued, 263);
    assert_eq!(episode.basis_probe.policy, CpuPolicy::UtilityV55);
    assert_eq!(episode.basis_probe.tick, Tick(234));
    assert_eq!(episode.basis_probe.settled_stop_tick, Tick(234));
    assert_eq!(
        episode.basis_probe.position,
        Position {
            x: 11_727,
            y: 1_536
        }
    );
    assert_eq!(episode.basis_probe.visible_bombs, 2);
    assert_eq!(episode.basis_probe.blocks, 48);
    assert_eq!(
        episode
            .basis_probe
            .capture
            .memory
            .strategic_intent()
            .unwrap()
            .approach_cell,
        Some(CellId::new(11, 1)),
    );
    assert!(episode.basis_probe.planner_intent.manual.is_none());
    assert_eq!(episode.basis_probe.planner_work, 10_627);
    assert_eq!(episode.basis_probe.dispatched_work, 10_743);
    assert_eq!(
        episode.basis_probe.planner_intent,
        grenadiers_core::cpu::Intent::moving(Direction::North)
    );
    assert!(!episode.basis_probe.planner_execution);
    assert!(episode.basis_probe.teleporter_enabled);
    assert!(episode.basis_probe.visible_bombs > 0);
    assert!(episode.basis_probe.blocks > 0);
    assert!(!episode.basis_probe.exposed_teleporter);
    assert!(!episode.basis_probe.had_origin);
    assert_eq!(episode.worker.dequeued, episode.worker.delivered);
    assert_eq!((episode.worker.failed, episode.worker.cancelled), (0, 0));
    assert!(
        episode
            .worker
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    assert!(
        episode
            .admissions
            .iter()
            .all(|admission| *admission == Default::default())
    );
}
