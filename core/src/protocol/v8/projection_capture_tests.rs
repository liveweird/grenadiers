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
