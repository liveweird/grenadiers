use crate::cpu::{
    Observation, ObservedActor, ObservedEnvironment, ObservedPickup, UnknownActorFacts, WorkBudget,
};
use crate::game::geometry::MovementSegment;
use crate::game::{CellId, Direction, PickupKind, Position, Tick};

use super::teleporter_step::charge_directional_advance;
use super::teleporter_step_geometry::charge_segment_scans;
use super::teleporter_step_tests::{ALLOWANCE, collect_observation, forecast_for, request};
use super::{RouteFailure, certify_teleporter_step};

#[test]
fn contested_target_and_reachable_forced_pickups_reject() {
    let contested = collect_observation()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                unreachable!()
            };
            actor.position = Position { x: 4_500, y: 9_728 };
        })
        .unwrap();
    rejects(&contested, Direction::North, RouteFailure::NoRoute);

    for kind in [
        PickupKind::RemoteBomb,
        PickupKind::Landmine,
        PickupKind::Bracelet,
    ] {
        let reachable = collect_observation()
            .map_parts(|parts| {
                parts.pickups[0] = ObservedPickup {
                    cell: CellId::new(1, 2),
                    kind,
                };
            })
            .unwrap();
        rejects(&reachable, Direction::North, RouteFailure::Unsupported);
    }
}

#[test]
fn incomplete_hidden_living_roster_rejects() {
    let basis = collect_observation();
    let environment = basis.environment().unwrap();
    let mut slow = environment.actor_storm_slow_remaining_ticks().to_vec();
    slow[0] = None;
    let replacement_environment = ObservedEnvironment::new(
        environment.storm_cloud_config(),
        environment.storm_cloud(),
        slow,
    );
    let respawn = basis.respawn_lairs().cloned();
    let pits = basis.spike_pits().cloned();
    let mut parts = basis.into_parts();
    let actor = match &parts.actors[0] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => unreachable!(),
    };
    parts.actors[0] = ObservedActor::Hidden {
        id: actor.id,
        seat: actor.seat,
        alive: true,
    };
    let hidden = Observation::new_rules_v10(parts, replacement_environment, respawn, pits).unwrap();
    rejects(&hidden, Direction::North, RouteFailure::Unsupported);
}

#[test]
fn partner_landing_mob_pit_and_storm_traps_reject() {
    for trap in [
        Position::center(CellId::new(6, 5)),
        Position { x: 5_632, y: 5_632 },
        Position { x: 3_584, y: 4_384 },
    ] {
        let trapped = collect_observation()
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                    unreachable!()
                };
                actor.position = trap;
                actor.unknown = UnknownActorFacts::ALL;
            })
            .unwrap();
        rejects(&trapped, Direction::North, RouteFailure::NoRoute);
    }
}

#[test]
fn zero_budget_fails_before_footprint_or_kernel_scan() {
    let observation = collect_observation();
    let forecast = forecast_for(&observation);
    let mut helper_work = WorkBudget::new(0);
    assert_eq!(
        charge_segment_scans(
            &forecast,
            &[MovementSegment::new(
                Position {
                    x: 3_584,
                    y: 10_250
                },
                Position {
                    x: 3_584,
                    y: 10_199
                },
            )],
            &mut helper_work,
        ),
        Err(RouteFailure::WorkExhausted)
    );
    assert_eq!(helper_work.spent(), 0);

    let mut kernel_work = WorkBudget::new(0);
    assert_eq!(
        certify_teleporter_step(
            &observation,
            &forecast,
            request(&observation, Direction::North),
            &mut kernel_work,
        )
        .unwrap_err(),
        RouteFailure::WorkExhausted
    );
    assert_eq!(kernel_work.spent(), 0);
}

#[test]
fn directional_motion_bound_is_charged_atomically_before_advance() {
    let observation = collect_observation();
    // Four actors: 16 fixed units + 8 per roster member.
    let mut short = WorkBudget::new(47);
    assert_eq!(
        charge_directional_advance(&observation, &mut short),
        Err(RouteFailure::WorkExhausted)
    );
    assert_eq!(short.spent(), 0);

    let mut exact = WorkBudget::new(48);
    assert_eq!(charge_directional_advance(&observation, &mut exact), Ok(()));
    assert_eq!(exact.spent(), 48);
}

fn rejects(observation: &Observation, direction: Direction, expected: RouteFailure) {
    let forecast = forecast_for(observation);
    assert_eq!(
        certify_teleporter_step(
            observation,
            &forecast,
            request(observation, direction),
            &mut WorkBudget::new(ALLOWANCE),
        )
        .unwrap_err(),
        expected
    );
    assert_eq!(observation.parts().tick, Tick(6_121));
}
