use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::{
    BombKnowledge, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedFlame,
    UnknownBombFacts, WorkBudget,
};
use crate::game::{
    BlastKind, CellId, Direction, EnabledItems, EntityId, Equipment, MovementSegment, Position,
    Tick,
};

use super::punch::{select_landing, select_landing_unmetered};
use super::route_expand::retain_recipient_barrier;
use super::safety::{RecipientBarrier, check_segments_with_grace, stationary_segment};
use super::tests::{key, observation};
use super::{
    PunchFailure, PunchOutcome, PunchRequest, RouteAssurance, RouteFailure, RouteLimits, RouteMode,
    RouteStep, plan_punch,
};

pub(super) fn punch_observation() -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.settings = parts.rules.settings.with_enabled_items(EnabledItems::PUNCH);
    let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
        unreachable!()
    };
    rival.live_bomb_count = 1;
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.position = Position::center(CellId::new(7, 7));
    cpu.heading = Direction::East;
    cpu.speed = crate::game::SpeedLevel::Four;
    cpu.equipment = Some(Equipment::Punch);
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: Tick(0),
        cell: CellId::new(8, 7),
        owner: EntityId(1),
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 180,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    Observation::new(parts).unwrap()
}

pub(super) fn request() -> PunchRequest {
    PunchRequest {
        execution_tick: Tick(7),
        settled_stop_tick: Tick(0),
        limits: RouteLimits::default(),
    }
}

fn plan() -> (Observation, super::PunchCertificate) {
    let basis = punch_observation();
    let mut work = WorkBudget::new(32_768);
    let certificate = plan_punch(
        &basis,
        &ObservedHazardMemory::new(key()),
        request(),
        &mut work,
    )
    .unwrap_or_else(|error| panic!("punch failed after {} work: {error:?}", work.spent()));
    (basis, certificate)
}

fn small_punch_observation() -> Observation {
    let mut parts = punch_observation().into_parts();
    parts.rules.version = 6;
    parts.arena.width = 11;
    parts.arena.height = 9;
    parts.arena.terrain = vec![crate::game::Terrain::Floor; 99];
    for row in 0..9 {
        for column in 0..11 {
            if row == 0 || column == 0 || row == 8 || column == 10 {
                parts.arena.terrain[row * 11 + column] = crate::game::Terrain::Wall;
            }
        }
    }
    let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
        unreachable!()
    };
    rival.position = Position::center(CellId::new(9, 7));
    Observation::new(parts).unwrap()
}

#[test]
fn small_board_punch_uses_the_authoritative_short_axis_cycle() {
    let observed = small_punch_observation();
    let BombKnowledge::Visible(bombs) = &observed.parts().bombs else {
        unreachable!()
    };
    assert_eq!(
        select_landing_unmetered(&observed, &bombs[0], Position::center(CellId::new(7, 7)),),
        Ok(CellId::new(2, 7))
    );
}

#[test]
fn small_board_punch_charges_actual_geometry_before_landing_search() {
    let observed = small_punch_observation();
    let BombKnowledge::Visible(bombs) = &observed.parts().bombs else {
        unreachable!()
    };
    let actor = Position::center(CellId::new(7, 7));
    let mut exact = WorkBudget::new(99);
    assert_eq!(
        select_landing(&observed, &bombs[0], actor, &mut exact),
        Ok(CellId::new(2, 7))
    );
    assert_eq!(exact.spent(), 99);

    let mut short = WorkBudget::new(98);
    assert_eq!(
        select_landing(&observed, &bombs[0], actor, &mut short),
        Err(PunchFailure::Route(RouteFailure::WorkExhausted))
    );
    assert_eq!(short.spent(), 0);
    assert_eq!(short.remaining(), 98);

    let large = punch_observation();
    let BombKnowledge::Visible(large_bombs) = &large.parts().bombs else {
        unreachable!()
    };
    let mut legacy = WorkBudget::new(195);
    assert!(select_landing(&large, &large_bombs[0], actor, &mut legacy).is_ok());
    assert_eq!(legacy.spent(), 195);
}

pub(super) fn scene(
    basis: &Observation,
    certificate: &super::PunchCertificate,
    tick: Tick,
    cell: CellId,
) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - basis.parts().tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let signature = certificate.bomb();
    let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
        unreachable!()
    };
    bombs[0].cell = cell;
    bombs[0].trigger = ObservedBombTrigger::Timed {
        remaining_ticks: u16::try_from(signature.detonates_at.0 - tick.0).unwrap(),
        paused: false,
    };
    Observation::new(parts).unwrap()
}

