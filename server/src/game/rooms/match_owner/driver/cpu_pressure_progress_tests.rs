use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use grenadiers_core::game::{SeatId, Tick};

use super::evaluation_support::{EvaluationCase, HumanScript, run};
use crate::game::rooms::replay::ReplayAction;

const PREVIOUS_STALL: Tick = Tick(1600);

#[test]
fn repeated_bomb_pressure_keeps_queued_cpu_progressing() {
    let case = || {
        EvaluationCase::new(
            "a0-c0-pressure-progress",
            "a0",
            [11; 32],
            "c0",
            [17; 32],
            HumanScript::BombRetreat,
            3_600,
        )
    };
    let first = run(case(), grenadiers_core::room::CpuProfile::CURRENT)
        .unwrap()
        .trace;
    let repeat = run(case(), grenadiers_core::room::CpuProfile::CURRENT)
        .unwrap()
        .trace;

    assert_eq!(first, repeat, "fixed seeds must replay an exact CPU trace");
    assert!(first.replay_verified);
    assert_eq!(first.running_ticks, 3_600);
    assert!(first.censored);
    assert!(first.terminal.is_none());
    assert!(first.cpu_alive);

    let human_bombs_after_stall = first
        .accepted_actions
        .iter()
        .filter(|(tick, seat, action)| {
            *tick > PREVIOUS_STALL
                && *seat == SeatId::One
                && matches!(action, ReplayAction::Bomb(_))
        })
        .count();
    let cpu_moves_after_stall = first
        .accepted_actions
        .iter()
        .filter(|(tick, seat, action)| {
            *tick > PREVIOUS_STALL
                && *seat == SeatId::Two
                && matches!(action, ReplayAction::Movement(Some(_)))
        })
        .count();
    eprintln!(
        "PRESSURE_PROGRESS running={} humanBombsAfter1600={} cpuMovesAfter1600={} movements={} manuals={} visited={} attempts={} delivered={} workSum={} workMax={}",
        first.running_ticks,
        human_bombs_after_stall,
        cpu_moves_after_stall,
        first.accepted_movement_actions,
        first.accepted_manual_actions,
        first.visited_cells.len(),
        first.planning.attempts,
        first.planning.delivered,
        first.work_sum,
        first.work_max
    );
    assert!(
        human_bombs_after_stall >= 3,
        "fixture must apply repeated bomb pressure"
    );
    assert!(
        cpu_moves_after_stall > 0,
        "fresh queued plans must resume movement"
    );
    assert!(
        first.visited_cells.len() > 5,
        "CPU must exceed the release-92 five-cell stall"
    );
    assert!(first.planning.attempts > 0);
    assert!(first.planning.delivered > 0);
    assert!(first.work_sum > 0);
    assert!(
        first
            .planning
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
}
