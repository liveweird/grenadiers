use std::collections::BTreeSet;

use grenadiers_core::{
    cpu::{BombKnowledge, ObservedActor, ObservedBombTrigger, survival::WORK_ALLOWANCE},
    game::{CellId, OilBarrelState, Position, SeatId, Tick},
    room::MatchPhase,
};

use super::{
    range_three_escape_diagnostics::{ForecastResult, QueuedBasis, SelectorResult},
    range_three_escape_owner_support::{Outcome, run},
};

#[test]
fn seed47_characterizes_range_three_escape_with_and_without_visible_oil() {
    let with_oil = run(true, 1_000);
    let without_oil = run(false, 1_000);
    eprintln!(
        "RANGE3_SUMMARY with_oil trigger={:?} deaths={:?} flames={} clear={:?} final={:?} jobs={}/{} bases={} transitions={} no_oil trigger={:?} deaths={:?} flames={} clear={:?} final={:?} jobs={}/{} bases={} transitions={}",
        with_oil.trigger,
        with_oil.first_deaths,
        with_oil.max_visible_flames,
        with_oil.flames_clear_after_trigger,
        with_oil.final_tick,
        with_oil.worker.delivered,
        with_oil.worker.dequeued,
        with_oil.queued_bases.len(),
        with_oil.transitions.len(),
        without_oil.trigger,
        without_oil.first_deaths,
        without_oil.max_visible_flames,
        without_oil.flames_clear_after_trigger,
        without_oil.final_tick,
        without_oil.worker.delivered,
        without_oil.worker.dequeued,
        without_oil.queued_bases.len(),
        without_oil.transitions.len(),
    );
    eprintln!("RANGE3_BASES_WITH_OIL {:#?}", with_oil.queued_bases);
    eprintln!("RANGE3_BASES_WITHOUT_OIL {:#?}", without_oil.queued_bases);
    eprintln!("RANGE3_RECOVERY_WITH_OIL {:#?}", with_oil.recovery_bases);
    eprintln!(
        "RANGE3_RECOVERY_WITHOUT_OIL {:#?}",
        without_oil.recovery_bases
    );
    print_transition_evidence("with_oil", &with_oil);
    print_transition_evidence("without_oil", &without_oil);
    print_wave_diagnostics("with_oil", &with_oil);
    print_wave_diagnostics("without_oil", &without_oil);

    let initial_positions = [
        Position {
            x: 13_824,
            y: 1_536,
        },
        Position {
            x: 1_536,
            y: 11_776,
        },
        Position {
            x: 13_824,
            y: 11_776,
        },
    ];
    assert_eq!(with_oil.initial_blocks, 81);
    assert_eq!(without_oil.initial_blocks, 81);
    assert_eq!(with_oil.initial_positions, initial_positions);
    assert_eq!(without_oil.initial_positions, initial_positions);
    assert!(with_oil.with_oil);
    assert!(!without_oil.with_oil);

    for outcome in [&with_oil, &without_oil] {
        assert_eq!(outcome.worker.dequeued, 594);
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
        assert!(
            outcome
                .admissions
                .iter()
                .all(|trace| *trace == Default::default())
        );
        assert!(
            outcome
                .transitions
                .iter()
                .any(|transition| transition.flags.route)
        );
        assert!(
            outcome
                .transitions
                .iter()
                .any(|transition| transition.flags.origin && !transition.flags.route)
        );
        assert!(
            outcome
                .transitions
                .iter()
                .any(|transition| transition.flags.pulse)
        );
        assert!(
            outcome
                .transitions
                .iter()
                .any(|transition| transition.flags.continuation)
        );
        assert!(outcome.transitions.iter().all(|transition| {
            matches!(transition.seat, SeatId::Two | SeatId::Three | SeatId::Four)
                && transition.tick <= outcome.final_tick
                && transition.state.contains("observation=")
        }));
        assert!(matches!(outcome.final_phase, MatchPhase::Result { .. }));
    }

    let with = with_oil
        .trigger
        .as_ref()
        .expect("seed47 must reach the range-3 trigger");
    let without = without_oil
        .trigger
        .as_ref()
        .expect("the paired no-Oil scene must reach the same trigger");
    assert_eq!(with.oil, vec![CellId::new(10, 1), CellId::new(13, 8)]);
    assert!(without.oil.is_empty());
    assert_eq!(with.placed_at, without.placed_at);
    assert_eq!(with.detonates_at, without.detonates_at);
    assert_eq!(with.cells, without.cells);
    assert_eq!(with.placed_at, Tick(862));
    assert_eq!(with.detonates_at, Tick(1_042));
    assert_eq!(
        with.cells,
        [CellId::new(13, 3), CellId::new(3, 11), CellId::new(11, 11)]
    );
    assert_eq!(with_oil.queued_bases.len(), 3);
    assert_eq!(without_oil.queued_bases.len(), 3);
    assert_eq!(with_oil.recovery_bases.len(), 3);
    assert_eq!(without_oil.recovery_bases.len(), 3);
    assert!(with_oil.max_visible_flames > 0);
    assert!(without_oil.max_visible_flames > 0);
    assert_eq!(with_oil.first_deaths, [Some(with.detonates_at); 3]);
    assert_eq!(without_oil.first_deaths, [Some(without.detonates_at); 3]);
    assert_eq!(with_oil.final_tick, with.detonates_at);
    assert_eq!(without_oil.final_tick, without.detonates_at);
    assert_eq!(with_oil.flames_clear_after_trigger, None);
    assert_eq!(without_oil.flames_clear_after_trigger, None);

    for capture in with_oil
        .queued_bases
        .iter()
        .chain(&without_oil.queued_bases)
    {
        assert_eq!(capture.observation.parts().recipient, capture.seat);
        assert_eq!(capture.hazards.key(), capture.memory.key());
        assert_eq!(capture.observation.parts().tick, Tick(867));
        assert_eq!(capture.settled_stop, Tick(867));
        assert!(matches!(
            capture.seat,
            SeatId::Two | SeatId::Three | SeatId::Four
        ));
        let _actual_origin = capture.origin();
        assert_eq!(capture.selector.result, SelectorResult::None);
        assert_eq!(capture.selector.work, 32_768);
        assert!(capture.forecast.work > 32_768);
        assert_eq!(
            capture.forecast.result,
            ForecastResult::Ok {
                basis_tick: Tick(867),
                horizon_end: Tick(1_707),
            }
        );
    }
    for capture in with_oil
        .recovery_bases
        .iter()
        .chain(&without_oil.recovery_bases)
    {
        assert_eq!(capture.observation.parts().tick, Tick(972));
        assert_eq!(capture.settled_stop, Tick(972));
        let _actual_origin = capture.origin();
        assert_eq!(capture.selector.result, SelectorResult::Pulse);
        assert!(capture.selector.work > 32_768);
        assert!(capture.selector.work <= 65_536);
        assert!(capture.forecast.work < 32_768);
        assert_eq!(
            capture.forecast.result,
            ForecastResult::Ok {
                basis_tick: Tick(972),
                horizon_end: Tick(1_812),
            }
        );
    }
    for capture in &with_oil.queued_bases {
        assert_exact_basis(capture, true);
    }
    for capture in &without_oil.queued_bases {
        assert_exact_basis(capture, false);
    }
    assert!(
        with_oil
            .queued_bases
            .iter()
            .all(|capture| capture.forecast.work == 41_524)
    );
    assert!(
        without_oil
            .queued_bases
            .iter()
            .all(|capture| capture.forecast.work == 41_124)
    );
}

