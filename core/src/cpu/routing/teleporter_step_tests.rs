use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121;
use crate::cpu::{MemoryKey, MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{CellId, Direction, Equipment, Position, SpecialWeapon, Tick};

use super::{RouteFailure, TeleporterStepPhase, TeleporterStepRequest, certify_teleporter_step};

pub(super) const TARGET: CellId = CellId::new(3, 9);
pub(super) const ALLOWANCE: u32 = 65_536;

pub(super) fn forecast_for(observation: &Observation) -> crate::cpu::forecast::HazardForecast {
    let mut memory = ObservedHazardMemory::new(MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    });
    memory.observe(observation).unwrap();
    forecast(
        observation,
        &memory,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap()
}

pub(super) fn request(observation: &Observation, direction: Direction) -> TeleporterStepRequest {
    TeleporterStepRequest {
        target: TARGET,
        movement: MovementIntent::Direction(direction),
        settled_stop_tick: observation.parts().tick,
    }
}

#[test]
fn captured_scene_with_assumed_stop_certifies_inactive_approach() {
    // Sample 99 contains the public pose but no controller-private settled
    // clock. This explicitly assumes settlement at the captured basis.
    let observation = captured_tick_6121();
    let forecast = forecast_for(&observation);
    let mut work = WorkBudget::new(ALLOWANCE);
    let proof = certify_teleporter_step(
        &observation,
        &forecast,
        request(&observation, Direction::West),
        &mut work,
    )
    .unwrap();
    assert_eq!(proof.phase(), TeleporterStepPhase::Approach);
    assert_eq!(proof.start_window().start, Tick(6_122));
    assert_eq!(proof.start_window().end, Tick(6_129));
    assert_eq!(proof.safe_until(), Tick(6_139));
    assert_eq!(proof.target(), TARGET);
    assert_eq!(proof.basis_observation(), &observation);
    assert!(work.spent() < ALLOWANCE);
}

#[test]
fn one_unit_short_of_measured_proof_budget_fails_closed() {
    let observation = captured_tick_6121();
    let forecast = forecast_for(&observation);
    let mut measured = WorkBudget::new(ALLOWANCE);
    certify_teleporter_step(
        &observation,
        &forecast,
        request(&observation, Direction::West),
        &mut measured,
    )
    .unwrap();
    let exact = measured.spent();
    let mut exact_work = WorkBudget::new(exact);
    assert!(
        certify_teleporter_step(
            &observation,
            &forecast,
            request(&observation, Direction::West),
            &mut exact_work,
        )
        .is_ok()
    );
    let mut short = WorkBudget::new(exact - 1);
    assert_eq!(
        certify_teleporter_step(
            &observation,
            &forecast,
            request(&observation, Direction::West),
            &mut short,
        )
        .unwrap_err(),
        RouteFailure::WorkExhausted
    );
}

#[test]
fn all_nominal_lanes_entering_target_produces_collect_phase() {
    // Counterfactual relocation only; this is not captured sample data.
    let observation = collect_observation();
    let forecast = forecast_for(&observation);
    let proof = certify_teleporter_step(
        &observation,
        &forecast,
        request(&observation, Direction::North),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap();
    assert_eq!(proof.phase(), TeleporterStepPhase::Collect);
}

pub(super) fn collect_observation() -> Observation {
    captured_tick_6121()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = Position {
                x: 3_584,
                y: 10_250,
            };
            actor.heading = Direction::North;
        })
        .unwrap()
}

#[test]
fn forced_rival_state_and_immediate_weapon_fail_closed() {
    let basis = captured_tick_6121();
    let frozen = basis
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            actor.voluntary_actions_frozen = true;
        })
        .unwrap();
    let frozen_forecast = forecast_for(&frozen);
    assert_eq!(
        certify_teleporter_step(
            &frozen,
            &frozen_forecast,
            request(&frozen, Direction::West),
            &mut WorkBudget::new(ALLOWANCE),
        )
        .unwrap_err(),
        RouteFailure::Unsupported
    );

    let armed = basis
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                unreachable!()
            };
            actor.weapon = Some(SpecialWeapon::Remote);
        })
        .unwrap();
    let armed_forecast = forecast_for(&armed);
    assert_eq!(
        certify_teleporter_step(
            &armed,
            &armed_forecast,
            request(&armed, Direction::West),
            &mut WorkBudget::new(ALLOWANCE),
        )
        .unwrap_err(),
        RouteFailure::Unsupported
    );

    for actor_index in [1, 2] {
        let bracelet = captured_tick_6121()
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[actor_index] else {
                    unreachable!()
                };
                actor.equipment = Some(Equipment::Bracelet);
            })
            .unwrap();
        let bracelet_forecast = forecast_for(&bracelet);
        assert_eq!(
            certify_teleporter_step(
                &bracelet,
                &bracelet_forecast,
                request(&bracelet, Direction::West),
                &mut WorkBudget::new(ALLOWANCE),
            )
            .unwrap_err(),
            RouteFailure::Unsupported,
            "actor index {actor_index}"
        );
    }
}
