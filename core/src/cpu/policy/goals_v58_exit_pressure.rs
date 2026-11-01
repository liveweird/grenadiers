use crate::cpu::policy::goals::v24::pressure::{PressureOrigin, RivalPressure};
use crate::cpu::survival::ObservedBlastGeometry;
use crate::cpu::{BombKnowledge, Observation, ObservedActor, WorkBudget};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{BlastKind, CellId, Direction, Equipment, PickupKind, Position, Terrain};

const MAX_ORIGIN_PROBES: usize = 32;

pub(super) fn promote(
    observation: &Observation,
    kind: BlastKind,
    range: u16,
    geometry: &ObservedBlastGeometry<'_>,
    rivals: &mut [RivalPressure],
    work: &mut WorkBudget,
) {
    for rival in rivals {
        let allowance = rival.unused_slot.min(work.remaining());
        let mut child = WorkBudget::new(allowance);
        try_promote(observation, kind, range, geometry, rival, &mut child);
        let _ = work.try_take(child.spent());
    }
}

fn try_promote(
    observation: &Observation,
    kind: BlastKind,
    range: u16,
    geometry: &ObservedBlastGeometry<'_>,
    rival: &mut RivalPressure,
    work: &mut WorkBudget,
) {
    if !rival.complete || rival.probes >= MAX_ORIGIN_PROBES {
        return;
    }
    let Some(baseline) = rival.baseline else {
        return;
    };
    let Some((target, exits)) = supported_target(observation, rival, work) else {
        return;
    };
    let Some(mut best_coverage) = coverage(geometry, baseline.approach, range, kind, &exits, work)
    else {
        return;
    };
    let mut best = baseline;
    for (offset, origin) in rival.origins.iter().skip(rival.next_probe).enumerate() {
        let probes = rival.probes.saturating_add(offset);
        if probes >= MAX_ORIGIN_PROBES || origin.distance != baseline.distance {
            break;
        }
        let Some(threatens) =
            geometry.threatens_position(origin.approach, range, kind, target, work)
        else {
            return;
        };
        if !threatens {
            continue;
        }
        let Some(candidate_coverage) =
            coverage(geometry, origin.approach, range, kind, &exits, work)
        else {
            return;
        };
        if candidate_coverage > best_coverage {
            best = candidate_from(origin, baseline);
            best_coverage = candidate_coverage;
        }
    }
    rival.baseline = Some(best);
}

fn supported_target(
    observation: &Observation,
    rival: &RivalPressure,
    work: &mut WorkBudget,
) -> Option<(Position, Vec<Position>)> {
    let parts = observation.parts();
    if !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        || !parts.flames.is_empty()
        || !parts.oil.is_empty()
    {
        return None;
    }
    let mut target = None;
    for actor in &parts.actors {
        work.try_take(1).ok()?;
        match actor {
            ObservedActor::Hidden { alive: true, .. } => return None,
            ObservedActor::Visible(actor) if actor.seat == rival.seat => {
                if !actor.alive
                    || actor.voluntary_actions_frozen
                    || actor.position.cell().is_none()
                    || actor.position != Position::center(actor.position.cell()?)
                    || actor.position != rival.position
                {
                    return None;
                }
                target = Some(actor);
            }
            _ => {}
        }
    }
    let target = target?;
    for pickup in &parts.pickups {
        work.try_take(1).ok()?;
        if pickup.kind == PickupKind::Teleporter {
            return None;
        }
    }
    let target_cell = target.position.cell()?;
    let wall_pass = target.equipment == Some(Equipment::WallPass)
        && parts.rules.settings.enabled_items().contains_wall_pass();
    let mut exits = Vec::new();
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let Some(cell) = adjacent(
            target_cell,
            direction,
            parts.arena.width,
            parts.arena.height,
        ) else {
            continue;
        };
        work.try_take(1).ok()?;
        let index = parts.arena.index(cell)?;
        if parts.arena.terrain.get(index) != Some(&Terrain::Floor) {
            continue;
        }
        let soft = charged_contains(&parts.arena.blocks, cell, work)?;
        if soft && !wall_pass {
            continue;
        }
        if charged_pickup(parts.pickups.as_slice(), cell, work)?
            || charged_pit(observation, cell, work)?
            || charged_mob(observation, cell, work)?
            || charged_actor(parts.actors.as_slice(), cell, work)?
        {
            return None;
        }
        exits.push(Position::center(cell));
    }
    Some((target.position, exits))
}

