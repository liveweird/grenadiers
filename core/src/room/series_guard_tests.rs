use super::*;

#[test]
fn unsupported_winner_is_rejected_before_match_series_mutation() {
    let mut series = MatchSeries::new();
    series.start_round(1, Tick(0)).unwrap();
    let before = series.clone();

    assert_eq!(
        series.complete_round(1, RoundResult::Winner(SeatId::Three), Tick(1)),
        Err(SeriesError::UnsupportedSeat(SeatId::Three))
    );
    assert_eq!(series, before);
}
