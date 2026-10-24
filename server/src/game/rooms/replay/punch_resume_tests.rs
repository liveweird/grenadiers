use grenadiers_core::{
    game::{CellId, Direction, EnabledItems, Equipment, SeatId},
    protocol::Counter,
    room::{DepartureReason, reconnect::ReservationState},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, MatchRuntimeError, collect_generated_pickup, move_cells, plant,
    start_optional_runtime_with_fuse, wait_clear,
};

fn public_bomb(frame: &str, id: u32) -> serde_json::Value {
    for private in [
        "lastSuccessfulPunchTick",
        "last_successful_punch_tick",
        "hasBeenPunched",
        "bombGrace",
        "motion",
    ] {
        assert!(
            !frame.contains(private),
            "private field {private} was published"
        );
    }
    serde_json::from_str::<serde_json::Value>(frame).unwrap()["state"]["bombs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|bomb| bomb["id"] == id)
        .cloned()
        .expect("punched bomb is present in resumed public state")
}

#[test]
fn successful_punch_survives_replacement_restore_and_cooldown_without_replaying_actions() {
    let mut runtime = start_optional_runtime_with_fuse(EnabledItems::PUNCH, 31, 180);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .equipment(),
        Some(Equipment::Punch)
    );

    // Open the seed-31 route used by the core Punch proof.
    plant(&mut runtime, 2);
    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 3);
    plant(&mut runtime, 3);
    move_cells(&mut runtime, Direction::West, 1);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 1);
    plant(&mut runtime, 4);
    move_cells(&mut runtime, Direction::East, 1);

    let epoch = runtime.lifecycle().round_attempt();
    let before = runtime.sim().unwrap().state.world().bombs();
    let first = *before
        .values()
        .find(|bomb| bomb.cell() == CellId::new(4, 1))
        .unwrap();
    runtime
        .action(
            SeatId::One,
            epoch,
            MatchAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    runtime.tick().unwrap();
    runtime
        .action(SeatId::One, epoch, MatchAction::Movement(None))
        .unwrap();
    runtime
        .action(SeatId::One, epoch, MatchAction::Equipment(5))
        .unwrap();
    runtime.tick().unwrap();

    let punched_at = runtime.sim().unwrap().state.tick();
    let punched = runtime.sim().unwrap().state.world().bombs()[&first.id()];
    assert_eq!(punched.id(), first.id());
    assert_eq!(punched.cell(), CellId::new(1, 1));
    assert_eq!(punched.owner(), first.owner());
    assert_eq!(punched.range(), first.range());
    assert_eq!(punched.detonates_at(), first.detonates_at());
    assert!(punched.has_been_punched());
    assert!(!punched.has_been_kicked());
    assert_eq!(punched.motion(), None);
    let trusted = format!(
        "{:?}",
        runtime.replay_inspection().simulation.unwrap().state
    );
    assert!(trusted.contains(&format!(
        "last_successful_punch_tick: Some(Tick({}))",
        punched_at.0
    )));

    // Queue the action only after the cooldown expires, so replacement fencing—not the
    // cooldown—is what prevents it from executing.
    while runtime.sim().unwrap().state.tick().0 < punched_at.0 + 12 {
        runtime.tick().unwrap();
    }
    move_cells(&mut runtime, Direction::West, 3);
    runtime
        .action(SeatId::One, epoch, MatchAction::Equipment(6))
        .unwrap();
    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(
        replacement.last_bomb_action_id,
        Some(Counter::new(6).unwrap())
    );
    let published = public_bomb(
        &replacement.notice.frame(SeatId::One).unwrap(),
        first.id().0,
    );
    assert_eq!(
        published["cell"],
        serde_json::json!({"column": 1, "row": 1})
    );
    assert_eq!(published["owner"], first.owner().0);
    assert_eq!(published["range"], first.range());
    assert_eq!(published["placedAt"], first.placed_at().0);
    assert_eq!(published["trigger"]["type"], "timed");
    runtime.commit_admission(replacement);
    runtime.tick().unwrap();
    let after_replacement = runtime.sim().unwrap().state.world().bombs()[&first.id()];
    assert_eq!(after_replacement.cell(), CellId::new(1, 1));
    assert!(after_replacement.has_been_punched());

    runtime
        .action(SeatId::One, epoch, MatchAction::Equipment(7))
        .unwrap();
    runtime.tick().unwrap();
    let replacement_punch_tick = runtime.sim().unwrap().state.tick();
    let replacement_punch = runtime.sim().unwrap().state.world().bombs()[&first.id()];
    assert_ne!(replacement_punch.cell(), CellId::new(1, 1));
    assert_eq!(replacement_punch.id(), first.id());
    assert_eq!(replacement_punch.owner(), first.owner());
    assert_eq!(replacement_punch.range(), first.range());
    assert_eq!(replacement_punch.detonates_at(), first.detonates_at());
    assert!(replacement_punch.has_been_punched());
    assert_eq!(replacement_punch.motion(), None);
    let replacement_punch_cell = replacement_punch.cell();

    runtime.stop_seat_inputs(SeatId::One);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    assert!(matches!(
        runtime.lifecycle().reconnect_grace().seat(SeatId::One),
        ReservationState::Disconnected { .. }
    ));
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(
        restoration.last_bomb_action_id,
        Some(Counter::new(7).unwrap())
    );
    public_bomb(
        &restoration.notice.frame(SeatId::One).unwrap(),
        first.id().0,
    );
    runtime.commit_admission(restoration);
    let restored = format!(
        "{:?}",
        runtime.replay_inspection().simulation.unwrap().state
    );
    assert!(restored.contains(&format!(
        "last_successful_punch_tick: Some(Tick({}))",
        replacement_punch_tick.0
    )));
    assert!(runtime.sim().unwrap().state.world().bombs()[&first.id()].has_been_punched());

    assert_eq!(
        runtime.action(SeatId::One, epoch, MatchAction::Equipment(7)),
        Err(MatchRuntimeError::DuplicateAction)
    );
    // This fresh action targets an empty cell; private cooldown retention is asserted above.
    runtime
        .action(SeatId::One, epoch, MatchAction::Equipment(8))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(
        runtime.sim().unwrap().state.world().bombs()[&first.id()].cell(),
        replacement_punch_cell
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}
