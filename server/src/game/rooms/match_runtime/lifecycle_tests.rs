use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::{
    game::{Direction, RoundAssignment, SeatId, SpeedLevel},
    protocol::v8::{MatchResult, MatchWinnerReason, RoomPhase},
    room::{DepartureReason, MatchPhase},
};

use super::MatchAction;
use super::test_support::*;

#[test]
fn three_real_round_wins_use_parity_assignments_and_fresh_state() {
    let mut runtime = fixed_runtime();
    let boundary = start_first_round(&mut runtime);
    assert_snapshot(&boundary);

    for round in 1..=3 {
        let sim = runtime.sim().unwrap();
        assert_eq!(sim.state.round_assignment().round_number().get(), round);
        assert_eq!(sim.state.running_updates(), 0);
        assert!(sim.state.world().bombs().is_empty());
        assert!(
            sim.state
                .actors()
                .iter()
                .all(|actor| actor.speed() == SpeedLevel::Base)
        );
        let assigned = sim
            .state
            .round_assignment()
            .spawns(sim.state.world().arena());
        for actor in sim.state.actors() {
            assert_eq!(
                actor.position(),
                grenadiers_core::game::Position::center(assigned[actor.seat().index()])
            );
        }

        let result = finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
        assert_snapshot(&result);
        assert_eq!(result.room.round_number.get(), round);
        assert_eq!(result.room.scores.one.get(), round as u8);
        if round == 3 {
            assert!(matches!(
                result.room.phase,
                RoomPhase::MatchResult {
                    result: MatchResult::Winner {
                        seat: grenadiers_core::protocol::Seat::One,
                        reason: MatchWinnerReason::RoundWins,
                    },
                    ..
                }
            ));
        } else {
            assert!(matches!(result.room.phase, RoomPhase::Result { .. }));
            let countdown = advance_to_inter_round(&mut runtime);
            assert_view_only(&countdown);
            assert!(runtime.sim().is_some());
            let next = tick_until_running(&mut runtime);
            assert_snapshot(&next);
        }
    }
}

#[test]
fn a_real_mutual_elimination_draw_preserves_zero_scores() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let result = finish_round_with_bombs(&mut runtime, &[SeatId::One, SeatId::Two]);

    assert!(matches!(result.room.phase, RoomPhase::Result { .. }));
    assert_eq!(result.room.round_number.get(), 1);
    assert_eq!(result.room.scores.one.get(), 0);
    assert_eq!(result.room.scores.two.get(), 0);
    assert!(matches!(
        result.room.last_round.unwrap().result,
        grenadiers_core::protocol::WireRoundResult::Draw {
            reason: grenadiers_core::protocol::DrawReason::MutualElimination
        }
    ));
}

#[test]
fn contested_round_times_out_on_exactly_10_800_running_updates() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    for expected in 1..10_800 {
        runtime.tick().unwrap();
        assert_eq!(runtime.sim().unwrap().state.running_updates(), expected);
        assert_eq!(runtime.lifecycle().phase(), MatchPhase::Running);
    }
    let result = runtime.tick().unwrap().expect("timeout snapshot");

    assert_eq!(runtime.sim().unwrap().state.running_updates(), 10_800);
    assert_eq!(
        runtime.sim().unwrap().state.tick(),
        runtime.lifecycle().tick()
    );
    assert!(matches!(result.room.phase, RoomPhase::Result { .. }));
    assert!(matches!(
        result.room.last_round.unwrap().result,
        grenadiers_core::protocol::WireRoundResult::Draw {
            reason: grenadiers_core::protocol::DrawReason::Timeout
        }
    ));
}

#[test]
fn entropy_is_requested_once_per_actual_start_and_not_for_canceled_countdown() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut runtime = generated_runtime(counting_source(Arc::clone(&calls)));
    occupy_both(&mut runtime);
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    for _ in 0..90 {
        runtime.tick().unwrap();
    }
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(false))
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(SeatId::One, epoch, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, epoch, MatchAction::Ready(true))
        .unwrap();
    tick_until_running(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        runtime.sim().unwrap().generation_record.unwrap().seed(),
        [1; 32]
    );

    finish_round_with_bombs(&mut runtime, &[SeatId::One, SeatId::Two]);
    advance_to_inter_round(&mut runtime);
    tick_until_running(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        runtime.sim().unwrap().generation_record.unwrap().seed(),
        [2; 32]
    );
}

