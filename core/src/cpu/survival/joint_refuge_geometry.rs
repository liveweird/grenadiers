use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, Direction, Terrain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RefugeCandidate {
    pub movement: MovementIntent,
    pub refuge: CellId,
}

pub(super) fn refuge_candidates(
    observation: &Observation,
    origin: CellId,
    target: CellId,
    range: u16,
    blocked_away: bool,
    work: &mut WorkBudget,
) -> Option<[RefugeCandidate; 2]> {
    if blocked_away
        && directly_away_is_static(observation, origin, target, work)?
        && let Some(candidates) = blocked_away_candidates(observation, origin, target, range, work)
    {
        return Some(candidates);
    }
    legacy_refuge_candidates(observation, origin, target, range, work)
}

pub(super) fn legacy_refuge_candidates(
    observation: &Observation,
    origin: CellId,
    target: CellId,
    range: u16,
    work: &mut WorkBudget,
) -> Option<[RefugeCandidate; 2]> {
    let (away_column, away_row, lateral_column, lateral_row, direction) =
        match delta(origin, target)? {
            (0, -1) => (0, 1, 1, 0, Direction::South),
            (1, 0) => (-1, 0, 0, 1, Direction::West),
            (0, 1) => (0, -1, 1, 0, Direction::North),
            (-1, 0) => (1, 0, 0, 1, Direction::East),
            _ => return None,
        };
    let base_column = i32::from(origin.column) + away_column;
    let base_row = i32::from(origin.row) + away_row;
    let lateral = i32::from(range.max(1));
    let make = |sign: i32| {
        checked_cell(
            observation,
            base_column + lateral_column * lateral * sign,
            base_row + lateral_row * lateral * sign,
        )
        .map(|refuge| RefugeCandidate {
            movement: MovementIntent::Direction(direction),
            refuge,
        })
    };
    ordered(observation, [make(-1)?, make(1)?], work)
}

fn blocked_away_candidates(
    observation: &Observation,
    origin: CellId,
    target: CellId,
    range: u16,
    work: &mut WorkBudget,
) -> Option<[RefugeCandidate; 2]> {
    let (target_column, target_row) = delta(origin, target)?;
    let lateral = [(-target_row, target_column), (target_row, -target_column)];
    let distance = i32::from(range.max(1));
    let make = |(lateral_column, lateral_row): (i32, i32)| {
        checked_cell(
            observation,
            i32::from(origin.column) + distance * (target_column + lateral_column),
            i32::from(origin.row) + distance * (target_row + lateral_row),
        )
        .and_then(|refuge| {
            direction(lateral_column, lateral_row).map(|direction| RefugeCandidate {
                movement: MovementIntent::Direction(direction),
                refuge,
            })
        })
    };
    ordered(observation, [make(lateral[0])?, make(lateral[1])?], work)
}

fn directly_away_is_static(
    observation: &Observation,
    origin: CellId,
    target: CellId,
    work: &mut WorkBudget,
) -> Option<bool> {
    work.try_take(1).ok()?;
    let (target_column, target_row) = delta(origin, target)?;
    let column = i32::from(origin.column) - target_column;
    let row = i32::from(origin.row) - target_row;
    let Some(cell) = checked_cell(observation, column, row) else {
        return Some(true);
    };
    let index = observation.parts().arena.index(cell)?;
    let wall = observation.parts().arena.terrain.get(index) == Some(&Terrain::Wall);
    let lair = observation
        .respawn_lairs()
        .is_some_and(|lairs| lairs.lairs().iter().any(|candidate| candidate.cell == cell));
    Some(wall || lair)
}

fn ordered(
    observation: &Observation,
    mut candidates: [RefugeCandidate; 2],
    work: &mut WorkBudget,
) -> Option<[RefugeCandidate; 2]> {
    let scores = [
        public_danger_clearance(observation, candidates[0].refuge, work)?,
        public_danger_clearance(observation, candidates[1].refuge, work)?,
    ];
    if (std::cmp::Reverse(scores[1]), candidates[1].refuge)
        < (std::cmp::Reverse(scores[0]), candidates[0].refuge)
    {
        candidates.swap(0, 1);
    }
    Some(candidates)
}

