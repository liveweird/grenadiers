use crate::cpu::{Observation, ObservedActor, UnknownActorFacts, VisibleActor};
use crate::game::{CellId, Direction, EntityId, Equipment, Position, SeatId, SpeedLevel, Tick};

use super::{CommittedPulse, bracelet_guard_tests, bracelet_tests};

fn third_actor(position: Position) -> ObservedActor {
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

fn multiplayer_bracelet(position: Position) -> Observation {
    let mut parts = bracelet_tests::bracelet_observation(40).into_parts();
    parts.rules.version = 6;
    parts.actors.push(third_actor(position));
    Observation::new(parts).unwrap()
}

#[test]
fn bracelet_planning_checks_third_party_victim_sweep_blockers() {
    let distant = multiplayer_bracelet(Position::center(CellId::new(3, 3)));
    assert!(bracelet_tests::tactic(&distant).is_some());

    let blocking = multiplayer_bracelet(Position::center(CellId::new(9, 7)));
    assert!(bracelet_tests::tactic(&blocking).is_none());
}

#[test]
fn bracelet_planning_withdraws_when_any_rival_is_hidden() {
    let mut parts = multiplayer_bracelet(Position::center(CellId::new(3, 3))).into_parts();
    parts.actors[2] = ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    };
    assert!(bracelet_tests::tactic(&Observation::new(parts).unwrap()).is_none());
}

#[test]
fn bracelet_planning_withdraws_for_a_third_bracelet_countershove_threat() {
    let mut parts = multiplayer_bracelet(Position::center(CellId::new(7, 6))).into_parts();
    let ObservedActor::Visible(third) = &mut parts.actors[2] else {
        unreachable!()
    };
    third.equipment = Some(Equipment::Bracelet);
    assert!(bracelet_tests::tactic(&Observation::new(parts).unwrap()).is_none());
}

#[test]
fn bracelet_admission_rechecks_third_party_blockers_and_hidden_rivals() {
    let mut parts = bracelet_guard_tests::bracelet_observation().into_parts();
    parts.rules.version = 6;
    parts
        .actors
        .push(third_actor(Position::center(CellId::new(3, 3))));
    let basis = Observation::new(parts).unwrap();
    let lease = bracelet_guard_tests::lease(&basis, true);
    assert!(CommittedPulse::admit(lease.clone(), basis.clone(), &basis, Tick(10)).is_ok());

    let mut blocked = basis.clone().into_parts();
    let ObservedActor::Visible(third) = &mut blocked.actors[2] else {
        unreachable!()
    };
    third.position = Position::center(CellId::new(9, 7));
    assert!(
        CommittedPulse::admit(
            lease.clone(),
            basis.clone(),
            &Observation::new(blocked).unwrap(),
            Tick(10)
        )
        .is_err()
    );

    let mut hidden = basis.clone().into_parts();
    hidden.actors[2] = ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    };
    assert!(
        CommittedPulse::admit(lease, basis, &Observation::new(hidden).unwrap(), Tick(10)).is_err()
    );
}

#[test]
fn bracelet_admission_withdraws_for_a_new_third_party_countershove_threat() {
    let mut parts = bracelet_guard_tests::bracelet_observation().into_parts();
    parts.rules.version = 6;
    parts
        .actors
        .push(third_actor(Position::center(CellId::new(7, 6))));
    let basis = Observation::new(parts).unwrap();
    let lease = bracelet_guard_tests::lease(&basis, true);
    let mut threatened = basis.clone().into_parts();
    let ObservedActor::Visible(third) = &mut threatened.actors[2] else {
        unreachable!()
    };
    third.equipment = Some(Equipment::Bracelet);
    assert!(
        CommittedPulse::admit(
            lease,
            basis,
            &Observation::new(threatened).unwrap(),
            Tick(10)
        )
        .is_err()
    );
}
