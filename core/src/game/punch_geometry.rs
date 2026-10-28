use super::arena::Terrain;
use super::{CellId, Direction};
use crate::room::BoardSize;

pub(crate) fn adjacent_cell(cell: CellId, direction: Direction) -> Option<CellId> {
    match direction {
        Direction::North => cell
            .row
            .checked_sub(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::East => cell
            .column
            .checked_add(1)
            .map(|column| CellId::new(column, cell.row)),
        Direction::South => cell
            .row
            .checked_add(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::West => cell
            .column
            .checked_sub(1)
            .map(|column| CellId::new(column, cell.row)),
    }
}

pub(crate) fn landing_candidates(
    source: CellId,
    direction: Direction,
) -> impl Iterator<Item = CellId> {
    landing_candidates_for_board(source, direction, BoardSize::FifteenByThirteen)
}

pub(crate) fn landing_candidates_for_board(
    source: CellId,
    direction: Direction,
    board: BoardSize,
) -> impl Iterator<Item = CellId> {
    let (width, height) = board.dimensions();
    let (coordinate, length, delta) = match direction {
        Direction::East => (source.column, width - 2, 3_i32),
        Direction::West => (source.column, width - 2, -3_i32),
        Direction::South => (source.row, height - 2, 3_i32),
        Direction::North => (source.row, height - 2, -3_i32),
    };
    (1..=length)
        .map(move |ordinal| {
            let wrapped = 1
                + (i32::from(coordinate) - 1 + delta * i32::from(ordinal))
                    .rem_euclid(i32::from(length));
            let wrapped = u16::try_from(wrapped).expect("wrapped interior coordinate fits u16");
            match direction {
                Direction::East | Direction::West => CellId::new(wrapped, source.row),
                Direction::North | Direction::South => CellId::new(source.column, wrapped),
            }
        })
        .take_while(move |candidate| *candidate != source)
}

pub(crate) const fn landing_is_open(
    terrain: Option<Terrain>,
    has_block: bool,
    has_unlit_oil: bool,
    has_other_bomb: bool,
    overlaps_actor: bool,
) -> bool {
    matches!(terrain, Some(Terrain::Floor))
        && !has_block
        && !has_unlit_oil
        && !has_other_bomb
        && !overlaps_actor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_wrap_in_authoritative_stride_order_without_returning_source() {
        let source = CellId::new(13, 5);
        let east = landing_candidates(source, Direction::East).collect::<Vec<_>>();
        assert_eq!(
            east[..4],
            [
                CellId::new(3, 5),
                CellId::new(6, 5),
                CellId::new(9, 5),
                CellId::new(12, 5)
            ]
        );
        assert!(!east.contains(&source));

        let north = landing_candidates(CellId::new(4, 1), Direction::North).collect::<Vec<_>>();
        assert_eq!(
            north[..3],
            [CellId::new(4, 9), CellId::new(4, 6), CellId::new(4, 3)]
        );
    }

    #[test]
    fn nine_cell_interior_preserves_two_other_stride_candidates() {
        let candidates = landing_candidates_for_board(
            CellId::new(1, 5),
            Direction::East,
            BoardSize::ElevenByNine,
        )
        .collect::<Vec<_>>();

        assert_eq!(candidates, [CellId::new(4, 5), CellId::new(7, 5)]);
    }

    #[test]
    fn static_eligibility_rejects_each_authoritative_blocker() {
        assert!(landing_is_open(
            Some(Terrain::Floor),
            false,
            false,
            false,
            false
        ));
        assert!(!landing_is_open(
            Some(Terrain::Wall),
            false,
            false,
            false,
            false
        ));
        for blockers in [
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (false, false, false, true),
        ] {
            assert!(!landing_is_open(
                Some(Terrain::Floor),
                blockers.0,
                blockers.1,
                blockers.2,
                blockers.3,
            ));
        }
    }
}
