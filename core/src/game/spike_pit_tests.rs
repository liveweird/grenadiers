use std::collections::BTreeSet;

use super::*;
use crate::game::{Arena, Position, fixed_arena};
use crate::room::BoardSize;

#[test]
fn contact_cycle_uses_half_open_deadlines_without_extension() {
    let settings = SpikePitSettings::DEFAULT;
    let mut clock = SpikePitClock::new();
    assert_eq!(
        clock.update(Tick(10), true, settings),
        Ok(SpikePitPhase::Warning)
    );
    assert_eq!(clock.next_deadline(), Some(Tick(46)));
    assert_eq!(clock.remaining_ticks(Tick(10)), Ok(36));

    assert_eq!(
        clock.update(Tick(45), true, settings),
        Ok(SpikePitPhase::Warning)
    );
    assert_eq!(clock.next_deadline(), Some(Tick(46)));
    assert_eq!(clock.phase_at(Tick(46)), Ok(SpikePitPhase::Raised));
    assert_eq!(clock.remaining_ticks(Tick(46)), Ok(180));
    let interval = clock.raised_interval().unwrap();
    assert!(interval.contains(Tick(46)));
    assert!(interval.contains(Tick(225)));
    assert!(!interval.contains(Tick(226)));
    assert!(interval.overlaps(Tick(45), Tick(47)));
    assert!(!interval.overlaps(Tick(226), Tick(227)));

    assert_eq!(
        clock.update(Tick(46), true, settings),
        Ok(SpikePitPhase::Raised)
    );
    assert_eq!(clock.next_deadline(), Some(Tick(226)));
    assert_eq!(
        clock.update(Tick(225), true, settings),
        Ok(SpikePitPhase::Raised)
    );
    assert_eq!(
        clock.update(Tick(226), true, settings),
        Ok(SpikePitPhase::Idle)
    );
}

#[test]
fn retraction_tick_suppresses_continuous_contact_until_the_following_update() {
    let settings = SpikePitSettings::DEFAULT;
    let mut clock = SpikePitClock::new();
    clock.update(Tick(10), true, settings).unwrap();
    clock.update(Tick(46), true, settings).unwrap();

    assert_eq!(
        clock.update(Tick(226), true, settings),
        Ok(SpikePitPhase::Idle)
    );
    assert_eq!(
        clock.update(Tick(226), true, settings),
        Ok(SpikePitPhase::Idle)
    );
    assert_eq!(
        clock.update(Tick(226), true, settings),
        Ok(SpikePitPhase::Idle)
    );
    assert_eq!(
        clock.update(Tick(227), true, settings),
        Ok(SpikePitPhase::Warning)
    );
    assert_eq!(clock.next_deadline(), Some(Tick(263)));
}

#[test]
fn deadline_overflow_is_atomic() {
    let settings = SpikePitSettings::DEFAULT;
    let mut clock = SpikePitClock::new();
    assert_eq!(
        clock.update(Tick(u64::MAX - 35), true, settings),
        Err(SpikePitClockError::DeadlineOverflow)
    );
    assert_eq!(clock, SpikePitClock::new());

    let mut raised_deadline_overflow = SpikePitClock::new();
    assert_eq!(
        raised_deadline_overflow.update(Tick(u64::MAX - 100), true, settings),
        Err(SpikePitClockError::DeadlineOverflow)
    );
    assert_eq!(raised_deadline_overflow, SpikePitClock::new());

    let mut established_horizon = SpikePitClock::new();
    established_horizon
        .update(Tick(10), false, settings)
        .unwrap();
    assert_eq!(
        established_horizon.update(Tick(u64::MAX - 35), true, settings),
        Err(SpikePitClockError::DeadlineOverflow)
    );
    assert_eq!(
        established_horizon.update(Tick(11), true, settings),
        Ok(SpikePitPhase::Warning)
    );
}

#[test]
fn update_and_forecast_reject_ticks_behind_the_successful_horizon() {
    let settings = SpikePitSettings::DEFAULT;
    let mut clock = SpikePitClock::new();
    clock.update(Tick(10), true, settings).unwrap();
    clock.update(Tick(46), true, settings).unwrap();

    let snapshot = clock;
    assert_eq!(
        clock.update(Tick(45), true, settings),
        Err(SpikePitClockError::NonMonotonicUpdate {
            at: Tick(45),
            last_updated_at: Tick(46),
        })
    );
    assert_eq!(clock, snapshot);
    assert_eq!(
        clock.phase_at(Tick(45)),
        Err(SpikePitClockError::PastTickQuery {
            at: Tick(45),
            last_updated_at: Tick(46),
        })
    );
    assert_eq!(
        clock.remaining_ticks(Tick(45)),
        Err(SpikePitClockError::PastTickQuery {
            at: Tick(45),
            last_updated_at: Tick(46),
        })
    );

    clock.update(Tick(226), true, settings).unwrap();
    assert_eq!(
        clock.update(Tick(225), true, settings),
        Err(SpikePitClockError::NonMonotonicUpdate {
            at: Tick(225),
            last_updated_at: Tick(226),
        })
    );
    assert_eq!(clock.phase(), SpikePitPhase::Idle);
}

