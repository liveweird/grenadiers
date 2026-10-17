use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::{EntityId, RoundResult, SeatId, Tick as DomainTick};
use crate::protocol::{Counter, RulesVersion, Seat, WireRoundResult, project_state};
use crate::room::{COUNTDOWN_TICKS, DepartureReason, InterruptReason};

use super::projection_test_support::{advance_idle, occupied, project, start, state_at};
use super::{
    MatchInterruptReason, MatchResult, RoomPhase, RoomProjectionError, validate_admission,
    validate_state_binding,
};

fn completed_round() -> (crate::room::MatchLifecycle, crate::protocol::ProjectedRound) {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    let mut state = state_at(lifecycle.tick().0);
    let spawn = state.world().arena().spawns()[0];
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(3),
        spawn,
        EntityId(1),
        2,
        DomainTick(lifecycle.tick().0),
        DomainTick(lifecycle.tick().0 + 1),
    )
    .unwrap();
    state.step(&[]).unwrap();
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::Two)))
        .unwrap();
    (lifecycle, project_state(&state.public_view()).unwrap())
}

#[test]
fn running_and_completed_bodies_bind_to_real_projected_state() {
    let mut running = occupied();
    start(&mut running);
    let projected = project_state(&state_at(running.tick().0).public_view()).unwrap();
    let room = project(&running);
    let epoch = room.round_id;
    assert_eq!(validate_state_binding(&room, epoch, &projected), Ok(()));
    assert_eq!(
        validate_admission(
            &room,
            Seat::One,
            Some(epoch),
            Some(&projected),
            Some(Counter::new(1).unwrap()),
        ),
        Ok(())
    );

    let (completed, body) = completed_round();
    let result = project(&completed);
    assert_eq!(
        body.result,
        Some(WireRoundResult::Winner { seat: Seat::Two })
    );
    assert_eq!(
        validate_state_binding(&result, result.last_round.unwrap().round_id, &body),
        Ok(())
    );
}

#[test]
fn canceled_result_uses_completed_epoch_not_current_command_epoch() {
    let (mut lifecycle, body) = completed_round();
    advance_idle(&mut lifecycle, COUNTDOWN_TICKS * 2 - 1);
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let room = project(&lifecycle);
    let completed_epoch = room.last_round.unwrap().round_id;
    assert!(completed_epoch < room.round_id);
    assert_eq!(
        validate_state_binding(&room, completed_epoch, &body),
        Ok(())
    );
    assert_eq!(
        validate_state_binding(&room, room.round_id, &body),
        Err(RoomProjectionError::InvalidStateBinding)
    );
    assert_eq!(
        validate_admission(&room, Seat::Two, Some(completed_epoch), Some(&body), None),
        Ok(())
    );
    assert_eq!(
        validate_admission(
            &room,
            Seat::Two,
            Some(completed_epoch),
            Some(&body),
            Some(Counter::new(1).unwrap()),
        ),
        Err(RoomProjectionError::InvalidAdmission)
    );
}

#[test]
fn terminal_binding_distinguishes_completed_and_unfinished_worlds() {
    let (mut completed, body) = completed_round();
    completed
        .depart_with_grace([Some(DepartureReason::Explicit), None])
        .unwrap();
    let completed_room = project(&completed);
    assert_eq!(
        validate_state_binding(
            &completed_room,
            completed_room.last_round.unwrap().round_id,
            &body,
        ),
        Ok(())
    );

    let mut unfinished = occupied();
    start(&mut unfinished);
    let unfinished_body = project_state(&state_at(unfinished.tick().0).public_view()).unwrap();
    unfinished
        .depart_with_grace([Some(DepartureReason::Explicit), None])
        .unwrap();
    let unfinished_room = project(&unfinished);
    assert_eq!(
        validate_state_binding(&unfinished_room, unfinished_room.round_id, &unfinished_body),
        Ok(())
    );
}

#[test]
fn state_binding_rejects_wrong_epoch_tick_result_and_nonphysics_phase() {
    let (lifecycle, body) = completed_round();
    let room = project(&lifecycle);
    let epoch = room.last_round.unwrap().round_id;

    assert_eq!(
        validate_state_binding(&room, Counter::new(epoch.get() + 1).unwrap(), &body),
        Err(RoomProjectionError::InvalidStateBinding)
    );
    let mut wrong_tick = body.clone();
    wrong_tick.tick = crate::protocol::Tick::new(body.tick.get() + 1).unwrap();
    assert_eq!(
        validate_state_binding(&room, epoch, &wrong_tick),
        Err(RoomProjectionError::InvalidStateBinding)
    );
    let mut wrong_result = body.clone();
    wrong_result.result = None;
    assert_eq!(
        validate_state_binding(&room, epoch, &wrong_result),
        Err(RoomProjectionError::InvalidStateBinding)
    );
    let mut wrong_rules = body;
    wrong_rules.rules_version = RulesVersion::ClassicV1;
    assert_eq!(
        validate_state_binding(&room, epoch, &wrong_rules),
        Err(RoomProjectionError::InvalidStateBinding)
    );

    let waiting = project(&occupied());
    let waiting_body = project_state(&state_at(0).public_view()).unwrap();
    assert_eq!(
        validate_state_binding(&waiting, waiting.round_id, &waiting_body),
        Err(RoomProjectionError::InvalidStateBinding)
    );
}

#[test]
fn admission_requires_connected_identity_truthful_nullability_and_context() {
    let waiting = project(&occupied());
    assert_eq!(
        validate_admission(&waiting, Seat::One, None, None, None),
        Ok(())
    );
    assert_eq!(
        validate_admission(
            &waiting,
            Seat::One,
            None,
            None,
            Some(Counter::new(1).unwrap())
        ),
        Err(RoomProjectionError::InvalidAdmission)
    );

    let mut disconnected = occupied();
    disconnected
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    assert_eq!(
        validate_admission(&project(&disconnected), Seat::One, None, None, None),
        Err(RoomProjectionError::InvalidAdmission)
    );

    let mut construction_failure = occupied();
    start(&mut construction_failure);
    construction_failure
        .interrupt(InterruptReason::InternalError)
        .unwrap();
    let room = project(&construction_failure);
    assert!(matches!(
        room.phase,
        RoomPhase::MatchResult {
            result: MatchResult::Interrupted {
                reason: MatchInterruptReason::InternalError
            },
            ..
        }
    ));
    assert_eq!(
        validate_admission(&room, Seat::One, None, None, None),
        Ok(())
    );

    let mut ordinary = room;
    let RoomPhase::MatchResult { result, .. } = &mut ordinary.phase else {
        unreachable!()
    };
    *result = MatchResult::Interrupted {
        reason: MatchInterruptReason::ServerStopped,
    };
    assert_eq!(
        validate_admission(&ordinary, Seat::One, None, None, None),
        Err(RoomProjectionError::InvalidAdmission)
    );
}
