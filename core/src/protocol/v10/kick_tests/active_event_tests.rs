use std::collections::BTreeSet;

use super::*;

#[test]
fn active_event_only_history_keeps_countdown_and_rejects_malformed_motion() {
    let mut remote = kick_state(GameBombTrigger::Remote, BTreeSet::new());
    let report = remote
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let capture = capture_expanded(&remote, &[report]).unwrap();
    assert!(capture.world.0.bombs.is_empty());
    let event_kick = capture.reports.0[0]
        .events
        .iter()
        .find_map(|event| event.kick_bomb)
        .unwrap();
    assert!(event_kick.has_been_kicked);
    assert_eq!(event_kick.motion.unwrap().countdown, 9);
    assert_eq!(
        event_kick.motion.unwrap().direction,
        crate::protocol::Direction::East
    );

    let mut malformed = capture.reports.0;
    malformed[0]
        .events
        .iter_mut()
        .find_map(|event| event.kick_bomb.as_mut())
        .unwrap()
        .motion
        .as_mut()
        .unwrap()
        .countdown = 10;
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(&capture.world, malformed,),
        Err(ProjectionError::InvalidEvent)
    );
}
