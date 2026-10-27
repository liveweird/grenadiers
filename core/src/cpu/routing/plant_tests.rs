use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedEffect, UnknownBombFacts, WorkBudget,
};
use crate::game::geometry::position_overlaps_cell;
use crate::game::types::UNITS_PER_CELL;
use crate::game::{
    Direction, EffectKind, EntityId, InputAction, RulesConfig, SeatId, SeatInput, Tick,
};
use crate::room::RoomSettings;

use super::plant::derive_bomb_modifiers;
use super::tests::{actual_state, key, observation};
use super::{
    PlantFailure, PlantPresence, PlantRequest, RouteLimits, RouteStep, plan_ordinary_plant,
};

pub(super) fn request(first_movement: MovementIntent) -> PlantRequest {
    PlantRequest {
        execution_tick: Tick(7),
        settled_stop_tick: Tick(0),
        first_movement,
        limits: RouteLimits::default(),
    }
}

fn plan(first_movement: MovementIntent) -> (Observation, super::PlantCertificate) {
    let observed = observation(Vec::new(), Vec::new());
    let mut work = WorkBudget::new(65_536);
    let certificate = plan_ordinary_plant(
        &observed,
        &ObservedHazardMemory::new(key()),
        request(first_movement),
        &mut work,
    )
    .unwrap_or_else(|error| panic!("plant failed after {} work: {error:?}", work.spent()));
    (observed, certificate)
}

pub(super) fn current_scene(
    basis: &Observation,
    certificate: &super::PlantCertificate,
    tick: Tick,
    present: Option<(EntityId, u16)>,
) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - basis.parts().tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let lane = certificate.route().lane(Tick(7)).unwrap();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    if let Some((position, heading)) = lane.pose_at(tick) {
        actor.position = position;
        actor.heading = heading;
    }
    actor.live_bomb_count = u16::from(present.is_some());
    actor.weapon = certificate.expected_weapon(if let Some((id, _)) = present {
        PlantPresence::Present(id)
    } else if tick < certificate.placement().placed_at {
        PlantPresence::Awaiting
    } else {
        PlantPresence::Absent
    });
    parts.bombs = BombKnowledge::Visible(
        present
            .map(|(id, range)| {
                let placement = certificate.placement();
                ObservedBomb {
                    id,
                    placed_at: placement.placed_at,
                    cell: placement.cell,
                    owner: placement.owner,
                    range,
                    trigger: ObservedBombTrigger::Timed {
                        remaining_ticks: u16::try_from(placement.detonates_at.0 - tick.0).unwrap(),
                        paused: false,
                    },
                    blast: placement.blast,
                    unknown: UnknownBombFacts::ALL,
                }
            })
            .into_iter()
            .collect(),
    );
    Observation::new(parts).unwrap()
}

#[test]
fn certified_plant_replays_actual_movement_placement_and_consumed_origin_grace() {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position.x = 3 * UNITS_PER_CELL - 16;
    let observed = Observation::new(parts).unwrap();
    let certificate = plan_ordinary_plant(
        &observed,
        &ObservedHazardMemory::new(key()),
        request(MovementIntent::Direction(Direction::East)),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert!(matches!(
        certificate.route().steps().first(),
        Some(RouteStep::Move {
            direction: Direction::East,
            updates: 1
        })
    ));
    assert_eq!(certificate.placement().cell, crate::game::CellId::new(3, 1));
    let lane = certificate.route().lane(Tick(7)).unwrap();
    let actor = match &observed.parts().actors[1] {
        ObservedActor::Visible(actor) => actor,
        _ => unreachable!(),
    };
    let mut state = actual_state(
        actor.position,
        actor.heading,
        actor.speed,
        RoomSettings::DEFAULT,
    );
    for _ in 0..6 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    let bomb = state
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.owner() == certificate.placement().owner)
        .unwrap();
    assert_eq!(bomb.placed_at(), certificate.placement().placed_at);
    assert_eq!(bomb.cell(), certificate.placement().cell);

    let mut exited = false;
    while state.tick().0 + 1 < certificate.route().survive_until().0 {
        let next = Tick(state.tick().0 + 1);
        let mut input = Vec::new();
        for (index, step) in certificate.route().steps().iter().enumerate() {
            if index > 0
                && lane.step_start(index) == Some(next)
                && let RouteStep::Move { direction, .. } = step
            {
                input.push(SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Press(*direction),
                });
            }
            if matches!(step, RouteStep::Move { .. }) && lane.stop_update(index) == Some(next) {
                input.push(SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Clear,
                });
            }
        }
        state.step(&input).unwrap();
        let actual = &state.actors()[SeatId::Two.index()];
        assert_eq!(
            lane.pose_at(state.tick()),
            Some((actual.position(), actual.heading()))
        );
        let overlaps = position_overlaps_cell(actual.position(), certificate.placement().cell);
        if exited {
            assert!(!overlaps, "route re-entered its consumed origin grace");
        }
        exited |= !overlaps;
    }
    assert!(exited);
    assert!(state.actors()[SeatId::Two.index()].alive());
}

