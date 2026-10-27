use grenadiers_core::game::{RulesConfig, SeatId, Tick};

use super::evaluation_support::{EvaluationCase, HumanScript, run};
use crate::game::rooms::replay::ReplayAction;

#[test]
fn a2_c0_recovers_from_confirmed_plant_invalidation_and_survives_its_flames() {
    let evaluation = run(
        EvaluationCase::new(
            "a2-c0-retreat-recovery",
            "a2",
            [73; 32],
            "c0",
            [17; 32],
            HumanScript::BombRetreat,
            260,
        ),
        grenadiers_core::room::CpuProfile::CURRENT,
    )
    .unwrap();
    let trace = &evaluation.trace;
    let planted_at = trace
        .first_observed_cpu_bomb_placed_tick
        .expect("CPU plant must become visible");
    assert_eq!(planted_at, Tick(190));
    let flame_expiry = Tick(
        planted_at.0
            + trace.settings.bomb_fuse_ticks()
            + RulesConfig::expanded_v5(trace.settings).flame_ticks(),
    );
    assert_eq!(flame_expiry, Tick(400));

    let invalidation = trace
        .frames
        .iter()
        .find(|frame| frame.physical_tick == Tick(204))
        .expect("rival bomb invalidation snapshot");
    let recovery = trace
        .frames
        .iter()
        .find(|frame| frame.recovery_pulse)
        .expect("fresh checked recovery pulse");
    assert!(recovery.physical_tick > invalidation.physical_tick);
    assert_ne!(recovery.cpu_position, invalidation.cpu_position);
    assert!(trace.accepted_actions.iter().any(|(tick, seat, action)| {
        *tick > Tick(204)
            && *seat == SeatId::Two
            && matches!(action, ReplayAction::Movement(Some(_)))
    }));
    assert_eq!(
        trace
            .accepted_actions
            .iter()
            .filter(|(tick, seat, action)| {
                *tick <= flame_expiry
                    && *seat == SeatId::Two
                    && matches!(action, ReplayAction::Bomb(_))
            })
            .count(),
        1,
        "recovery must not authorize a second Bomb"
    );

    let after_flame = trace
        .frames
        .iter()
        .find(|frame| frame.physical_tick >= flame_expiry)
        .expect("run must reach full own-flame expiry");
    assert!(after_flame.cpu_alive);
    let first_origin = trace
        .frames
        .iter()
        .position(|frame| frame.recovery_origin)
        .expect("confirmed Plant invalidation must issue recovery provenance");
    let cleared = trace.frames[first_origin + 1..]
        .iter()
        .position(|frame| !frame.recovery_origin)
        .map(|offset| first_origin + 1 + offset)
        .expect("leaving the origin footprint must revoke recovery provenance");
    assert!(
        trace.frames[cleared..]
            .iter()
            .take_while(|frame| frame.physical_tick <= flame_expiry)
            .all(|frame| !frame.recovery_origin && !frame.recovery_pulse),
        "cleared provenance must not return during its certified hazard"
    );
}
