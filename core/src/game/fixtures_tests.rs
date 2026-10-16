use super::fixed_arena;
use crate::game::{arena::Terrain, types::CellId};

fn rotated(cell: CellId) -> CellId {
    CellId::new(14 - cell.column, 12 - cell.row)
}

#[test]
fn fixed_fixture_terrain_and_blocks_have_half_turn_symmetry() {
    let arena = fixed_arena();

    for row in 0..13 {
        for column in 0..15 {
            let cell = CellId::new(column, row);
            assert_eq!(arena.terrain(cell), arena.terrain(rotated(cell)));
            assert_eq!(
                arena.blocks().contains(&cell),
                arena.blocks().contains(&rotated(cell))
            );
        }
    }
}

#[test]
fn fixed_fixture_has_classic_hard_pillars_and_clear_preview_escape_routes() {
    let arena = fixed_arena();

    assert_eq!(arena.terrain(CellId::new(2, 2)), Some(Terrain::Wall));
    assert_eq!(arena.terrain(CellId::new(12, 10)), Some(Terrain::Wall));

    for cell in [
        CellId::new(1, 1),
        CellId::new(2, 1),
        CellId::new(3, 1),
        CellId::new(4, 1),
        CellId::new(13, 11),
        CellId::new(12, 11),
        CellId::new(11, 11),
        CellId::new(10, 11),
        CellId::new(1, 2),
        CellId::new(13, 10),
    ] {
        assert_eq!(arena.terrain(cell), Some(Terrain::Floor));
        assert!(!arena.blocks().contains(&cell));
    }
}
