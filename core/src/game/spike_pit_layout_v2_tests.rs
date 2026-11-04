use std::collections::BTreeSet;

use crate::game::{
    ArenaConstructionProfile, ArenaGenerationConfig, EnabledItems, PlayerCount, RoundAssignment,
    lair_layout_v1,
};
use crate::room::{BoardSize, BombCapacityDropMultiplier};

use super::{spike_pit_layout_v1, spike_pit_layout_v2};

#[test]
fn v2_matrix_preserves_reflections_lair_rings_and_connected_exits() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            for multiplier in [1, 5, 10] {
                let generated = ArenaConstructionProfile::WeightedLairClearanceV7
                    .generate(
                        [board.width() as u8 + count.get() as u8 + multiplier as u8; 32],
                        EnabledItems::ALL,
                        ArenaGenerationConfig {
                            player_count: count,
                            board,
                            assignment: RoundAssignment::first(),
                        },
                        BombCapacityDropMultiplier::new(multiplier).unwrap(),
                        true,
                    )
                    .unwrap();
                let arena = generated.arena();
                let lairs = lair_layout_v1(board);
                let sites = spike_pit_layout_v2(arena, [], lairs);
                assert!(matches!(sites.len(), 0 | 2 | 4));
                assert_eq!(
                    sites.iter().copied().collect::<BTreeSet<_>>(),
                    sites
                        .iter()
                        .flat_map(|cell| reflections(arena, *cell))
                        .collect()
                );
                let ring = crate::game::layout_clearance::lair_floor_ring(arena, lairs);
                assert!(sites.iter().all(|cell| !ring.contains(cell)));
                let mut blocked = arena.blocks().clone();
                blocked.extend(sites);
                assert!(crate::game::layout_clearance::exits_share_corner_network(
                    arena, &blocked, &lairs,
                ));
            }
        }
    }
}

#[test]
fn no_active_lairs_preserve_published_v1_layout() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let generated = ArenaConstructionProfile::WeightedLairClearanceV7
            .generate(
                [17; 32],
                EnabledItems::EMPTY,
                ArenaGenerationConfig {
                    player_count: PlayerCount::Four,
                    board,
                    assignment: RoundAssignment::first(),
                },
                BombCapacityDropMultiplier::DEFAULT,
                false,
            )
            .unwrap();
        assert_eq!(
            spike_pit_layout_v2(generated.arena(), [], []),
            spike_pit_layout_v1(generated.arena(), [])
        );
    }
}

fn reflections(arena: &crate::game::Arena, cell: crate::game::CellId) -> [crate::game::CellId; 4] {
    [
        cell,
        crate::game::CellId::new(arena.width() - 1 - cell.column, cell.row),
        crate::game::CellId::new(cell.column, arena.height() - 1 - cell.row),
        crate::game::CellId::new(
            arena.width() - 1 - cell.column,
            arena.height() - 1 - cell.row,
        ),
    ]
}
