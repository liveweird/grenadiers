use super::*;
use crate::game::{DrawReason, SpeedLevel};
use crate::room::RoomSettings;

fn settings(wins: u8) -> RoomSettings {
    RoomSettings::new(SpeedLevel::Base, 256, wins, 180).unwrap()
}

fn complete(
    series: &mut MatchSeries,
    epoch: u64,
    starts_at: u64,
    result: RoundResult,
) -> Option<MatchOutcome> {
    series.start_round(epoch, Tick(starts_at)).unwrap();
    series
        .complete_round(epoch, result, Tick(starts_at + 1))
        .unwrap()
}

#[test]
fn one_win_target_finishes_immediately_and_survives_rematch_reset() {
    let mut series = MatchSeries::with_settings(settings(1));
    let outcome = complete(&mut series, 1, 0, RoundResult::Winner(SeatId::Two)).unwrap();
    assert_eq!(outcome.result(), MatchResult::ScoreWinner(SeatId::Two));
    assert_eq!(series.wins_to_finish(), 1);

    let ready_at = series.hold_deadline().unwrap();
    assert_eq!(
        series.set_rematch_vote(SeatId::One, true, ready_at),
        Ok(RematchStatus::WaitingForOpponent)
    );
    assert_eq!(
        series.set_rematch_vote(SeatId::Two, true, ready_at),
        Ok(RematchStatus::Started)
    );
    assert_eq!(series.wins_to_finish(), 1);
    assert_eq!(series.scores(), [0, 0]);

    assert!(complete(&mut series, 2, ready_at.0, RoundResult::Winner(SeatId::One)).is_some());
}

#[test]
fn nine_win_target_ignores_draws_and_waits_for_the_ninth_point() {
    let mut series = MatchSeries::with_settings(settings(9));
    assert_eq!(
        complete(
            &mut series,
            1,
            0,
            RoundResult::Draw(DrawReason::MutualElimination)
        ),
        None
    );
    assert_eq!(series.scores(), [0, 0]);

    for win in 1..=9_u64 {
        let starts_at = win * 181;
        let outcome = complete(
            &mut series,
            win + 1,
            starts_at,
            RoundResult::Winner(SeatId::One),
        );
        assert_eq!(outcome.is_some(), win == 9);
        assert_eq!(series.scores(), [u8::try_from(win).unwrap(), 0]);
    }
    assert_eq!(series.wins_to_finish(), 9);
}

#[test]
fn non_score_terminal_results_do_not_change_the_configured_target() {
    let mut series = MatchSeries::with_settings(settings(9));
    series.start_round(1, Tick(0)).unwrap();
    let outcome = series
        .resolve_departures([Some(DepartureReason::Explicit), None], Tick(1))
        .unwrap()
        .unwrap();

    assert_eq!(outcome.result(), MatchResult::ForfeitWinner(SeatId::Two));
    assert_eq!(series.wins_to_finish(), 9);
    assert_eq!(series.scores(), [0, 0]);
}