#[test]
fn collected_real_fixture_upgrade_is_reset_in_the_next_actual_round() {
    let mut runtime = generated_runtime(pickup_fixture_source());
    start_first_round(&mut runtime);
    move_cells(&mut runtime, Direction::East, 1);
    plant(&mut runtime, 1);
    move_cells(&mut runtime, Direction::West, 1);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    assert_eq!(runtime.sim().unwrap().state.actors()[0].bomb_capacity(), 2);

    finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    advance_to_inter_round(&mut runtime);
    tick_until_running(&mut runtime);
    let actor = &runtime.sim().unwrap().state.actors()[0];
    assert_eq!(actor.bomb_capacity(), 1);
    assert_eq!(actor.bomb_range(), 2);
    assert_eq!(actor.speed(), SpeedLevel::Base);
}

#[test]
fn running_forfeit_keeps_an_unfinished_snapshot_at_the_outcome_tick() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    runtime.tick().unwrap();
    let notice = runtime
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap()
        .unwrap();
    let value = snapshot_json(&notice);

    assert_eq!(value["state"]["runningUpdates"], 1);
    assert_eq!(value["room"]["tick"], value["room"]["phase"]["endedAt"]);
    assert_eq!(value["room"]["phase"]["result"]["reason"], "forfeit");
    assert!(runtime.sim().unwrap().state.result().is_none());
}

#[test]
fn completed_hold_forfeit_keeps_the_earlier_completed_world() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let completed = finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    let completed_tick = runtime.sim().unwrap().state.tick();
    assert_eq!(
        completed.room.last_round.unwrap().ended_at.get(),
        completed_tick.0
    );
    for _ in 0..3 {
        runtime.tick().unwrap();
    }
    let notice = runtime
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap()
        .unwrap();
    let value = snapshot_json(&notice);

    assert_eq!(runtime.sim().unwrap().state.tick(), completed_tick);
    assert!(value["room"]["phase"]["endedAt"].as_u64().unwrap() > completed_tick.0);
    assert_eq!(value["state"]["actors"][0]["alive"], true);
    assert_eq!(value["state"]["actors"][1]["alive"], false);
    assert_eq!(value["room"]["phase"]["result"]["seat"], "two");
}

#[test]
fn inter_round_keeps_old_world_private_and_terminal_can_bind_completed_round() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    let countdown = advance_to_inter_round(&mut runtime);
    assert_view_only(&countdown);
    assert!(runtime.sim().is_some());

    let terminal = runtime
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap()
        .unwrap();
    assert_snapshot(&terminal);
    assert!(matches!(terminal.room.phase, RoomPhase::MatchResult { .. }));
}

#[test]
fn rematch_votes_after_the_hold_reset_match_facts_and_epoch_once() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    for round in 1..=3 {
        finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
        if round < 3 {
            advance_to_inter_round(&mut runtime);
            tick_until_running(&mut runtime);
        }
    }
    let old_epoch = runtime.lifecycle().round_attempt();
    for _ in 0..180 {
        runtime.tick().unwrap();
    }
    runtime
        .action(SeatId::One, old_epoch, MatchAction::Rematch(true))
        .unwrap();
    let reset = runtime
        .action(SeatId::Two, old_epoch, MatchAction::Rematch(true))
        .unwrap()
        .unwrap();

    assert_view_only(&reset);
    assert_eq!(phase_type(&reset), "waiting");
    assert_eq!(reset.room.round_number.get(), 0);
    assert_eq!(reset.room.scores.one.get(), 0);
    assert!(reset.room.last_round.is_none());
    assert_eq!(runtime.lifecycle().round_attempt(), old_epoch + 1);
    assert!(runtime.sim().is_none());
}

#[test]
fn round_assignment_helper_still_describes_expected_parity() {
    assert_eq!(RoundAssignment::first().priority_seat(), SeatId::One);
    assert_eq!(
        RoundAssignment::new(std::num::NonZeroU32::new(2).unwrap()).priority_seat(),
        SeatId::Two
    );
}
