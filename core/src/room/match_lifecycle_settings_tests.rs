use super::*;
use crate::game::SpeedLevel;
use crate::room::series::RESULT_HOLD_TICKS;

fn custom(wins: u8) -> RoomSettings {
    RoomSettings::new(SpeedLevel::Three, 64, wins, 90).unwrap()
}

fn occupied(settings: RoomSettings) -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::with_settings(settings);
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle
}

fn advance_idle(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

#[test]
fn custom_construction_and_lobby_updates_bind_series_and_clear_readiness() {
    let initial = custom(9);
    let replacement = RoomSettings::new(SpeedLevel::Four, 192, 1, 300).unwrap();
    let mut lifecycle = occupied(initial);
    lifecycle.set_ready(SeatId::One, true).unwrap();

    assert_eq!(lifecycle.settings(), initial);
    assert_eq!(lifecycle.series().wins_to_finish(), 9);
    assert_eq!(
        lifecycle.set_settings(replacement),
        Ok(MatchChange::Changed)
    );
    assert_eq!(lifecycle.settings(), replacement);
    assert_eq!(lifecycle.series().wins_to_finish(), 1);
    assert!(lifecycle.seats().iter().all(|seat| !seat.ready()));
    assert_eq!(
        lifecycle.set_settings(replacement),
        Ok(MatchChange::Unchanged)
    );
}

#[test]
fn settings_changes_outside_the_waiting_lobby_are_rejected_atomically() {
    let initial = custom(3);
    let replacement = custom(1);
    let mut lifecycle = occupied(initial);
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();

    let before = lifecycle.clone();
    assert_eq!(
        lifecycle.set_settings(replacement),
        Err(MatchError::InvalidPhase)
    );
    assert_eq!(lifecycle, before);

    advance_idle(&mut lifecycle, COUNTDOWN_TICKS);
    let before = lifecycle.clone();
    assert_eq!(
        lifecycle.set_settings(replacement),
        Err(MatchError::InvalidPhase)
    );
    assert_eq!(lifecycle, before);
}

#[test]
fn configured_target_is_retained_after_a_completed_match_and_rematch() {
    let settings = custom(1);
    let mut lifecycle = occupied(settings);
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    advance_idle(&mut lifecycle, COUNTDOWN_TICKS);
    assert!(matches!(
        lifecycle.complete_running_tick(Some(RoundResult::Winner(SeatId::One))),
        Ok(MatchChange::MatchFinished(_))
    ));

    advance_idle(&mut lifecycle, RESULT_HOLD_TICKS);
    lifecycle.set_rematch_vote(SeatId::One, true).unwrap();
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::Two, true),
        Ok(MatchChange::RematchStarted)
    );
    assert_eq!(lifecycle.settings(), settings);
    assert_eq!(lifecycle.series().wins_to_finish(), 1);
}