fn coverage(
    geometry: &ObservedBlastGeometry<'_>,
    origin: CellId,
    range: u16,
    kind: BlastKind,
    exits: &[Position],
    work: &mut WorkBudget,
) -> Option<u8> {
    exits.iter().try_fold(0_u8, |covered, &exit| {
        geometry
            .threatens_position(origin, range, kind, exit, work)
            .map(|hit| covered + u8::from(hit))
    })
}

fn candidate_from(origin: &PressureOrigin, baseline: super::Candidate) -> super::Candidate {
    super::Candidate {
        approach: origin.approach,
        distance: origin.distance,
        first_step: origin.first_step,
        ..baseline
    }
}

fn adjacent(cell: CellId, direction: Direction, width: u16, height: u16) -> Option<CellId> {
    let (column, row) = match direction {
        Direction::North => (i32::from(cell.column), i32::from(cell.row) - 1),
        Direction::East => (i32::from(cell.column) + 1, i32::from(cell.row)),
        Direction::South => (i32::from(cell.column), i32::from(cell.row) + 1),
        Direction::West => (i32::from(cell.column) - 1, i32::from(cell.row)),
    };
    (column >= 0 && row >= 0 && column < i32::from(width) && row < i32::from(height))
        .then(|| CellId::new(column as u16, row as u16))
}

fn charged_contains(cells: &[CellId], cell: CellId, work: &mut WorkBudget) -> Option<bool> {
    work.try_take(u32::try_from(cells.len()).ok()?).ok()?;
    Some(cells.contains(&cell))
}

fn charged_pickup(
    pickups: &[crate::cpu::ObservedPickup],
    cell: CellId,
    work: &mut WorkBudget,
) -> Option<bool> {
    work.try_take(u32::try_from(pickups.len()).ok()?).ok()?;
    Some(pickups.iter().any(|pickup| pickup.cell == cell))
}

fn charged_pit(observation: &Observation, cell: CellId, work: &mut WorkBudget) -> Option<bool> {
    let pits = observation.spike_pits().map_or(&[][..], |pits| pits.pits());
    work.try_take(u32::try_from(pits.len()).ok()?).ok()?;
    Some(pits.iter().any(|pit| pit.cell == cell))
}

fn charged_mob(observation: &Observation, cell: CellId, work: &mut WorkBudget) -> Option<bool> {
    let mobs = observation
        .respawn_lairs()
        .map_or(&[][..], |lairs| lairs.mobs());
    work.try_take(u32::try_from(mobs.len()).ok()?).ok()?;
    Some(
        mobs.iter()
            .any(|mob| position_overlaps_cell(mob.position, cell)),
    )
}

