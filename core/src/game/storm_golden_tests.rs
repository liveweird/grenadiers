use super::storm_test_support::state_with;
use crate::game::{CellId, Direction, EnabledItems, PlayerCount, Position};
use crate::room::BoardSize;

#[test]
fn stream_one_multi_leg_golden_reaches_and_reflects_at_the_west_boundary() {
    // Independent OpenSSL ChaCha20 vector: zero key, IV counter=0/stream=1. The first LE word is
    // 3_604_955_119 (West); word 50 is 1_839_243_895 (West, reflected to East at this boundary).
    let mut game = state_with(
        BoardSize::ElevenByNine,
        PlayerCount::Two,
        EnabledItems::EMPTY,
        [0; 32],
    );
    let checkpoints = [
        (8, CellId::new(6, 5), Direction::North),
        (16, CellId::new(5, 4), Direction::West),
        (24, CellId::new(4, 3), Direction::North),
        (32, CellId::new(3, 6), Direction::South),
        (40, CellId::new(4, 7), Direction::South),
        (48, CellId::new(1, 6), Direction::West),
        (49, CellId::new(1, 5), Direction::North),
    ];
    let mut completed = 0;
    for (legs, cell, direction) in checkpoints {
        for _ in completed * 128..legs * 128 {
            game.step(&[]).unwrap();
        }
        completed = legs;
        let cloud = game.storm_cloud().unwrap();
        assert_eq!(cloud.position, Position::center(cell));
        assert_eq!(cloud.direction, direction);
        assert_eq!(cloud.leg_remaining_ticks, 0);
    }
    game.step(&[]).unwrap();
    let reflected = game.storm_cloud().unwrap();
    assert_eq!(reflected.direction, Direction::East);
    assert_eq!(
        reflected.position.x,
        Position::center(CellId::new(1, 5)).x + 8
    );
    assert_eq!(reflected.leg_remaining_ticks, 127);
}
