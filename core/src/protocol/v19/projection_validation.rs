use super::{ExpandedCapture, RoomView, Seat};

pub(super) fn valid_lairs(room: &RoomView, capture: &ExpandedCapture) -> bool {
    let tuning = room.configuration.domain().base().lairs();
    if !tuning.enabled() {
        return capture.lairs.is_empty() && capture.mobs.is_empty();
    }
    let expected = crate::game::lair_layout_v1(room.configuration.domain().base().base().board());
    if capture.lairs.len() != 2 || capture.mobs.len() > crate::game::MAX_LAIRS {
        return false;
    }
    if capture
        .mobs
        .iter()
        .any(|mob| mob.frozen != capture.mobs[0].frozen)
    {
        return false;
    }
    let active_stopwatch = capture.base.world.actors.iter().any(|actor| {
        actor.view.alive
            && actor
                .view
                .effects
                .as_slice()
                .iter()
                .any(|effect| effect.kind() == super::EffectKind::Stopwatch)
    });
    if capture
        .mobs
        .first()
        .is_some_and(|mob| mob.frozen != active_stopwatch)
    {
        return false;
    }
    let (width, height) = room
        .configuration
        .domain()
        .base()
        .base()
        .board()
        .dimensions();
    if capture.mobs.iter().any(|mob| {
        let half = crate::game::types::HALF_FOOTPRINT as u32;
        u32::from(mob.position.x()) < half
            || u32::from(mob.position.x()) + half > u32::from(width) * 1_024
            || u32::from(mob.position.y()) < half
            || u32::from(mob.position.y()) + half > u32::from(height) * 1_024
    }) {
        return false;
    }
    let mut ids = capture
        .base
        .world
        .actors
        .iter()
        .map(|actor| actor.view.id.get())
        .chain(capture.base.world.bombs.iter().map(|bomb| bomb.id.get()))
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != capture.base.world.actors.len() + capture.base.world.bombs.len() {
        return false;
    }
    capture.lairs.iter().enumerate().all(|(index, lair)| {
        let terrain_index =
            usize::from(lair.cell.row()) * usize::from(width) + usize::from(lair.cell.column());
        lair.id as usize == index
            && lair.cell
                == crate::protocol::Cell::new(
                    expected[index].column as u8,
                    expected[index].row as u8,
                )
                .ok()
                .unwrap()
            && capture.base.world.arena.terrain.get(terrain_index)
                == Some(&crate::protocol::v14::Terrain::Wall)
            && match (lair.living_mobs, lair.spawn_remaining_ticks) {
                (0, Some(remaining)) => {
                    remaining <= u16::from(tuning.settings().spawn_seconds()) * 60
                }
                (1, None) => true,
                _ => false,
            }
            && usize::from(lair.living_mobs)
                == capture
                    .mobs
                    .iter()
                    .filter(|mob| mob.lair_id as usize == index)
                    .count()
    }) && capture
        .mobs
        .iter()
        .all(|mob| mob.lair_id < 2 && ids.insert(mob.id.get()))
}

pub(super) fn valid_spike_pits(room: &RoomView, capture: &ExpandedCapture) -> bool {
    use crate::game::{SpikePitClock, SpikePitPhase as DomainPhase};
    use crate::room::SpikePitConfig;

    let tuning = room.configuration.domain().spike_pits();
    if !tuning.enabled() {
        return capture.spike_pits.is_empty();
    }
    if capture.spike_pits.len() > crate::game::MAX_SPIKE_PITS
        || capture.spike_pits.windows(2).any(|pair| {
            (pair[0].cell.row(), pair[0].cell.column())
                >= (pair[1].cell.row(), pair[1].cell.column())
        })
    {
        return false;
    }
    let SpikePitConfig::Enabled(settings) = tuning.config() else {
        return false;
    };
    matches!(capture.spike_pits.len(), 0 | 2 | 4)
        && capture.spike_pits.iter().all(|pit| {
            let phase = match pit.phase {
                super::SpikePitPhase::Idle => DomainPhase::Idle,
                super::SpikePitPhase::Warning => DomainPhase::Warning,
                super::SpikePitPhase::Raised => DomainPhase::Raised,
            };
            valid_public_spike_cell(room, capture, pit.cell)
                && reflected_spike_cells(room, pit.cell).is_some_and(|reflections| {
                    reflections.into_iter().all(|reflected| {
                        capture
                            .spike_pits
                            .iter()
                            .any(|other| other.cell == reflected)
                    })
                })
                && SpikePitClock::from_observation(
                    phase,
                    pit.remaining_ticks.into(),
                    crate::game::Tick(room.tick.get()),
                    settings,
                )
                .is_ok()
        })
}

fn valid_public_spike_cell(
    room: &RoomView,
    capture: &ExpandedCapture,
    cell: crate::protocol::Cell,
) -> bool {
    use crate::protocol::v14::projection_facts::OilPhase;

    let board = room.configuration.domain().base().base().board();
    let column = u16::from(cell.column());
    let row = u16::from(cell.row());
    let terrain_index =
        usize::from(cell.row()) * usize::from(board.width()) + usize::from(cell.column());
    let spawn_reserved = [
        (1, 1),
        (board.width() - 2, 1),
        (1, board.height() - 2),
        (board.width() - 2, board.height() - 2),
    ]
    .into_iter()
    .any(|(spawn_column, spawn_row)| spawn_column.abs_diff(column) + spawn_row.abs_diff(row) <= 1);
    column > 0
        && row > 0
        && column + 1 < board.width()
        && row + 1 < board.height()
        && capture.base.world.arena.terrain.get(terrain_index) == Some(&super::Terrain::Floor)
        && !capture.base.world.arena.blocks.contains(&cell)
        && !capture.lairs.iter().any(|lair| lair.cell == cell)
        && !capture
            .base
            .world
            .oil
            .iter()
            .any(|oil| oil.cell == cell && matches!(oil.phase, OilPhase::Unlit | OilPhase::Burning))
        && !spawn_reserved
}

fn reflected_spike_cells(
    room: &RoomView,
    cell: crate::protocol::Cell,
) -> Option<[crate::protocol::Cell; 3]> {
    let board = room.configuration.domain().base().base().board();
    let reflected_column = u8::try_from(board.width() - 1 - u16::from(cell.column())).ok()?;
    let reflected_row = u8::try_from(board.height() - 1 - u16::from(cell.row())).ok()?;
    Some([
        crate::protocol::Cell::new(reflected_column, cell.row()).ok()?,
        crate::protocol::Cell::new(cell.column(), reflected_row).ok()?,
        crate::protocol::Cell::new(reflected_column, reflected_row).ok()?,
    ])
}

pub(super) fn valid_players(world: &crate::protocol::v14::projection_facts::WorldFacts) -> bool {
    let positions = world
        .actors
        .iter()
        .filter(|actor| actor.view.alive)
        .map(|actor| {
            (
                match actor.view.seat {
                    Seat::One => crate::game::SeatId::One,
                    Seat::Two => crate::game::SeatId::Two,
                    Seat::Three => crate::game::SeatId::Three,
                    Seat::Four => crate::game::SeatId::Four,
                },
                crate::game::Position {
                    x: actor.view.position.x().into(),
                    y: actor.view.position.y().into(),
                },
            )
        })
        .collect::<Vec<_>>();
    crate::game::player_collision::first_player_position_conflict(&positions).is_none()
}
