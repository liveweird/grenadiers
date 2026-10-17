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

#[test]
fn forfeit_and_interruption_rematches_observe_the_179_180_hold_boundary() {
    for terminal in [TerminalAction::Depart, TerminalAction::Interrupt] {
        let mut series = MatchSeries::new();
        series.start_round(1, Tick(1)).unwrap();
        terminal.apply(&mut series, Tick(10)).unwrap();
        assert_eq!(series.hold_deadline(), Some(Tick(190)));

        let before = series.clone();
        assert_eq!(
            series.set_rematch_vote(SeatId::One, true, Tick(189)),
            Err(SeriesError::ResultHoldActive {
                ready_at: Tick(190)
            })
        );
        assert_eq!(series, before);
        assert_eq!(
            series.set_rematch_vote(SeatId::One, true, Tick(190)),
            Ok(RematchStatus::WaitingForOpponent)
        );
        assert_eq!(series.rematch_votes(), [true, false]);
    }
}

#[derive(Clone, Copy)]
enum TerminalAction {
    Depart,
    Interrupt,
}

impl TerminalAction {
    fn apply(
        self,
        series: &mut MatchSeries,
        tick: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        match self {
            Self::Depart => {
                series.resolve_departures([Some(DepartureReason::Explicit), None], tick)
            }
            Self::Interrupt => series.interrupt(InterruptReason::ServerStopped, tick),
        }
    }
}
