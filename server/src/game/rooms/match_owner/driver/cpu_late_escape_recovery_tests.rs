use grenadiers_core::game::{SeatId, Tick};

use super::evaluation_support::{EvaluationCase, HumanScript, run};
use crate::game::rooms::replay::ReplayAction;

#[derive(Clone, Copy)]
struct RecoveryCase {
    id: &'static str,
    arena_id: &'static str,
    arena_seed: [u8; 32],
    placements: [Tick; 2],
    invalidated_at: Tick,
    flame_expiry: Tick,
}

#[test]
fn scene_change_at_second_plant_recovers_both_generated_cases() {
    for case in [
        RecoveryCase {
            id: "a2-c0-retreat-late-recovery",
            arena_id: "a2",
            arena_seed: [73; 32],
            placements: [Tick(190), Tick(412)],
            invalidated_at: Tick(414),
            flame_expiry: Tick(622),
        },
        RecoveryCase {
            id: "a1-c0-retreat-late-recovery",
            arena_id: "a1",
            arena_seed: [37; 32],
            placements: [Tick(214), Tick(424)],
            invalidated_at: Tick(426),
            flame_expiry: Tick(634),
        },
    ] {
        assert_recovery(case);
    }
}

fn assert_recovery(case: RecoveryCase) {
    let trace = run(EvaluationCase::new(
        case.id,
        case.arena_id,
        case.arena_seed,
        "c0",
        [17; 32],
        HumanScript::BombRetreat,
        480,
    ))
    .unwrap()
    .trace;
    assert!(
        trace
            .observed_cpu_bomb_placements
            .iter()
            .map(|placement| placement.tick)
            .collect::<Vec<_>>()
            .starts_with(&case.placements)
    );

    let invalidation = trace
        .frames
        .iter()
        .find(|frame| frame.physical_tick == case.invalidated_at)
        .expect("second placement publication must be observed");
    assert!(invalidation.cpu_alive);
    let origin_index = trace
        .frames
        .iter()
        .position(|frame| frame.physical_tick >= case.invalidated_at && frame.recovery_origin)
        .expect("invalidated Awaiting plant must issue recovery provenance");
    let recovery = trace.frames[origin_index..]
        .iter()
        .find(|frame| frame.recovery_pulse)
        .expect("fresh worker result must admit a checked recovery pulse");
    assert!(recovery.physical_tick >= case.invalidated_at);
    assert!(trace.accepted_actions.iter().any(|(tick, seat, action)| {
        *tick >= case.invalidated_at
            && *seat == SeatId::Two
            && matches!(action, ReplayAction::Movement(Some(_)))
    }));

    for (tick, seat, action) in &trace.accepted_actions {
        if *seat == SeatId::Two
            && matches!(action, ReplayAction::Bomb(_) | ReplayAction::Equipment(_))
        {
            assert!(
                !trace
                    .frames
                    .iter()
                    .any(|frame| frame.physical_tick == *tick && frame.recovery_origin),
                "recovery provenance must never authorize manual input"
            );
        }
    }
    let after_flame = trace
        .frames
        .iter()
        .find(|frame| frame.physical_tick >= case.flame_expiry)
        .unwrap_or_else(|| {
            panic!(
                "{} ended before {:?}: terminal={:?} last={:?}",
                case.id,
                case.flame_expiry,
                trace.terminal,
                trace.frames.last().map(|frame| (
                    frame.physical_tick,
                    frame.cpu_alive,
                    frame.human_alive
                ))
            )
        });
    assert!(after_flame.cpu_alive);
    let cleared = trace.frames[origin_index + 1..]
        .iter()
        .position(|frame| !frame.recovery_origin)
        .map(|offset| origin_index + 1 + offset)
        .expect("recovery provenance must clear after leaving the bomb footprint");
    assert!(
        trace.frames[cleared..]
            .iter()
            .take_while(|frame| frame.physical_tick <= case.flame_expiry)
            .all(|frame| !frame.recovery_origin && !frame.recovery_pulse),
        "cleared provenance must not reappear before the certified hazard expires"
    );
}