fn assert_exact_basis(capture: &QueuedBasis, with_oil: bool) {
    let parts = capture.observation.parts();
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        panic!("range-3 basis must expose complete bomb facts");
    };
    assert_eq!(bombs.len(), 3);
    assert_eq!(
        bombs.iter().map(|bomb| bomb.cell).collect::<BTreeSet<_>>(),
        BTreeSet::from([CellId::new(13, 3), CellId::new(3, 11), CellId::new(11, 11),])
    );
    assert!(bombs.iter().all(|bomb| {
        bomb.placed_at == Tick(862)
            && bomb.range == 3
            && matches!(
                bomb.trigger,
                ObservedBombTrigger::Timed {
                    remaining_ticks,
                    paused: false,
                } if parts.tick.0 + u64::from(remaining_ticks) == 1_042
            )
    }));
    if with_oil {
        assert_eq!(
            parts
                .oil
                .iter()
                .map(|oil| (oil.cell, oil.state))
                .collect::<Vec<_>>(),
            vec![
                (CellId::new(10, 1), OilBarrelState::Unlit),
                (CellId::new(13, 8), OilBarrelState::Unlit),
            ]
        );
    } else {
        assert!(parts.oil.is_empty());
    }
}

fn print_transition_evidence(label: &str, outcome: &Outcome) {
    for (stage, transition) in [
        (
            "route",
            outcome
                .transitions
                .iter()
                .find(|transition| transition.flags.route),
        ),
        (
            "withdrawal_origin",
            outcome
                .transitions
                .iter()
                .find(|transition| transition.flags.origin && !transition.flags.route),
        ),
        (
            "pulse",
            outcome
                .transitions
                .iter()
                .find(|transition| transition.flags.pulse),
        ),
        (
            "continuation",
            outcome
                .transitions
                .iter()
                .find(|transition| transition.flags.continuation),
        ),
    ] {
        let transition = transition.expect("the production controller must expose each stage");
        eprintln!(
            "RANGE3_TRANSITION {label} {stage} seat={:?} tick={:?} flags={:?} state={}",
            transition.seat, transition.tick, transition.flags, transition.state
        );
    }
}

