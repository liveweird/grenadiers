use super::*;

#[test]
fn power_blast_is_captured_without_downgrading() {
    let bomb = game::BombView {
        id: game::EntityId(3),
        cell: game::CellId::new(1, 1),
        owner: game::EntityId(1),
        range: 2,
        placed_at: game::Tick(10),
        trigger: game::BombTrigger::Timed {
            detonates_at: game::Tick(190),
        },
        blast: game::BlastKind::Power,
    };
    assert_eq!(
        capture_bomb(bomb, game::Tick(10)).unwrap().blast,
        BlastKind::Power
    );
}

#[test]
fn remote_trigger_is_captured_without_inventing_a_fuse() {
    let bomb = game::BombView {
        id: game::EntityId(3),
        cell: game::CellId::new(1, 1),
        owner: game::EntityId(1),
        range: 2,
        placed_at: game::Tick(10),
        trigger: game::BombTrigger::Remote,
        blast: game::BlastKind::Standard,
    };
    assert_eq!(
        capture_bomb(bomb, game::Tick(500)).unwrap().trigger,
        BombTrigger::remote()
    );
}