#[test]
fn dual_branch_route_and_confirmation_preserve_the_actual_bomb_signature() {
    let (basis, certificate) = plan();
    assert_eq!(certificate.execution_tick(), Tick(7));
    assert_eq!(certificate.route().assurance(), &RouteAssurance::Certified);
    assert_eq!(
        certificate.route().steps().first(),
        Some(&RouteStep::Wait { updates: 4 })
    );

    let awaiting = scene(&basis, &certificate, Tick(6), certificate.bomb().source);
    assert!(certificate.can_admit(&basis, &awaiting, Tick(6)));
    assert_eq!(
        certificate.confirm(&basis, &awaiting),
        Ok(PunchOutcome::Awaiting)
    );
    let unchanged = scene(&basis, &certificate, Tick(7), certificate.bomb().source);
    assert_eq!(
        certificate.confirm(&basis, &unchanged),
        Ok(PunchOutcome::Unchanged)
    );
    let relocated = scene(&basis, &certificate, Tick(7), certificate.landing());
    assert_eq!(
        certificate.confirm(&basis, &relocated),
        Ok(PunchOutcome::Relocated)
    );

    let displaced = scene(&basis, &certificate, Tick(7), CellId::new(4, 1));
    assert_eq!(
        certificate.confirm(&basis, &displaced),
        Err(PunchFailure::SceneMismatch)
    );
}

#[test]
fn admission_rechecks_the_rival_envelope_and_work_exhaustion_fails_closed() {
    let (basis, certificate) = plan();
    let mut current = scene(&basis, &certificate, Tick(6), certificate.bomb().source).into_parts();
    let ObservedActor::Visible(rival) = &mut current.actors[0] else {
        unreachable!()
    };
    rival.position = Position::center(certificate.landing());
    assert!(!certificate.can_admit(&basis, &Observation::new(current).unwrap(), Tick(6)));

    let mut changed = scene(&basis, &certificate, Tick(6), certificate.bomb().source).into_parts();
    let ObservedActor::Visible(rival) = &mut changed.actors[0] else {
        unreachable!()
    };
    rival.id = EntityId(91);
    assert!(!certificate.can_admit(&basis, &Observation::new(changed).unwrap(), Tick(6)));

    let mut threatened = punch_observation().into_parts();
    let ObservedActor::Visible(rival) = &mut threatened.actors[0] else {
        unreachable!()
    };
    rival.position = Position::center(CellId::new(11, 7));
    assert_eq!(
        plan_punch(
            &Observation::new(threatened).unwrap(),
            &ObservedHazardMemory::new(key()),
            request(),
            &mut WorkBudget::new(65_536),
        ),
        Err(PunchFailure::Unsupported),
        "the first candidate is rejected instead of skipping to the next landing"
    );

    assert!(matches!(
        plan_punch(
            &basis,
            &ObservedHazardMemory::new(key()),
            request(),
            &mut WorkBudget::new(0),
        ),
        Err(PunchFailure::Route(RouteFailure::WorkExhausted))
    ));
}

#[test]
fn legacy_admission_preserves_the_supported_clear_rival_movement_path() {
    let (basis, certificate) = plan();
    let mut current = scene(&basis, &certificate, Tick(6), certificate.bomb().source).into_parts();
    let ObservedActor::Visible(rival) = &mut current.actors[0] else {
        unreachable!()
    };
    rival.position = Position::center(CellId::new(2, 2));
    assert!(certificate.can_admit(&basis, &Observation::new(current).unwrap(), Tick(6)));
}

#[test]
fn recipient_barrier_waives_only_named_motion_blocking_uncertainty() {
    let basis = punch_observation();
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let position = Position::center(CellId::new(7, 7));
    let segments = stationary_segment(position);
    let window = TickWindow {
        start: Tick(1),
        end: Tick(7),
    };
    assert!(
        check_segments_with_grace(
            &hazards,
            &segments,
            window,
            RouteMode::CertifiedOnly,
            None,
            Some(RecipientBarrier {
                cell: CellId::new(7, 7),
                bomb_id: EntityId(90),
            }),
        )
        .is_ok()
    );
    assert_eq!(
        check_segments_with_grace(
            &hazards,
            &segments,
            window,
            RouteMode::CertifiedOnly,
            None,
            Some(RecipientBarrier {
                cell: CellId::new(7, 7),
                bomb_id: EntityId(91),
            }),
        ),
        Err(RouteFailure::NoRoute)
    );

    let mut lethal = basis.into_parts();
    lethal.flames.push(ObservedFlame {
        cell: CellId::new(7, 7),
        remaining_ticks: 20,
    });
    let lethal = Observation::new(lethal).unwrap();
    let hazards = forecast(
        &lethal,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        check_segments_with_grace(
            &hazards,
            &segments,
            window,
            RouteMode::CertifiedOnly,
            None,
            Some(RecipientBarrier {
                cell: CellId::new(7, 7),
                bomb_id: EntityId(90),
            }),
        ),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn recipient_barrier_is_consumed_on_exit_and_cannot_be_restored_by_reentry() {
    let cell = CellId::new(7, 7);
    let barrier = Some(RecipientBarrier {
        cell,
        bomb_id: EntityId(90),
    });
    let center = Position::center(cell);
    let outside = Position::center(CellId::new(7, 8));
    let exited = retain_recipient_barrier(barrier, &[MovementSegment::new(center, outside)]);
    assert_eq!(exited, None);
    assert_eq!(
        retain_recipient_barrier(exited, &[MovementSegment::new(outside, center)]),
        None
    );
}
