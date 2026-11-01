use crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121;
use crate::cpu::{Observation, ObservationError, ObservedRespawnLairs, WorkBudget};
use crate::game::{Direction, Position};
use crate::room::{RespawnLairConfig, RespawnLairSettings};

use super::teleporter_step_tests::{
    ALLOWANCE, collect_observation, forecast_for, request, without_respawn_lairs,
};
use super::{RouteFailure, TeleporterStepPhase, certify_teleporter_step};

#[test]
fn honestly_disabled_rules10_lairs_support_approach_and_collect() {
    let approach = without_respawn_lairs(captured_tick_6121());
    let approach_forecast = forecast_for(&approach);
    let approach_proof = certify_teleporter_step(
        &approach,
        &approach_forecast,
        request(&approach, Direction::West),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap();
    assert_eq!(approach_proof.phase(), TeleporterStepPhase::Approach);
    assert!(approach_proof.basis_observation().respawn_lairs().is_none());

    let collect = without_respawn_lairs(collect_observation());
    let collect_forecast = forecast_for(&collect);
    let collect_proof = certify_teleporter_step(
        &collect,
        &collect_forecast,
        request(&collect, Direction::North),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap();
    assert_eq!(collect_proof.phase(), TeleporterStepPhase::Collect);
    let partner = match &collect.parts().actors[0] {
        crate::cpu::ObservedActor::Visible(actor) => actor.position,
        crate::cpu::ObservedActor::Hidden { .. } => unreachable!(),
    };
    assert!(collect_proof.allows_stopped_position(partner));
    assert!(!collect_proof.allows_stopped_position(Position {
        x: 14_336,
        y: 12_288
    }));
}

#[test]
fn disabled_contact_does_not_accept_an_older_rules_scene() {
    let disabled = without_respawn_lairs(captured_tick_6121());
    let environment = disabled.environment().unwrap().clone();
    let mut parts = disabled.into_parts();
    parts.rules.version = 9;
    let older = Observation::new_rules_v9(parts, environment, None).unwrap();
    let forecast = forecast_for(&older);
    assert_eq!(
        certify_teleporter_step(
            &older,
            &forecast,
            request(&older, Direction::West),
            &mut WorkBudget::new(ALLOWANCE),
        )
        .unwrap_err(),
        RouteFailure::Unsupported
    );
}

#[test]
fn inconsistent_explicit_disabled_or_shapeless_enabled_overlay_is_unconstructible() {
    let disabled = without_respawn_lairs(captured_tick_6121());
    let environment = disabled.environment().unwrap().clone();
    let pits = disabled.spike_pits().cloned();
    let parts = disabled.into_parts();
    for config in [
        RespawnLairConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
    ] {
        assert_eq!(
            Observation::new_rules_v10(
                parts.clone(),
                environment.clone(),
                Some(ObservedRespawnLairs::new(config, Vec::new(), Vec::new())),
                pits.clone(),
            ),
            Err(ObservationError::InvalidActors)
        );
    }
}