#[test]
fn confirmation_binds_a_real_id_or_proves_exact_absence() {
    let (basis, certificate) = plan(MovementIntent::Direction(Direction::East));
    let awaiting = current_scene(&basis, &certificate, Tick(6), None);
    assert_eq!(
        certificate.confirm(&basis, &awaiting),
        Ok(PlantPresence::Awaiting)
    );
    let present = current_scene(
        &basis,
        &certificate,
        Tick(8),
        Some((EntityId(400), certificate.placement().range)),
    );
    assert_eq!(
        certificate.confirm(&basis, &present),
        Ok(PlantPresence::Present(EntityId(400)))
    );
    assert_eq!(
        certificate.expected_live_bombs(
            certificate.placement().owner,
            Tick(8),
            PlantPresence::Present(EntityId(400))
        ),
        Some(1)
    );
    let absent = current_scene(&basis, &certificate, Tick(8), None);
    assert_eq!(
        certificate.confirm(&basis, &absent),
        Ok(PlantPresence::Absent)
    );
    let wrong = current_scene(&basis, &certificate, Tick(8), Some((EntityId(401), 7)));
    assert_eq!(
        certificate.confirm(&basis, &wrong),
        Err(PlantFailure::SceneMismatch)
    );
}

#[test]
fn stopped_first_input_is_encoded_as_the_exact_four_update_prefix() {
    let (_, certificate) = plan(MovementIntent::Stop);
    assert_eq!(
        certificate.route().steps().first(),
        Some(&RouteStep::Wait { updates: 4 })
    );
    let lane = certificate.route().lane(Tick(7)).unwrap();
    assert_eq!(lane.step_start(0), Some(Tick(7)));
    assert_eq!(lane.stop_update(0), Some(Tick(7)));
    assert_eq!(lane.ready_update(0), Some(Tick(11)));
}

#[test]
fn placement_modifiers_use_absolute_expiry_and_authoritative_precedence() {
    let observed = observation(Vec::new(), Vec::new());
    let ObservedActor::Visible(mut actor) = observed.parts().actors[1].clone() else {
        unreachable!()
    };
    actor.bomb_range = 5;
    actor.effects = vec![
        ObservedEffect {
            kind: EffectKind::GoldenFlame,
            remaining_ticks: 8,
        },
        ObservedEffect {
            kind: EffectKind::ShortFlame,
            remaining_ticks: 8,
        },
        ObservedEffect {
            kind: EffectKind::ShortFuse,
            remaining_ticks: 8,
        },
        ObservedEffect {
            kind: EffectKind::LongFuse,
            remaining_ticks: 8,
        },
    ];
    let rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    assert_eq!(
        derive_bomb_modifiers(&actor, Tick(0), Tick(7), rules),
        Ok((1, rules.bomb_fuse_ticks()))
    );
    for effect in &mut actor.effects {
        effect.remaining_ticks = 7;
    }
    assert_eq!(
        derive_bomb_modifiers(&actor, Tick(0), Tick(7), rules),
        Ok((5, rules.bomb_fuse_ticks()))
    );
}

#[test]
fn expiring_stopwatch_and_control_modifiers_use_the_absolute_route_clock() {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.effects = vec![ObservedEffect {
        kind: EffectKind::Stopwatch,
        remaining_ticks: 7,
    }];
    actor.voluntary_actions_frozen = false;
    let stopped = Observation::new(parts).unwrap();
    assert!(
        plan_ordinary_plant(
            &stopped,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Direction(Direction::East)),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );

    let mut rival_frozen = stopped.into_parts();
    let ObservedActor::Visible(actor) = &mut rival_frozen.actors[1] else {
        unreachable!()
    };
    actor.voluntary_actions_frozen = true;
    let rival_frozen = Observation::new(rival_frozen).unwrap();
    assert_eq!(
        plan_ordinary_plant(
            &rival_frozen,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Direction(Direction::East)),
            &mut WorkBudget::new(65_536),
        ),
        Err(PlantFailure::Unsupported)
    );

    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.effects = vec![
        ObservedEffect {
            kind: EffectKind::SlowFoot,
            remaining_ticks: 20,
        },
        ObservedEffect {
            kind: EffectKind::ReverseControls,
            remaining_ticks: 20,
        },
    ];
    let controlled = Observation::new(parts).unwrap();
    assert!(
        plan_ordinary_plant(
            &controlled,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Direction(Direction::West)),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );
}
