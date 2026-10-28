use grenadiers_core::{
    game::{Direction, InputAction, SeatId},
    protocol::{Counter, v14},
};

use super::{RosterAction, RosterRuntimeError, tests};

fn queued_actions(runtime: &super::RosterRuntime) -> Vec<InputAction> {
    let mut copy = runtime.clone();
    copy.inputs
        .take(copy.lifecycle.tick())
        .into_iter()
        .filter(|input| input.seat == SeatId::One)
        .map(|input| input.action)
        .collect()
}

fn token() -> v14::ResumeToken {
    v14::ResumeToken::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .unwrap()
}

#[test]
fn connected_replacement_fences_inputs_only_after_successful_delivery_commit() {
    let mut runtime = tests::runtime(3);
    tests::start(&mut runtime, 3);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Movement(Some(Direction::South)),
        )
        .unwrap();
    runtime
        .action(SeatId::One, 1, RosterAction::Bomb(7))
        .unwrap();
    let expected = vec![InputAction::Press(Direction::South), InputAction::PlaceBomb];
    assert_eq!(queued_actions(&runtime), expected);

    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    assert!(prepared.is_replacement);
    assert!(prepared.encode(Counter::new(1).unwrap(), token()).is_ok());
    drop(prepared);
    assert_eq!(queued_actions(&runtime), expected);

    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(prepared);
    assert_eq!(queued_actions(&runtime), vec![InputAction::Clear]);
    assert_eq!(
        runtime.action(SeatId::One, 1, RosterAction::Bomb(7)),
        Err(RosterRuntimeError::DuplicateAction)
    );
    assert!(
        runtime
            .action(SeatId::One, 1, RosterAction::Bomb(8))
            .is_ok()
    );
}