fn public_danger_clearance(
    observation: &Observation,
    cell: CellId,
    work: &mut WorkBudget,
) -> Option<u32> {
    let inspected = observation
        .respawn_lairs()
        .map_or(0, |lairs| lairs.lairs().len() + lairs.mobs().len());
    work.try_take(u32::try_from(inspected).ok()?).ok()?;
    let lair_cells = observation
        .respawn_lairs()
        .into_iter()
        .flat_map(|lairs| lairs.lairs().iter().map(|lair| lair.cell));
    let mob_cells = observation
        .respawn_lairs()
        .into_iter()
        .flat_map(|lairs| lairs.mobs().iter().filter_map(|mob| mob.position.cell()));
    Some(
        lair_cells
            .chain(mob_cells)
            .map(|danger| {
                u32::from(cell.column.abs_diff(danger.column))
                    + u32::from(cell.row.abs_diff(danger.row))
            })
            .min()
            .unwrap_or(u32::MAX),
    )
}

fn delta(origin: CellId, target: CellId) -> Option<(i32, i32)> {
    let delta = (
        i32::from(target.column) - i32::from(origin.column),
        i32::from(target.row) - i32::from(origin.row),
    );
    (delta.0.abs() + delta.1.abs() == 1).then_some(delta)
}

fn checked_cell(observation: &Observation, column: i32, row: i32) -> Option<CellId> {
    (column >= 0
        && row >= 0
        && column < i32::from(observation.parts().arena.width)
        && row < i32::from(observation.parts().arena.height))
    .then(|| CellId::new(column as u16, row as u16))
}

fn direction(column: i32, row: i32) -> Option<Direction> {
    match (column, row) {
        (0, -1) => Some(Direction::North),
        (1, 0) => Some(Direction::East),
        (0, 1) => Some(Direction::South),
        (-1, 0) => Some(Direction::West),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_observation() -> Observation {
        super::super::foreign_bomb_farming_clear_fixture::captured_clear_basis().observation
    }

    #[test]
    fn blocked_away_geometry_rotates_with_the_target_cardinal() {
        let observation = clear_observation();
        let mut work = WorkBudget::new(1_000);
        let candidates = refuge_candidates(
            &observation,
            CellId::new(7, 6),
            CellId::new(8, 6),
            2,
            true,
            &mut work,
        )
        .unwrap();
        assert_eq!(
            candidates,
            [
                RefugeCandidate {
                    movement: MovementIntent::Direction(Direction::North),
                    refuge: CellId::new(9, 4),
                },
                RefugeCandidate {
                    movement: MovementIntent::Direction(Direction::South),
                    refuge: CellId::new(9, 8),
                },
            ]
        );
    }

    #[test]
    fn a_soft_block_behind_the_actor_does_not_enable_blocked_away_geometry() {
        let observation = clear_observation();
        assert!(
            observation
                .parts()
                .arena
                .blocks
                .contains(&CellId::new(3, 3))
        );
        let mut blocked_work = WorkBudget::new(1_000);
        let blocked = refuge_candidates(
            &observation,
            CellId::new(4, 3),
            CellId::new(5, 3),
            2,
            true,
            &mut blocked_work,
        );
        let mut legacy_work = WorkBudget::new(1_000);
        let legacy = legacy_refuge_candidates(
            &observation,
            CellId::new(4, 3),
            CellId::new(5, 3),
            2,
            &mut legacy_work,
        );
        assert_eq!(blocked, legacy);
        assert_eq!(blocked_work.spent(), legacy_work.spent() + 1);
    }

    #[test]
    fn a_nonadjacent_farm_target_cannot_manufacture_blocked_away_geometry() {
        let observation = clear_observation();
        let mut work = WorkBudget::new(1_000);
        assert!(
            refuge_candidates(
                &observation,
                CellId::new(11, 1),
                CellId::new(11, 3),
                2,
                true,
                &mut work,
            )
            .is_none()
        );
    }
}
