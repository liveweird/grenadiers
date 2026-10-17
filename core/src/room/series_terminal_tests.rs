use super::*;

#[test]
fn terminal_hold_overflow_is_atomic_for_departures_and_interruptions() {
    let terminal_tick = Tick(u64::MAX - RESULT_HOLD_TICKS + 1);
    for terminal in [TerminalAction::Depart, TerminalAction::Interrupt] {
        let mut series = MatchSeries::new();
        series.start_round(1, Tick(1)).unwrap();
        let before = series.clone();
        let result = match terminal {
            TerminalAction::Depart => {
                series.resolve_departures([Some(DepartureReason::Explicit), None], terminal_tick)
            }
            TerminalAction::Interrupt => {
                series.interrupt(InterruptReason::ServerStopped, terminal_tick)
            }
        };

        assert_eq!(result, Err(SeriesError::TickOverflow));
        assert_eq!(series, before);
    }
}

#[derive(Clone, Copy)]
enum TerminalAction {
    Depart,
    Interrupt,
}
