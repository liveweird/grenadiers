use grenadiers_core::{
    cpu::{
        Intent, ManualIntent, MovementIntent, policy::StrategicTarget, survival::WORK_ALLOWANCE,
    },
    game::{CellId, Direction, Position, SpikePitPhase, Tick},
    room::{CpuProfile, MatchPhase},
};

use super::pit_entry_owner_support::{
    CAPTURE_PIT, Sample, TARGET, run, run_with_open_route, run_with_profile,
};

#[test]
fn archived_hard51_voluntarily_enters_the_quiet_capture_pit() {
    let outcome = run(240);
    let phase = |sample: &Sample| {
        sample
            .pits
            .iter()
            .find(|(cell, _, _)| *cell == CAPTURE_PIT)
            .copied()
            .unwrap()
    };
    let targeted = outcome
        .samples
        .iter()
        .find(|sample| {
            sample
                .intent
                .is_some_and(|intent| intent.target == StrategicTarget::Block { cell: TARGET })
        })
        .unwrap();
    let activated = outcome
        .samples
        .iter()
        .find(|sample| phase(sample).1 != SpikePitPhase::Idle)
        .unwrap();
    let raised = outcome
        .samples
        .iter()
        .find(|sample| phase(sample).1 == SpikePitPhase::Raised)
        .unwrap();
    let dead = outcome.samples.iter().find(|sample| !sample.alive).unwrap();
    eprintln!(
        "pit probe={:?} target={:?}@{:?} warning={:?}@{:?} raised/dead={:?}@{:?}",
        outcome.plan_probe,
        targeted.position,
        targeted.tick,
        activated.position,
        activated.tick,
        dead.position,
        dead.tick,
    );
    assert_eq!(outcome.samples[0].position, Position { x: 3_584, y: 8_515 });
    assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
    assert_eq!((outcome.worker.failed, outcome.worker.cancelled), (0, 0));
    assert!(
        outcome
            .worker
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    assert_eq!(outcome.admission, Default::default());
    assert_eq!(outcome.result_phase, MatchPhase::Running);
    assert!(
        outcome.samples[0]
            .controller
            .contains("stop_admission=Some")
    );
    assert_eq!(targeted.tick, Tick(199));
    assert_eq!(
        outcome.initial_pits,
        [
            CellId::new(3, 5),
            CellId::new(11, 5),
            CAPTURE_PIT,
            CellId::new(11, 7),
        ]
    );
    assert_eq!(
        (activated.tick, activated.position, phase(activated)),
        (
            Tick(206),
            Position { x: 3_584, y: 8_487 },
            (CAPTURE_PIT, SpikePitPhase::Warning, 36),
        )
    );
    assert_eq!(
        (raised.tick, raised.position, phase(raised), raised.alive),
        (
            Tick(242),
            Position { x: 3_584, y: 7_668 },
            (CAPTURE_PIT, SpikePitPhase::Raised, 180),
            false,
        )
    );
    assert_eq!((dead.tick, dead.position), (Tick(242), raised.position));
    let before_death = outcome
        .samples
        .iter()
        .find(|sample| sample.tick == Tick(241))
        .unwrap();
    assert!(before_death.alive);
    assert_eq!(before_death.position, raised.position);
    assert!(
        before_death
            .controller
            .contains("stop_admission=Some(Tick(240))")
    );
    assert!(
        outcome
            .samples
            .iter()
            .any(|sample| sample.controller.contains("pulse=true"))
    );
    let probe = outcome.plan_probe.as_ref().unwrap();
    assert_eq!(
        (
            probe.observation_tick,
            probe.settled_stop_tick,
            probe.position
        ),
        (Tick(198), Tick(198), Position { x: 3_776, y: 8_704 })
    );
    assert_eq!(probe.work, 57_850);
    assert_eq!(
        probe.intent,
        Intent {
            movement: MovementIntent::Direction(Direction::North),
            manual: None::<ManualIntent>,
        }
    );
    assert_eq!(probe.lease_basis_tick, Tick(198));
    assert_eq!(probe.lease_window, (Tick(199), Tick(206)));
    assert_eq!(probe.lease_hold_ticks, 12);
    assert_eq!(probe.lease_safe_until, Tick(227));
    assert_eq!(outcome.worker.work[..2], [65_532, 58_000]);
}

#[test]
fn pit_avoidance_profiles_farm_from_a_safe_approach_and_survive_their_bomb() {
    for profile in [
        CpuProfile::HARD_V53,
        CpuProfile::PIT_AVOIDANCE_V52_STANDARD,
        CpuProfile::PIT_AVOIDANCE_V52_EASY,
    ] {
        let outcome = run_with_open_route(profile, 3_000);
        eprintln!(
            "PIT_OWNER profile={profile:?} plant={:?} escaped={} destroyed={} won={} last={:?} work={:?}",
            outcome.plant,
            outcome.completed_escape,
            outcome.destroyed,
            outcome.opponent_defeated,
            outcome.samples.last(),
            outcome.worker.work.iter().max()
        );
        let plant = outcome
            .plant
            .expect("avoiding the pit must still make useful bombing progress");
        assert!(
            outcome.completed_escape,
            "CPU must survive through its first bomb's flame expiry"
        );
        assert!(outcome.destroyed || outcome.opponent_defeated);
        assert!(outcome.samples.iter().all(|sample| sample.alive));
        assert!(
            outcome
                .samples
                .iter()
                .filter(|sample| sample.tick < plant)
                .all(|sample| {
                    outcome.initial_pits.iter().all(|cell| {
                        !grenadiers_core::game::geometry::position_overlaps_cell(
                            sample.position,
                            *cell,
                        )
                    })
                }),
            "ordinary approach before planting must avoid even partial pit contact"
        );
        assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
        assert_eq!(
            (
                outcome.worker.failed,
                outcome.worker.cancelled,
                outcome.worker.dropped
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
        assert_eq!(outcome.admission, Default::default());
    }
}

#[test]
fn pit_avoidance_profiles_do_not_enter_the_capture_trap() {
    for profile in [
        CpuProfile::HARD_V53,
        CpuProfile::PIT_AVOIDANCE_V52_STANDARD,
        CpuProfile::PIT_AVOIDANCE_V52_EASY,
    ] {
        let outcome = run_with_profile(profile, 600);
        assert!(outcome.samples.iter().all(|sample| sample.alive));
        assert!(
            outcome
                .samples
                .iter()
                .all(|sample| outcome.initial_pits.iter().all(|cell| {
                    !grenadiers_core::game::geometry::position_overlaps_cell(sample.position, *cell)
                }))
        );
        assert_eq!(outcome.result_phase, MatchPhase::Running);
        assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
        assert_eq!(
            (
                outcome.worker.failed,
                outcome.worker.cancelled,
                outcome.worker.dropped
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
        assert_eq!(outcome.admission, Default::default());
    }
}