#[test]
fn contacts_require_positive_area_and_trace_actual_piecewise_segments() {
    let pit = CellId::new(3, 3);
    let mut tangent = Position::center(pit);
    tangent.x = 3 * 1_024 - 320;
    assert!(!position_contacts_spike_pit(tangent, pit));
    tangent.x += 1;
    assert!(position_contacts_spike_pit(tangent, pit));

    let start = Position::center(CellId::new(2, 2));
    let corner = Position::center(CellId::new(4, 2));
    let end = Position::center(CellId::new(4, 4));
    let mut trace = MovementTrace::new(start);
    trace.push(MovementSegment::new(start, corner));
    trace.push(MovementSegment::new(corner, end));
    assert!(!trace_contacts_spike_pit(&trace, pit));
    assert!(segment_contacts_spike_pit(
        MovementSegment::new(start, end),
        pit
    ));
}

#[test]
fn layout_is_clear_symmetric_bounded_and_respects_explicit_exclusions() {
    let arena = fixed_arena();
    let first = spike_pit_layout_v1(&arena, []);
    assert_eq!(first.len(), MAX_SPIKE_PITS);
    assert!(first.windows(2).all(|cells| cells[0] < cells[1]));
    for cell in &first {
        let reflected = CellId::new(
            arena.width() - 1 - cell.column,
            arena.height() - 1 - cell.row,
        );
        assert!(first.contains(&reflected));
    }
    assert!(first.iter().all(|&cell| {
        arena.terrain(cell) == Some(Terrain::Floor) && !arena.blocks().contains(&cell)
    }));

    let excluded = BTreeSet::from([first[0]]);
    let fallback = spike_pit_layout_v1(&arena, excluded);
    assert!(fallback.is_empty() || fallback.iter().all(|cell| !first.contains(cell)));
    assert!(fallback.len() <= MAX_SPIKE_PITS);
}

#[test]
fn layout_is_canonical_on_every_board_and_may_return_zero() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let arena = clear_arena(board);
        let layout = spike_pit_layout_v1(&arena, []);
        assert_eq!(layout.len(), MAX_SPIKE_PITS);
        assert!(layout.windows(2).all(|cells| cells[0] < cells[1]));
        for cell in &layout {
            assert!(layout.contains(&CellId::new(
                arena.width() - 1 - cell.column,
                arena.height() - 1 - cell.row,
            )));
        }

        let all_floor = (1..arena.height() - 1)
            .flat_map(|row| (1..arena.width() - 1).map(move |column| CellId::new(column, row)))
            .collect::<Vec<_>>();
        let reversed = all_floor.iter().rev().copied().collect::<Vec<_>>();
        assert!(spike_pit_layout_v1(&arena, all_floor).is_empty());
        assert!(spike_pit_layout_v1(&arena, reversed).is_empty());
    }
}

#[test]
fn layout_search_finds_a_clear_four_cell_orbit_beyond_the_old_candidates() {
    let board = BoardSize::FifteenByThirteen;
    let expected = [
        CellId::new(3, 4),
        CellId::new(11, 4),
        CellId::new(3, 8),
        CellId::new(11, 8),
    ];
    let arena = arena_with_only_layout_clear(board, &expected);
    assert_eq!(spike_pit_layout_v1(&arena, []), expected);
}

#[test]
fn layout_uses_a_two_cell_axis_orbit_only_when_no_four_cell_orbit_is_clear() {
    let board = BoardSize::FifteenByThirteen;
    let expected = [CellId::new(3, 6), CellId::new(11, 6)];
    let arena = arena_with_only_layout_clear(board, &expected);
    assert_eq!(spike_pit_layout_v1(&arena, []), expected);
}

#[test]
fn layout_prefers_four_cells_over_a_nearer_pair_and_never_uses_spawn_reservations() {
    let board = BoardSize::FifteenByThirteen;
    let pair = [CellId::new(6, 6), CellId::new(8, 6)];
    let orbit = [
        CellId::new(3, 4),
        CellId::new(11, 4),
        CellId::new(3, 8),
        CellId::new(11, 8),
    ];
    let clear = pair.into_iter().chain(orbit).collect::<Vec<_>>();
    let arena = arena_with_only_layout_clear(board, &clear);
    assert_eq!(spike_pit_layout_v1(&arena, []), orbit);

    let spawn_reservations_only = arena_with_only_layout_clear(board, &[]);
    assert!(spike_pit_layout_v1(&spawn_reservations_only, []).is_empty());
}

fn clear_arena(board: BoardSize) -> Arena {
    arena_with_blocks(board, BTreeSet::new())
}

fn arena_with_only_layout_clear(board: BoardSize, clear: &[CellId]) -> Arena {
    let mut blocks = (1..board.height() - 1)
        .flat_map(|row| (1..board.width() - 1).map(move |column| CellId::new(column, row)))
        .filter(|cell| !clear.contains(cell))
        .collect::<BTreeSet<_>>();
    for spawn in potential_spawns_for_board(board) {
        blocks.retain(|cell| !spawn_reserved(spawn, *cell));
    }
    arena_with_blocks(board, blocks)
}

fn arena_with_blocks(board: BoardSize, blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = vec![Terrain::Floor; board.cell_count()];
    for row in 0..board.height() {
        for column in 0..board.width() {
            if row == 0 || row == board.height() - 1 || column == 0 || column == board.width() - 1 {
                terrain[usize::from(row) * usize::from(board.width()) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new_for_board(board, terrain, blocks, potential_spawns_for_board(board)).unwrap()
}

fn potential_spawns_for_board(board: BoardSize) -> [CellId; 4] {
    [
        CellId::new(1, 1),
        CellId::new(board.width() - 2, 1),
        CellId::new(1, board.height() - 2),
        CellId::new(board.width() - 2, board.height() - 2),
    ]
}
