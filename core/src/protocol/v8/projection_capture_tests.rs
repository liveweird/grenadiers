use super::*;

#[test]
fn power_blast_is_captured_without_downgrading() {
    let bomb = game::Bomb::new_with_blast(
        game::EntityId(3),
        game::CellId::new(1, 1),
        game::EntityId(1),
        2,
        game::Tick(10),
        game::BombTrigger::Timed {
            detonates_at: game::Tick(190),
        },
        game::BlastKind::Power,
    )
    .unwrap();
    assert_eq!(
        capture_bomb(bomb, game::Tick(10), false).unwrap().0.blast,
        BlastKind::Power
    );
}

#[test]
fn remote_trigger_is_captured_without_inventing_a_fuse() {
    let bomb = game::Bomb::new_with_blast(
        game::EntityId(3),
        game::CellId::new(1, 1),
        game::EntityId(1),
        2,
        game::Tick(10),
        game::BombTrigger::Remote,
        game::BlastKind::Standard,
    )
    .unwrap();
    assert_eq!(
        capture_bomb(bomb, game::Tick(500), false)
            .unwrap()
            .0
            .trigger,
        BombTrigger::remote()
    );
}

#[test]
fn expanded_capture_rejects_non_legacy_forced_movement_facts() {
    let (state, report) = crate::protocol::forced_movement_test_support::bracelet_shove();
    let mut multiple = report.clone();
    let second = multiple.forced_movements[0];
    multiple.forced_movements.push(second);
    assert_eq!(
        capture_expanded(&state, &[multiple]),
        Err(ProjectionError::InvalidReports)
    );

    let mut malicious = report;
    malicious.forced_movements[0].seat = game::SeatId::Four;
    assert_eq!(
        capture_expanded(&state, &[malicious]),
        Err(ProjectionError::InvalidReports)
    );

    let (_, mut self_shove) = crate::protocol::forced_movement_test_support::bracelet_shove();
    self_shove.forced_movements[0].pusher = self_shove.forced_movements[0].seat;
    assert_eq!(
        capture_expanded(&state, &[self_shove]),
        Err(ProjectionError::InvalidReports)
    );
}
