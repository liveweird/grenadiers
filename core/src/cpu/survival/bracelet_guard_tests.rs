use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::tests::{key, observation};
use crate::cpu::routing::{PulseRequest, RouteMode, certify_pulse};
use crate::cpu::{BombKnowledge, MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{CellId, Direction, EnabledItems, EntityId, Equipment, Tick};

use super::{CommittedPulse, PulseStatus, WORK_ALLOWANCE};

pub(super) fn bracelet_observation() -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::BRACELET);
    parts.actors[0] = ObservedActor::Visible(crate::cpu::VisibleActor {
        id: EntityId(1),
        seat: crate::game::SeatId::One,
        position: crate::game::Position::center(CellId::new(8, 7)),
        heading: Direction::West,
        speed: crate::game::SpeedLevel::Base,
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
        unknown: crate::cpu::UnknownActorFacts::ALL,
    });
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.equipment = Some(Equipment::Bracelet);
    Observation::new(parts).unwrap()
}

pub(super) fn lease(basis: &Observation, guarded: bool) -> crate::cpu::routing::PulseLease {
    let hazards = forecast(
        basis,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let lease = certify_pulse(
        basis,
        &hazards,
        PulseRequest {
            movement: MovementIntent::Direction(Direction::East),
            hold_ticks: 2,
            start_window: TickWindow {
                start: Tick(11),
                end: Tick(17),
            },
            stopped_safe_until: Tick(30),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: Tick(10),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    if guarded {
        lease.with_bracelet_guard(EntityId(1), Direction::East)
    } else {
        lease
    }
}

#[test]
fn guarded_admission_requires_the_named_visible_shoveable_rival() {
    let basis = bracelet_observation();
    assert!(CommittedPulse::admit(lease(&basis, true), basis.clone(), &basis, Tick(10)).is_ok());

    let mut countershove = basis.clone().into_parts();
    let ObservedActor::Visible(rival) = &mut countershove.actors[0] else {
        unreachable!()
    };
    rival.equipment = Some(Equipment::Bracelet);
    let countershove = Observation::new(countershove).unwrap();
    assert!(
        CommittedPulse::admit(lease(&basis, true), basis.clone(), &countershove, Tick(10)).is_err()
    );

    let mut invisible = basis.clone().into_parts();
    invisible.actors[0] = ObservedActor::Hidden {
        id: EntityId(1),
        seat: crate::game::SeatId::One,
        alive: true,
    };
    let invisible = Observation::new(invisible).unwrap();
    assert!(
        CommittedPulse::admit(lease(&basis, true), basis.clone(), &invisible, Tick(10)).is_err()
    );
    assert!(
        CommittedPulse::admit(lease(&basis, false), basis.clone(), &invisible, Tick(10)).is_ok()
    );
}

#[test]
fn guarded_execution_stops_when_the_rival_can_countershove() {
    let basis = bracelet_observation();
    let lease = lease(&basis, true);
    let committed = CommittedPulse::admit(lease.clone(), basis.clone(), &basis, Tick(10)).unwrap();
    let mut current =
        super::tests::later(&basis, 11, lease.pose_after(1).min(), Direction::East).into_parts();
    let ObservedActor::Visible(rival) = &mut current.actors[0] else {
        unreachable!()
    };
    rival.equipment = Some(Equipment::Bracelet);
    assert_eq!(
        committed.validate(&Observation::new(current).unwrap(), Tick(11)),
        PulseStatus::Invalidated
    );
}

#[test]
fn bracelet_guard_does_not_treat_unknown_bombs_as_empty() {
    let basis = bracelet_observation();
    let lease = lease(&basis, true);
    let mut unknown = basis.clone().into_parts();
    unknown.bombs = BombKnowledge::Unavailable;
    assert!(
        CommittedPulse::admit(lease, basis, &Observation::new(unknown).unwrap(), Tick(10)).is_err()
    );
}