fn charged_actor(actors: &[ObservedActor], cell: CellId, work: &mut WorkBudget) -> Option<bool> {
    work.try_take(u32::try_from(actors.len()).ok()?).ok()?;
    Some(actors.iter().any(|actor| {
        matches!(actor, ObservedActor::Visible(actor) if position_overlaps_cell(actor.position, cell))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget};
    use crate::cpu::survival::exit_pressure_fixture::{
        NORTH_ORIGIN, RIVAL, RIVAL_CELL, START, WEST_ORIGIN, collected_basis_at,
    };
    use crate::game::SeatId;

    fn baseline() -> super::super::Candidate {
        super::super::Candidate {
            objective: StrategicObjectiveKind::Attack,
            target: StrategicTarget::Opponent { seat: RIVAL },
            approach: NORTH_ORIGIN,
            score: 4,
            distance: 2,
            first_step: CellId::new(6, 5),
        }
    }

    fn rival(origins: Vec<PressureOrigin>, unused_slot: u32) -> RivalPressure {
        RivalPressure {
            seat: SeatId::One,
            position: Position::center(RIVAL_CELL),
            baseline: Some(baseline()),
            origins,
            next_probe: 1,
            probes: 1,
            complete: true,
            unused_slot,
        }
    }

    fn origins() -> Vec<PressureOrigin> {
        vec![
            PressureOrigin {
                distance: 2,
                approach: NORTH_ORIGIN,
                first_step: CellId::new(6, 5),
            },
            PressureOrigin {
                distance: 2,
                approach: WEST_ORIGIN,
                first_step: CellId::new(5, 6),
            },
        ]
    }

    #[test]
    fn optional_comparison_needs_its_complete_charged_work_or_keeps_baseline() {
        let observation = collected_basis_at(START).observation;
        let mut geometry_work = WorkBudget::new(65_536);
        let geometry = ObservedBlastGeometry::new(&observation, &mut geometry_work).unwrap();
        let mut complete = rival(origins(), 65_536);
        let mut full = WorkBudget::new(65_536);
        try_promote(
            &observation,
            BlastKind::Standard,
            3,
            &geometry,
            &mut complete,
            &mut full,
        );
        assert_eq!(complete.baseline.unwrap().approach, WEST_ORIGIN);
        let exact = full.spent();
        assert!(exact > 0);

        let mut interrupted = rival(origins(), exact - 1);
        let mut short = WorkBudget::new(exact - 1);
        try_promote(
            &observation,
            BlastKind::Standard,
            3,
            &geometry,
            &mut interrupted,
            &mut short,
        );
        assert_eq!(interrupted.baseline.unwrap().approach, NORTH_ORIGIN);
    }

    #[test]
    fn optional_comparison_obeys_both_original_slot_and_shared_parent_boundaries() {
        let observation = collected_basis_at(START).observation;
        let mut geometry_work = WorkBudget::new(65_536);
        let geometry = ObservedBlastGeometry::new(&observation, &mut geometry_work).unwrap();
        let mut measured = rival(origins(), 65_536);
        let mut measurement = WorkBudget::new(65_536);
        try_promote(
            &observation,
            BlastKind::Standard,
            3,
            &geometry,
            &mut measured,
            &mut measurement,
        );
        let exact = measurement.spent();
        assert!(exact > 1);
        for (slot, parent, expected) in [
            (exact, exact, WEST_ORIGIN),
            (exact - 1, exact, NORTH_ORIGIN),
            (exact, exact - 1, NORTH_ORIGIN),
        ] {
            let mut records = [rival(origins(), slot)];
            let mut work = WorkBudget::new(parent);
            promote(
                &observation,
                BlastKind::Standard,
                3,
                &geometry,
                &mut records,
                &mut work,
            );
            assert_eq!(records[0].baseline.unwrap().approach, expected);
        }
    }

    #[test]
    fn successful_baseline_and_optional_candidates_share_the_per_rival_32_probe_cap() {
        let observation = collected_basis_at(START).observation;
        let mut geometry_work = WorkBudget::new(65_536);
        let geometry = ObservedBlastGeometry::new(&observation, &mut geometry_work).unwrap();
        let north = PressureOrigin {
            distance: 2,
            approach: NORTH_ORIGIN,
            first_step: CellId::new(6, 5),
        };
        let west = PressureOrigin {
            distance: 2,
            approach: WEST_ORIGIN,
            first_step: CellId::new(5, 6),
        };
        for (nonbetter, expected) in [(30, WEST_ORIGIN), (31, NORTH_ORIGIN)] {
            let mut candidates = vec![north];
            candidates.extend(std::iter::repeat_n(north, nonbetter));
            candidates.push(west);
            let mut record = rival(candidates, 65_536);
            let mut work = WorkBudget::new(65_536);
            try_promote(
                &observation,
                BlastKind::Standard,
                3,
                &geometry,
                &mut record,
                &mut work,
            );
            assert_eq!(record.baseline.unwrap().approach, expected);
        }
    }

    #[test]
    fn optional_exhaustion_never_discards_later_rivals_baselines() {
        let observation = collected_basis_at(START).observation;
        let mut geometry_work = WorkBudget::new(65_536);
        let geometry = ObservedBlastGeometry::new(&observation, &mut geometry_work).unwrap();
        let mut rivals = [rival(origins(), 1), rival(origins(), 1)];
        let mut work = WorkBudget::new(1);
        promote(
            &observation,
            BlastKind::Standard,
            3,
            &geometry,
            &mut rivals,
            &mut work,
        );
        assert!(rivals.iter().all(|rival| rival.baseline.is_some()));
        assert_eq!(rivals[1].baseline.unwrap().approach, NORTH_ORIGIN);
    }

    #[path = "../goals_v58_exit_pressure/gate_tests.rs"]
    mod gate_tests;
}
