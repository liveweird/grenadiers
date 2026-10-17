use grenadiers_core::{
    game::{Direction, RoundResult, SeatId},
    protocol::{Counter, MAX_COUNTER},
    room::{DepartureReason, MatchPhase, reconnect::ReservationState},
};

use super::test_support::*;
use super::{MatchAction, MatchRuntimeError};

#[test]
fn stale_epoch_and_invalid_phase_do_not_consume_bomb_high_water() {
    let mut runtime = fixed_runtime();
    occupy_both(&mut runtime);
    assert_eq!(
        runtime.action(SeatId::One, 1, MatchAction::Bomb(7)),
        Err(MatchRuntimeError::InvalidPhase)
    );
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    tick_until_running(&mut runtime);

    assert_eq!(
        runtime.action(SeatId::One, 0, MatchAction::Bomb(7)),
        Err(MatchRuntimeError::StaleEpoch)
    );
    assert_eq!(
        runtime.action(SeatId::One, 1, MatchAction::Bomb(MAX_COUNTER + 1)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(7))
        .unwrap();
    assert_eq!(
        runtime.action(SeatId::One, 1, MatchAction::Bomb(7)),
        Err(MatchRuntimeError::DuplicateAction)
    );
}

#[test]
fn movement_ttl_stops_the_actual_actor_and_clear_is_immediate() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let start = runtime.sim().unwrap().state.actors()[0].position();
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    for _ in 0..16 {
        runtime.tick().unwrap();
    }
    let stopped = runtime.sim().unwrap().state.actors()[0].position();
    assert!(stopped.x > start.x);
    runtime.tick().unwrap();
    assert_eq!(runtime.sim().unwrap().state.actors()[0].position(), stopped);

    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::West)))
        .unwrap();
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(None))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(runtime.sim().unwrap().state.actors()[0].position(), stopped);
}

#[test]
fn bomb_deduplication_resets_for_a_fresh_actual_round() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(50))
        .unwrap();
    assert_eq!(
        runtime.action(SeatId::One, 1, MatchAction::Bomb(50)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    advance_to_inter_round(&mut runtime);
    tick_until_running(&mut runtime);
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(SeatId::One, epoch, MatchAction::Bomb(50))
        .unwrap();
}

#[test]
fn canceled_connected_resume_preflight_does_not_fence_accepted_inputs() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let start = runtime.sim().unwrap().state.actors()[0].position();
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(9))
        .unwrap();

    drop(runtime.prepare_resume(SeatId::One).unwrap());
    runtime.tick().unwrap();

    assert!(runtime.sim().unwrap().state.actors()[0].position().x > start.x);
    assert_eq!(runtime.sim().unwrap().state.live_bomb_count(SeatId::One), 1);
}

#[test]
fn connected_replacement_drops_queued_actions_but_preserves_the_baseline() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(5))
        .unwrap();
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(6))
        .unwrap();

    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(prepared.last_bomb_action_id, Some(Counter::new(6).unwrap()));
    runtime.commit_admission(prepared);
    runtime.tick().unwrap();

    assert_eq!(runtime.sim().unwrap().state.live_bomb_count(SeatId::One), 0);
    assert_eq!(
        runtime.action(SeatId::One, 1, MatchAction::Bomb(6)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(7))
        .unwrap();
}

#[test]
fn failed_gameplay_placement_keeps_the_admitted_action_baseline() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(1))
        .unwrap();
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(2))
        .unwrap();
    runtime.tick().unwrap();

    assert_eq!(runtime.sim().unwrap().state.live_bomb_count(SeatId::One), 1);
    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(prepared.last_bomb_action_id, Some(Counter::new(2).unwrap()));
}

#[test]
fn exhausted_revision_fails_before_advancing_physics() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let tick = runtime.sim().unwrap().state.tick();
    runtime.set_revision(grenadiers_core::protocol::MAX_COUNTER);

    assert_eq!(runtime.tick(), Err(MatchRuntimeError::CapacityReached));
    assert_eq!(runtime.sim().unwrap().state.tick(), tick);
    assert_eq!(runtime.lifecycle().tick(), tick);
}

#[test]
fn transport_loss_fences_input_but_preserves_fuse_damage_and_grace() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(
            SeatId::One,
            epoch,
            MatchAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    runtime
        .action(SeatId::One, epoch, MatchAction::Bomb(1))
        .unwrap();
    runtime.tick().unwrap();
    let sim = runtime.sim().unwrap();
    let planted = *sim.state.world().bombs().values().next().unwrap();
    let stopped_at = sim.state.actors()[0].position();
    assert!(planted.detonates_at().unwrap().0 > sim.state.tick().0);

    runtime
        .action(
            SeatId::One,
            epoch,
            MatchAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    runtime
        .action(SeatId::One, epoch, MatchAction::Bomb(2))
        .unwrap();
    runtime.stop_seat_inputs(SeatId::One);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();

    let disconnected_at = runtime.lifecycle().tick();
    let ReservationState::Disconnected { expires_at } =
        runtime.lifecycle().reconnect_grace().seat(SeatId::One)
    else {
        panic!("transport loss did not reserve the seat");
    };
    assert_eq!(expires_at.0, disconnected_at.0 + 600);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0].position(),
        stopped_at
    );
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    assert_eq!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .get(&planted.id()),
        Some(&planted)
    );

    while runtime.lifecycle().tick().0 + 1 < planted.detonates_at().unwrap().0 {
        runtime.tick().unwrap();
        assert_eq!(
            runtime.sim().unwrap().state.actors()[0].position(),
            stopped_at
        );
        assert!(runtime.sim().unwrap().state.actors()[0].alive());
        assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
        assert_eq!(runtime.lifecycle().phase(), MatchPhase::Running);
    }
    runtime.tick().unwrap();

    let sim = runtime.sim().unwrap();
    assert_eq!(sim.state.tick(), planted.detonates_at().unwrap());
    assert!(sim.state.world().bombs().is_empty());
    assert!(sim.state.world().hazards().contains_key(&planted.cell()));
    assert!(!sim.state.actors()[0].alive());
    assert!(sim.state.actors()[1].alive());
    assert_eq!(sim.state.result(), Some(RoundResult::Winner(SeatId::Two)));
    assert_eq!(runtime.lifecycle().series().scores(), [0, 1]);
    assert!(matches!(
        runtime.lifecycle().phase(),
        MatchPhase::Result { .. }
    ));
    assert_eq!(
        runtime.lifecycle().reconnect_grace().seat(SeatId::One),
        ReservationState::Disconnected { expires_at }
    );
    assert!(runtime.lifecycle().tick().0 < expires_at.0);
}
