use crate::cpu::policy::UtilityMemory;
use crate::cpu::{BombKnowledge, MemoryKey, MovementIntent, ObservedActor};
use crate::game::{Direction, Position, Tick};

use super::refuge_handoff_tests::{CPU, ORIGIN, fixture, input, plan, reach_terminal_refuge};
use super::reliable_v27_oil_tests::observe;
use super::{CommittedRoute, LairSurvivalPlan, RouteStatus};

fn terminal_scene() -> (
    crate::game::State,
    crate::room::RoomSettings,
    CommittedRoute,
    crate::cpu::Observation,
    Tick,
    LairSurvivalPlan,
) {
    let (mut state, settings) = fixture();
    for _ in 0..48 {
        state
            .step(&input(MovementIntent::Direction(Direction::North), false))
            .unwrap();
    }
    state.step(&input(MovementIntent::Stop, false)).unwrap();
    let basis = observe(&state, settings);
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: CPU,
    };
    let (planned, _) = plan(&basis, &UtilityMemory::new(key));
    let (route, refuge, settled) = reach_terminal_refuge(&mut state, settings, basis, &planned);
    (state, settings, route, refuge, settled, planned)
}

fn bypass_publication_only(
    route: &CommittedRoute,
    changed: crate::cpu::Observation,
    settled: Tick,
) {
    let mut route = route.clone();
    route.last_observation = changed.clone();
    assert!(
        route
            .terminal_refuge_handoff(&changed, Some(settled))
            .is_none()
    );
}

#[test]
fn terminal_guards_reject_their_own_invalid_fact_after_publication_validation() {
    let (_, _, route, refuge, settled, _) = terminal_scene();

    let changed_scene = refuge
        .clone()
        .map_parts(|parts| {
            let rival = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() == crate::game::SeatId::One)
                .unwrap();
            let ObservedActor::Visible(rival) = rival else {
                unreachable!()
            };
            rival.heading = Direction::West;
        })
        .unwrap();
    assert!(
        route
            .terminal_refuge_handoff(&changed_scene, Some(settled))
            .is_none()
    );
    let stale = refuge
        .clone()
        .map_parts(|parts| parts.tick = Tick(parts.tick.0 + 1))
        .unwrap();
    assert!(
        route
            .terminal_refuge_handoff(&stale, Some(settled))
            .is_none()
    );

    let wrong_key = refuge
        .clone()
        .map_parts(|parts| parts.world.physical_round += 1)
        .unwrap();
    bypass_publication_only(&route, wrong_key, settled);

    let no_capacity = refuge
        .clone()
        .map_parts(|parts| {
            let actor = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() == CPU)
                .unwrap();
            let ObservedActor::Visible(actor) = actor else {
                unreachable!()
            };
            actor.bomb_capacity = actor.live_bomb_count;
        })
        .unwrap();
    bypass_publication_only(&route, no_capacity, settled);

    let origin_overlap = refuge
        .clone()
        .map_parts(|parts| {
            let actor = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() == CPU)
                .unwrap();
            let ObservedActor::Visible(actor) = actor else {
                unreachable!()
            };
            actor.position = Position::center(ORIGIN);
        })
        .unwrap();
    bypass_publication_only(&route, origin_overlap, settled);

    let missing_bomb = refuge
        .clone()
        .map_parts(|parts| parts.bombs = BombKnowledge::Visible(Vec::new()))
        .unwrap();
    bypass_publication_only(&route, missing_bomb, settled);
}

#[test]
fn terminal_guard_requires_more_than_the_normal_admission_delay_remaining() {
    let (mut state, settings, mut route, _, settled, _) = terminal_scene();
    let deadline = route.certificate.survive_until();
    loop {
        state.step(&[]).unwrap();
        let current = observe(&state, settings);
        assert_ne!(
            route.observe_with_stop(&current, Some(settled)),
            RouteStatus::Invalidated
        );
        if current.parts().tick.0 + super::MAX_PLAN_AGE_TICKS >= deadline.0 {
            assert!(
                route
                    .terminal_refuge_handoff(&current, Some(settled))
                    .is_none()
            );
            break;
        }
    }
}
