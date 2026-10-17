use crate::game::{DrawReason, RoundResult, SeatId};
use crate::protocol::{Seat, WireRoundResult};
use crate::room::DepartureReason;

use super::super::{CountdownKind, MatchResult, MatchWinnerReason, RoomPhase};
use super::test_support::{advance, occupied, project, score_terminal_lifecycle};

#[test]
fn projects_full_draw_three_win_and_automatic_countdown_flow() {
    let mut lifecycle = occupied();
    let waiting = project(&lifecycle);
    assert_eq!(waiting.round_number.get(), 0);
    assert_eq!(waiting.scores.one.get(), 0);
    assert_eq!(waiting.last_round, None);
    assert_eq!(waiting.phase, RoomPhase::Waiting);

    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    let countdown = project(&lifecycle);
    assert_eq!(
        countdown.phase,
        RoomPhase::Countdown {
            kind: CountdownKind::Initial,
            ends_at: crate::protocol::Tick::new(180).unwrap()
        }
    );
    advance(&mut lifecycle, 180);
    assert_eq!(project(&lifecycle).round_number.get(), 1);

    lifecycle
        .complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout)))
        .unwrap();
    let drawn = project(&lifecycle);
    assert_eq!(drawn.scores.one.get(), 0);
    assert_eq!(
        drawn.last_round.unwrap().result,
        WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::Timeout
        }
    );

    for expected_round in 2..=4 {
        advance(&mut lifecycle, 180);
        let inter = project(&lifecycle);
        assert!(matches!(
            inter.phase,
            RoomPhase::Countdown {
                kind: CountdownKind::InterRound,
                ..
            }
        ));
        assert_eq!(inter.round_number.get(), expected_round - 1);
        advance(&mut lifecycle, 180);
        assert_eq!(project(&lifecycle).round_number.get(), expected_round);
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        let view = project(&lifecycle);
        assert_eq!(u32::from(view.scores.one.get()), expected_round - 1);
    }

    let terminal = project(&lifecycle);
    assert_eq!(terminal.round_number.get(), 4);
    assert_eq!(terminal.scores.one.get(), 3);
    assert_eq!(
        terminal.phase,
        RoomPhase::MatchResult {
            ended_at: terminal.last_round.unwrap().ended_at,
            result: MatchResult::Winner {
                seat: Seat::One,
                reason: MatchWinnerReason::RoundWins
            },
            rematch_at: lifecycle
                .series()
                .hold_deadline()
                .unwrap()
                .0
                .try_into()
                .unwrap(),
        }
    );
}

#[test]
fn projects_rematch_reset_and_postfinal_departure_without_stale_consent() {
    let mut rematch = score_terminal_lifecycle();
    advance(&mut rematch, 180);
    let previous_epoch = rematch.round_attempt();
    rematch.set_rematch_vote(SeatId::One, true).unwrap();
    assert!(project(&rematch).seats[0].rematch_ready);
    rematch.set_rematch_vote(SeatId::Two, true).unwrap();
    let reset = project(&rematch);
    assert_eq!(reset.phase, RoomPhase::Waiting);
    assert_eq!(
        reset.round_id.get(),
        u32::try_from(previous_epoch + 1).unwrap()
    );
    assert_eq!(reset.round_number.get(), 0);
    assert_eq!(reset.last_round, None);
    assert!(reset.seats.iter().all(|seat| seat.occupied && !seat.ready));

    let mut departed = score_terminal_lifecycle();
    advance(&mut departed, 180);
    let final_outcome = departed.series().outcome();
    departed.set_rematch_vote(SeatId::One, true).unwrap();
    departed
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap();
    let view = project(&departed);
    assert!(!view.seats[0].occupied);
    assert!(!view.seats[0].rematch_ready);
    assert_eq!(departed.series().outcome(), final_outcome);
    assert!(matches!(view.phase, RoomPhase::MatchResult { .. }));
}
