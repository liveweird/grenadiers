use std::num::NonZeroU32;

use crate::game::{CellId, EnabledItems, PickupKind, PlayerCount, RoundAssignment, Terrain};
use crate::room::BoardSize;

use super::{ArenaGenerationConfig, generate_lair_clearance_multiplayer_arena};

fn config(board: BoardSize) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count: PlayerCount::Four,
        board,
        assignment: RoundAssignment::new(NonZeroU32::new(1).unwrap()),
    }
}

#[test]
fn prototype_v5_goldens_remain_exact_for_each_board() {
    let cases = [
        (
            BoardSize::ElevenByNine,
            [0; 32],
            EnabledItems::EMPTY,
            "..##.##../.X#X#X#X./........./.X.X.X.X./........./.X#X#X#X./..##.##..",
        ),
        (
            BoardSize::ThirteenByEleven,
            [1; 32],
            EnabledItems::ALL,
            ".....#...../.X.X.X.X.X./.#.I...I.#./OX#X.X.X#XO/##I#...#I##/OX#X.X.X#XO/.#.I...I.#./.X.X.X.X.X./.....#.....",
        ),
        (
            BoardSize::FifteenByThirteen,
            [0; 32],
            EnabledItems::EMPTY,
            "..##.###.##../.XCX.X#X.XCX./...#######.../.X.X.X#X.X.X./.#.........#./.XRX.X.X.XRX./.#.........#./.X.X.X#X.X.X./...#######.../.XCX.X#X.XCX./..##.###.##..",
        ),
    ];
    for (board, seed, enabled, expected) in cases {
        let generated =
            generate_lair_clearance_multiplayer_arena(seed, enabled, config(board), true).unwrap();
        assert_eq!(signature(&generated), expected);
    }
}

fn signature(generated: &super::GeneratedArena) -> String {
    let arena = generated.arena();
    (1..arena.height() - 1)
        .map(|row| {
            (1..arena.width() - 1)
                .map(|column| {
                    let cell = CellId::new(column, row);
                    match generated.concealed_pickups().get(&cell) {
                        Some(PickupKind::BombCapacity) => 'C',
                        Some(PickupKind::BlastRange) => 'R',
                        Some(PickupKind::MovementSpeed) => 'S',
                        Some(PickupKind::OilBarrel) => 'O',
                        Some(_) => 'I',
                        None if arena.blocks().contains(&cell) => '#',
                        None if arena.terrain(cell) == Some(Terrain::Wall) => 'X',
                        None => '.',
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}
