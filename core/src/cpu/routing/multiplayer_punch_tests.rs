use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{Observation, ObservedActor, UnknownActorFacts, VisibleActor, WorkBudget};
use crate::game::{CellId, Direction, EntityId, Position, SeatId, SpeedLevel, Tick};

use super::punch_tests::{punch_observation, request, scene};
use super::tests::key;
use super::{PunchFailure, plan_punch};

fn third(position: Position) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(3),
        seat: SeatId::Three,
        position,
        heading: Direction::West,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

fn multiplayer(position: Position) -> Observation {
    let mut parts = punch_observation().into_parts();
    parts.rules.version = 6;
    parts.actors.push(third(position));
    Observation::new(parts).unwrap()
}

fn plan(observation: &Observation) -> Result<super::PunchCertificate, PunchFailure> {
    plan_punch(
        observation,
        &ObservedHazardMemory::new(key()),
        request(),
        &mut WorkBudget::new(65_536),
    )
}

#[test]
fn punch_planning_checks_every_visible_rival_and_hidden_unknowns() {
    let basis = multiplayer(Position::center(CellId::new(3, 3)));
    let certificate = plan(&basis).unwrap();

    let mut blocked = basis.clone().into_parts();
    let ObservedActor::Visible(third) = &mut blocked.actors[2] else {
        unreachable!()
    };
    third.position = Position::center(certificate.landing());
    assert_eq!(
        plan(&Observation::new(blocked).unwrap()),
        Err(PunchFailure::Unsupported)
    );

    let mut hidden = basis.into_parts();
    hidden.actors[2] = ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    };
    assert_eq!(
        plan(&Observation::new(hidden).unwrap()),
        Err(PunchFailure::Unsupported)
    );
}

#[test]
fn punch_admission_rechecks_third_rival_position_and_visibility() {
    let basis = multiplayer(Position::center(CellId::new(3, 3)));
    let certificate = plan(&basis).unwrap();
    let awaiting = scene(&basis, &certificate, Tick(6), certificate.bomb().source);
    assert!(certificate.can_admit(&basis, &awaiting, Tick(6)));

    let mut moved = awaiting.clone().into_parts();
    let ObservedActor::Visible(third) = &mut moved.actors[2] else {
        unreachable!()
    };
    third.position = Position::center(certificate.landing());
    assert!(!certificate.can_admit(&basis, &Observation::new(moved).unwrap(), Tick(6)));

    let mut hidden = awaiting.into_parts();
    hidden.actors[2] = ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    };
    assert!(!certificate.can_admit(&basis, &Observation::new(hidden).unwrap(), Tick(6)));
}
