use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::routing::{
    PlantCertificate, PlantRequest, PulseRequest, RouteFailure, RouteLimits, RouteMode,
    certify_lair_pulse, plan_lair_ordinary_plant,
};
use crate::cpu::{MemoryKey, MovementIntent, ObservedActor, WorkBudget};
use crate::game::{CellId, Direction, Position, Tick};

use super::v138_farm_witness_fixture::{CPU, captured_tick_5580};
use super::{MobContactLease, PlantContactProof};

const ALLOWANCE: u32 = 65_536;
// The public sample contains the pose at tick 5_580, but not the controller's
// private settled-stop clock or subsequent inputs. Every certificate below is
// conditional on this stop/motion premise rather than proving it from capture.
const ASSUMED_SETTLED_STOP_TICK: Tick = Tick(5_580);
const ASSUMED_EXECUTION_TICK: Tick = Tick(5_587);

fn certified_plant(
    observation: &crate::cpu::Observation,
    direction: Direction,
) -> (PlantCertificate, u32) {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let mut work = WorkBudget::new(ALLOWANCE);
    let plant = plan_lair_ordinary_plant(
        observation,
        &history,
        PlantRequest {
            execution_tick: ASSUMED_EXECUTION_TICK,
            settled_stop_tick: ASSUMED_SETTLED_STOP_TICK,
            first_movement: MovementIntent::Direction(direction),
            limits: RouteLimits::default(),
        },
        &mut work,
    )
    .unwrap();
    (plant, work.spent())
}

fn counterfactual_alternative_origin() -> crate::cpu::Observation {
    captured_tick_5580()
        .map_parts(|parts| {
            let actor = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() == CPU)
                .unwrap();
            let ObservedActor::Visible(actor) = actor else {
                unreachable!()
            };
            actor.position = Position::center(CellId::new(1, 6));
            actor.heading = Direction::South;
        })
        .unwrap()
}

#[test]
fn captured_public_scene_with_assumed_stop_plant_fails_both_contact_certificates() {
    let observation = captured_tick_5580();
    let (plant, route_work) = certified_plant(&observation, Direction::North);
    assert_eq!(plant.placement().cell, CellId::new(4, 5));
    assert_eq!(plant.placement().detonates_at, Tick(5_767));
    assert_eq!(plant.route().survive_until(), Tick(5_797));
    assert_eq!(route_work, 12_157);

    // The range-two blast reaches both blockers that separate the western
    // corridor from the central mob component.
    assert!(
        observation
            .parts()
            .arena
            .blocks
            .contains(&CellId::new(2, 5))
    );
    assert!(
        observation
            .parts()
            .arena
            .blocks
            .contains(&CellId::new(5, 5))
    );

    let mut lease_work = WorkBudget::new(ALLOWANCE);
    assert_eq!(
        MobContactLease::certify_route(&observation, plant.route(), &mut lease_work),
        Err(RouteFailure::NoRoute)
    );
    assert_eq!(lease_work.spent(), 504);
    let mut topology_work = WorkBudget::new(ALLOWANCE);
    assert!(PlantContactProof::certify(&observation, &plant, &mut topology_work).is_none());
    assert_eq!(topology_work.spent(), 2_630);
}

#[test]
fn captured_public_scene_with_assumed_settled_stop_has_one_checked_westward_pulse() {
    let observation = captured_tick_5580();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    let mut work = WorkBudget::new(ALLOWANCE);
    let hazards = forecast(
        &observation,
        &history,
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let (pulse, contact) = certify_lair_pulse(
        &observation,
        &hazards,
        PulseRequest {
            movement: MovementIntent::Direction(Direction::West),
            hold_ticks: 12,
            start_window: TickWindow {
                start: Tick(5_581),
                end: Tick(5_588),
            },
            stopped_safe_until: Tick(5_609),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: ASSUMED_SETTLED_STOP_TICK,
        },
        &mut work,
    )
    .unwrap();
    assert_eq!(pulse.movement(), MovementIntent::Direction(Direction::West));
    assert_eq!(pulse.hold_ticks(), 12);
    assert_eq!(pulse.safe_until(), Tick(5_609));
    assert_eq!(contact.safe_until(), Tick(5_609));
    assert!(work.spent() <= ALLOWANCE);
}

#[test]
fn alternative_origin_is_safe_only_after_counterfactual_relocation() {
    let observation = counterfactual_alternative_origin();
    let (plant, route_work) = certified_plant(&observation, Direction::South);
    assert_eq!(plant.placement().cell, CellId::new(1, 6));
    assert_eq!(plant.placement().detonates_at, Tick(5_767));
    assert_eq!(plant.route().survive_until(), Tick(5_797));
    assert_eq!(route_work, 15_698);

    // North is stopped by the desired block, east by the fixed wall at (2,6),
    // and the remaining rays contain no soft blocks.
    assert!(
        observation
            .parts()
            .arena
            .blocks
            .contains(&CellId::new(1, 5))
    );
    assert!(
        !observation
            .parts()
            .arena
            .blocks
            .contains(&CellId::new(1, 7))
    );
    assert!(
        !observation
            .parts()
            .arena
            .blocks
            .contains(&CellId::new(1, 8))
    );

    let mut lease_work = WorkBudget::new(ALLOWANCE);
    let lease = MobContactLease::certify_route(&observation, plant.route(), &mut lease_work)
        .expect("counterfactual endpoint has a full contact-safe flame escape");
    assert_eq!(lease.safe_until(), Tick(5_797));
    assert_eq!(lease_work.spent(), 621);
    let mut topology_work = WorkBudget::new(ALLOWANCE);
    assert!(PlantContactProof::certify(&observation, &plant, &mut topology_work).is_some());
    assert_eq!(topology_work.spent(), 2_966);
}