fn print_wave_diagnostics(label: &str, outcome: &Outcome) {
    let placed_at = outcome.trigger.as_ref().unwrap().placed_at;
    for seat in [SeatId::Two, SeatId::Three, SeatId::Four] {
        let basis = outcome
            .queued_bases
            .iter()
            .find(|basis| basis.seat == seat)
            .unwrap();
        let ObservedActor::Visible(actor) = &basis.observation.parts().actors[seat.index()] else {
            panic!("queued CPU actor must remain visible");
        };
        let seat_transitions = outcome
            .transitions
            .iter()
            .filter(|transition| transition.seat == seat)
            .collect::<Vec<_>>();
        let route_withdrawal = seat_transitions.windows(2).find_map(|pair| {
            (pair[0].flags.route && !pair[1].flags.route && pair[1].tick >= placed_at)
                .then_some(pair[1])
        });
        let first_origin = seat_transitions
            .iter()
            .find(|transition| transition.tick >= placed_at && transition.flags.origin);
        let first_recovery = seat_transitions.iter().find(|transition| {
            transition.tick >= placed_at
                && (transition.flags.pulse || transition.flags.continuation)
        });
        eprintln!(
            "RANGE3_WAVE {label} seat={seat:?} placed={placed_at:?} basis_tick={:?} stop={:?} pose={:?} selector={:?} forecast={:?} first_origin={:?} route_withdrawal={:?} first_recovery={:?} delivered={}/{}",
            basis.observation.parts().tick,
            basis.settled_stop,
            actor.position,
            basis.selector,
            basis.forecast,
            first_origin.map(|transition| (transition.tick, transition.flags)),
            route_withdrawal.map(|transition| (transition.tick, transition.flags)),
            first_recovery.map(|transition| (transition.tick, transition.flags)),
            outcome.worker.delivered,
            outcome.worker.dequeued,
        );
        let _actual_origin = basis.origin();
        assert_eq!(first_origin.unwrap().tick, Tick(864));
        assert_eq!(route_withdrawal.unwrap().tick, Tick(864));
        let applied = first_recovery.unwrap();
        assert_eq!(applied.tick, Tick(973));
        assert!(applied.flags.pulse && !applied.flags.continuation);
    }
}
