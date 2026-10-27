use crate::cpu::{
    BombKnowledge, ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedFlame,
    UnknownBombFacts,
};
use crate::game::{BlastKind, CellId, Direction, EntityId};

use super::tests::{key, observation};
use super::{UtilityMemory, follow_committed_escape};

fn committed() -> UtilityMemory {
    UtilityMemory::from_route(
        key(),
        &[CellId::new(7, 7), CellId::new(8, 7), CellId::new(8, 6)],
        true,
    )
}

#[test]
fn inactive_or_cleared_escape_returns_none() {
    let start = observation(CellId::new(7, 7));
    assert_eq!(
        follow_committed_escape(&start, &UtilityMemory::new(key())),
        Ok(None)
    );
    let waypoint = CellId::new(8, 7);
    let (_, advanced) = committed()
        .next(waypoint, crate::game::Position::center(waypoint))
        .unwrap();
    let refuge = observation(CellId::new(8, 6));
    assert_eq!(follow_committed_escape(&refuge, &advanced), Ok(None));
}

#[test]
fn committed_escape_rejects_a_different_physical_world() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.world.physical_round += 1;
    let observation = crate::cpu::Observation::new(parts).unwrap();
    assert_eq!(
        follow_committed_escape(&observation, &committed()),
        Err(super::PolicyError::IdentityMismatch)
    );
}

#[test]
fn committed_steps_never_emit_manual_actions_or_choose_new_goals() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(9),
        placed_at: crate::game::Tick(0),
        cell: CellId::new(7, 7),
        owner: EntityId(2),
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 50,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    let observation = crate::cpu::Observation::new(parts).unwrap();
    let plan = follow_committed_escape(&observation, &committed())
        .unwrap()
        .unwrap();
    assert_eq!(plan.intent.manual, None);
    assert_eq!(
        plan.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::East)
    );
}

#[test]
fn blocked_unknown_or_unsupported_escape_stops_and_retains_route() {
    let memory = committed();
    let mut blocked = observation(CellId::new(7, 7)).into_parts();
    blocked.flames.push(ObservedFlame {
        cell: CellId::new(8, 7),
        remaining_ticks: 2,
    });
    let plan = follow_committed_escape(&crate::cpu::Observation::new(blocked).unwrap(), &memory)
        .unwrap()
        .unwrap();
    assert_eq!(plan.intent, crate::cpu::Intent::STOP);
    assert_eq!(plan.next_memory, memory);

    let mut frozen = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut frozen.actors[1] else {
        unreachable!()
    };
    actor.voluntary_actions_frozen = true;
    actor.effects.push(crate::cpu::ObservedEffect {
        kind: crate::game::EffectKind::Diarrhea,
        remaining_ticks: 1,
    });
    let plan = follow_committed_escape(&crate::cpu::Observation::new(frozen).unwrap(), &memory)
        .unwrap()
        .unwrap();
    assert_eq!(plan.intent, crate::cpu::Intent::STOP);
    assert_eq!(plan.next_memory, memory);

    let mut unknown = observation(CellId::new(7, 7)).into_parts();
    unknown.bombs = BombKnowledge::Unavailable;
    let plan = follow_committed_escape(&crate::cpu::Observation::new(unknown).unwrap(), &memory)
        .unwrap()
        .unwrap();
    assert_eq!(plan.intent, crate::cpu::Intent::STOP);
    assert_eq!(plan.next_memory, memory);
}

#[test]
fn displaced_nonadjacent_or_diagonal_actor_stops_with_unchanged_route() {
    let memory = committed();
    for displaced in [CellId::new(5, 7), CellId::new(6, 6)] {
        let observation = observation(displaced);
        let plan = follow_committed_escape(&observation, &memory)
            .unwrap()
            .unwrap();
        assert_eq!(plan.intent, crate::cpu::Intent::STOP);
        assert_eq!(plan.intent.manual, None);
        assert_eq!(plan.next_memory, memory);
    }
}

#[test]
fn committed_servo_rejects_off_corridor_and_away_heading_geometry() {
    let memory = committed();
    let mut off_axis = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut off_axis.actors[1] else {
        unreachable!()
    };
    actor.position.y += 65;
    let plan = follow_committed_escape(&crate::cpu::Observation::new(off_axis).unwrap(), &memory)
        .unwrap()
        .unwrap();
    assert_eq!(plan.intent, crate::cpu::Intent::STOP);
    assert_eq!(plan.next_memory, memory);

    let mut away = observation(CellId::new(8, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut away.actors[1] else {
        unreachable!()
    };
    actor.position.x -= 100;
    actor.heading = Direction::West;
    let plan = follow_committed_escape(&crate::cpu::Observation::new(away).unwrap(), &memory)
        .unwrap()
        .unwrap();
    assert_eq!(plan.intent, crate::cpu::Intent::STOP);
    assert_eq!(plan.next_memory, memory);
}

#[test]
fn committed_servo_accepts_the_literal_perpendicular_boundary() {
    let mut boundary = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut boundary.actors[1] else {
        unreachable!()
    };
    actor.position.y += 64;
    let plan = follow_committed_escape(
        &crate::cpu::Observation::new(boundary).unwrap(),
        &committed(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        plan.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::East)
    );
    assert_eq!(plan.intent.manual, None);
}
