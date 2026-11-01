use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::{
    MemoryKey, MovementIntent, Observation, ObservedActor, ObservedPickup, WorkBudget,
    policy::UtilityMemory,
};
use crate::game::{CellId, Direction, PickupKind, Position, Tick};

use super::teleporter_step_geometry::{distance_to_cell, max_travel};
use super::teleporter_step_v50_fixture::{CPU, TARGET, captured_tick_1381};
use super::{
    RouteFailure, TeleporterStepPhase, TeleporterStepRequest, certify_teleporter_step,
    certify_teleporter_step_v50,
};

const ALLOWANCE: u32 = 65_536;
const ASSUMED_SETTLED_STOP: Tick = Tick(1_381);

#[test]
fn captured_distant_second_teleporter_certifies_first_approach() {
    let observation = captured_tick_1381();
    assert_eq!(observation.parts().recipient, CPU);
    assert_eq!(observation.parts().tick, Tick(1_381));
    assert_eq!(
        (
            observation.parts().rules.running_updates,
            observation.parts().rules.remaining_ticks,
        ),
        (1_185, 9_615)
    );
    assert_eq!(observation.parts().arena.blocks.len(), 40);
    assert_eq!(observation.parts().pickups.len(), 2);
    assert_eq!(observation.respawn_lairs().unwrap().mobs().len(), 2);
    assert_eq!(observation.spike_pits().unwrap().pits().len(), 4);
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let guidance = crate::cpu::policy::teleporter_guidance_v50(
        &observation,
        &UtilityMemory::new(key),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap();
    let candidate = guidance.candidates[0].unwrap();
    assert_eq!(guidance.target, TARGET);
    assert_eq!(candidate.waypoint, CellId::new(9, 10));
    assert_eq!(
        candidate.movement,
        MovementIntent::Direction(Direction::South)
    );
    let forecast = forecast_for(&observation);
    let request = TeleporterStepRequest {
        target: guidance.target,
        movement: candidate.movement,
        settled_stop_tick: ASSUMED_SETTLED_STOP,
    };
    let mut measured = WorkBudget::new(ALLOWANCE);
    let proof =
        certify_teleporter_step_v50(&observation, &forecast, request, &mut measured).unwrap();
    assert_eq!(measured.spent(), 12_903);
    assert_eq!(proof.phase(), TeleporterStepPhase::Approach);
    assert_eq!(proof.target(), TARGET);
    assert!(
        proof
            .nominal_makes_progress_toward(candidate.waypoint, &mut WorkBudget::new(ALLOWANCE))
            .unwrap()
    );
    assert_exact_boundary(&observation, &forecast, request, measured.spent());
}

#[test]
fn distant_second_teleporter_also_certifies_collect_and_exact_budgets() {
    let observation = collect_observation();
    let forecast = forecast_for(&observation);
    let request = request(Direction::South);
    let mut measured = WorkBudget::new(ALLOWANCE);
    let proof =
        certify_teleporter_step_v50(&observation, &forecast, request, &mut measured).unwrap();
    assert_eq!(measured.spent(), 15_156);
    assert_eq!(proof.phase(), TeleporterStepPhase::Collect);

    assert_exact_boundary(&observation, &forecast, request, measured.spent());
}

#[test]
fn legacy_entry_point_retains_exactly_one_teleporter_contract() {
    let observation = captured_tick_1381();
    let forecast = forecast_for(&observation);
    assert_eq!(
        certify_teleporter_step(
            &observation,
            &forecast,
            request(Direction::East),
            &mut WorkBudget::new(ALLOWANCE),
        )
        .unwrap_err(),
        RouteFailure::InvalidRequest
    );
}

#[test]
fn every_actor_origin_must_be_far_from_each_unselected_pickup() {
    let rival_reachable = captured_tick_1381()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            actor.position = Position::center(CellId::new(1, 6));
        })
        .unwrap();
    rejects(&rival_reachable, RouteFailure::Unsupported);

    let root_origin_adversary = captured_tick_1381()
        .map_parts(|parts| {
            parts.pickups[0] = ObservedPickup {
                cell: CellId::new(9, 8),
                kind: PickupKind::Teleporter,
            };
        })
        .unwrap();
    rejects(&root_origin_adversary, RouteFailure::Unsupported);
}

#[test]
fn unselected_pickup_full_window_distance_boundary_is_exclusive() {
    let travel = max_travel(18).unwrap();
    let at_boundary = with_other_pickup_distance(travel);
    let actor = match &at_boundary.parts().actors[0] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => unreachable!(),
    };
    assert_eq!(
        distance_to_cell(actor.position, super::teleporter_step_v50_fixture::OTHER),
        travel
    );
    assert_eq!(
        certify(&at_boundary, Direction::South).unwrap_err(),
        RouteFailure::Unsupported
    );

    let outside = with_other_pickup_distance(travel + 1);
    let proof = certify(&outside, Direction::South).unwrap();
    assert_eq!(proof.phase(), TeleporterStepPhase::Approach);
}

#[test]
fn collect_still_rejects_an_unsafe_possible_partner_landing() {
    let observation = collect_observation()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            actor.position = Position::center(CellId::new(5, 5));
        })
        .unwrap();
    assert_eq!(
        certify(&observation, Direction::South).unwrap_err(),
        RouteFailure::NoRoute
    );
}

fn collect_observation() -> Observation {
    captured_tick_1381()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.position = Position {
                x: 13_824,
                y: 7_140,
            };
            actor.heading = Direction::South;
            let ObservedActor::Visible(partner) = &mut parts.actors[3] else {
                unreachable!()
            };
            partner.position = Position::center(CellId::new(13, 11));
        })
        .unwrap()
}

fn with_other_pickup_distance(distance: i64) -> Observation {
    captured_tick_1381()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            // OTHER's expanded footprint ends at x=2_368; y=6_656 is 192
            // units above it, leaving the requested horizontal contribution.
            actor.position = Position {
                x: i32::try_from(2_368 + distance - 192).unwrap(),
                y: 6_656,
            };
        })
        .unwrap()
}

fn certify(
    observation: &Observation,
    direction: Direction,
) -> Result<super::TeleporterStep, RouteFailure> {
    certify_teleporter_step_v50(
        observation,
        &forecast_for(observation),
        request(direction),
        &mut WorkBudget::new(ALLOWANCE),
    )
}

fn rejects(observation: &Observation, expected: RouteFailure) {
    assert_eq!(certify(observation, Direction::East).unwrap_err(), expected);
}

fn request(direction: Direction) -> TeleporterStepRequest {
    TeleporterStepRequest {
        target: TARGET,
        movement: MovementIntent::Direction(direction),
        settled_stop_tick: ASSUMED_SETTLED_STOP,
    }
}

fn forecast_for(observation: &Observation) -> crate::cpu::forecast::HazardForecast {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let mut memory = ObservedHazardMemory::new(key);
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

fn assert_exact_boundary(
    observation: &Observation,
    forecast: &crate::cpu::forecast::HazardForecast,
    request: TeleporterStepRequest,
    exact: u32,
) {
    let mut exact_work = WorkBudget::new(exact);
    assert!(certify_teleporter_step_v50(observation, forecast, request, &mut exact_work).is_ok());
    assert_eq!(exact_work.spent(), exact);
    let mut short = WorkBudget::new(exact - 1);
    assert_eq!(
        certify_teleporter_step_v50(observation, forecast, request, &mut short).unwrap_err(),
        RouteFailure::WorkExhausted
    );
}
