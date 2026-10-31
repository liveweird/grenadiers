use super::{ExpandedCapture, RoomView, Seat};

pub(super) fn valid_lairs(room: &RoomView, capture: &ExpandedCapture) -> bool {
    let tuning = room.configuration.domain().lairs();
    if !tuning.enabled() {
        return capture.lairs.is_empty() && capture.mobs.is_empty();
    }
    let expected = crate::game::lair_layout_v1(room.configuration.domain().base().board());
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
    let (width, height) = room.configuration.domain().base().board().dimensions();
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
