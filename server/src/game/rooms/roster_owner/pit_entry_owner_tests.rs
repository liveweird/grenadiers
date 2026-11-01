use grenadiers_core::{
    cpu::{
        Intent, ManualIntent, MovementIntent, policy::StrategicTarget, survival::WORK_ALLOWANCE,
    },
    game::{CellId, Direction, Position, SpikePitPhase, Tick},
    room::MatchPhase,
};

use super::pit_entry_owner_support::{CAPTURE_PIT, Sample, TARGET, run};

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
    let probe = &outcome.plan_probe;
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
