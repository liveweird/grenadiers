use grenadiers_core::game::{Direction, SeatId};

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
fn exhausted_revision_fails_before_advancing_physics() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let tick = runtime.sim().unwrap().state.tick();
    runtime.set_revision(grenadiers_core::protocol::MAX_COUNTER);

    assert_eq!(runtime.tick(), Err(MatchRuntimeError::CapacityReached));
    assert_eq!(runtime.sim().unwrap().state.tick(), tick);
    assert_eq!(runtime.lifecycle().tick(), tick);
}
