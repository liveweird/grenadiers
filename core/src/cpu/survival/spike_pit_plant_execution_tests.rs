use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::routing::{
    PlantCertificate, PlantPresence, PlantRequest, RouteLimits, RouteStep, plan_lair_ordinary_plant,
};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedSpikePits, UnknownBombFacts, WorkBudget,
};
use crate::game::{EntityId, Tick};

use super::spike_pit_execution_tests::observed;
use super::{CommittedRoute, ExecutionCertificate, RouteStatus};

fn key(observation: &Observation) -> crate::cpu::MemoryKey {
    crate::cpu::MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    }
}

fn planned_plant() -> (Observation, PlantCertificate, MovementIntent) {
    let basis = observed(0, crate::game::SpikePitPhase::Idle, 0);
    let certificate = plan_lair_ordinary_plant(
        &basis,
        &ObservedHazardMemory::new(key(&basis)),
        PlantRequest {
            execution_tick: Tick(7),
            settled_stop_tick: Tick(0),
            first_movement: MovementIntent::Direction(crate::game::Direction::East),
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(super::WORK_ALLOWANCE),
    )
    .unwrap();
    let movement = match certificate.route().steps().first().unwrap() {
        RouteStep::Move { direction, .. } => MovementIntent::Direction(*direction),
        RouteStep::Wait { .. } => MovementIntent::Stop,
    };
    (basis, certificate, movement)
}

fn scene(
    basis: &Observation,
    certificate: &PlantCertificate,
    tick: Tick,
    present: Option<EntityId>,
) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - basis.parts().tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let lane = certificate.route().lane(Tick(7)).unwrap();
    let (position, heading) = lane.pose_at(tick).unwrap();
    let actor = parts
        .actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
            _ => None,
        })
        .unwrap();
    actor.position = position;
    actor.heading = heading;
    actor.live_bomb_count = u16::from(present.is_some());
    let presence = present.map_or_else(
        || {
            if tick < certificate.placement().placed_at {
                PlantPresence::Awaiting
            } else {
                PlantPresence::Absent
            }
        },
        PlantPresence::Present,
    );
    actor.weapon = certificate.expected_weapon(presence);
    parts.bombs = BombKnowledge::Visible(
        present
            .map(|id| {
                let placement = certificate.placement();
                ObservedBomb {
                    id,
                    placed_at: placement.placed_at,
                    cell: placement.cell,
                    owner: placement.owner,
                    range: placement.range,
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
    let guard = lane.spike_pit_guard_at(tick).unwrap();
    Observation::new_rules_v10(
        parts,
        basis.environment().unwrap().clone(),
        basis.respawn_lairs().cloned(),
        Some(ObservedSpikePits::new(
            guard.config(),
            guard.expected().to_vec(),
        )),
    )
    .unwrap()
}

fn awaiting_route() -> (PlantCertificate, CommittedRoute, Observation) {
    let (basis, certificate, movement) = planned_plant();
    let awaiting = scene(&basis, &certificate, Tick(6), None);
    let route = CommittedRoute::admit_execution(
        ExecutionCertificate::Plant(Box::new(certificate.clone())),
        basis,
        &awaiting,
        Tick(6),
        movement,
    )
    .unwrap();
    (certificate, route, awaiting)
}

#[test]
fn present_and_absent_plant_outcomes_continue_under_the_certified_pit_timeline() {
    let (certificate, mut present_route, _) = awaiting_route();
    let basis = present_route.basis.clone();
    let present = scene(&basis, &certificate, Tick(7), Some(EntityId(700)));
    assert_ne!(present_route.observe(&present), RouteStatus::Invalidated);
    assert_eq!(
        present_route.plant_presence(),
        Some(PlantPresence::Present(EntityId(700)))
    );

    let (certificate, mut absent_route, _) = awaiting_route();
    let basis = absent_route.basis.clone();
    assert!(absent_route.select_absent_plant());
    let absent = scene(&basis, &certificate, Tick(7), None);
    assert_ne!(absent_route.observe(&absent), RouteStatus::Invalidated);
    assert_eq!(absent_route.plant_presence(), Some(PlantPresence::Absent));
}
