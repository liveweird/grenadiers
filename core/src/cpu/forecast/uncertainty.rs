use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::ControlFlow;

use crate::cpu::{Observation, WorkBudget};
use crate::game::arena::Terrain;
use crate::game::blast_geometry::{BlastSource, visit_blast};
use crate::game::{CellId, EntityId, OilBarrelState, RulesConfig, Tick};

use super::intervals::{add_uncertain, take};
use super::model::{ForecastBomb, ForecastField, Trigger, index, offset, terrain};
use super::{
    CellForecast, ForecastFailure, ForecastLimits, HazardKind, TickWindow, UncertainWindow,
    UncertaintyReason,
};

struct MotionIndex {
    positions: BTreeMap<EntityId, BTreeSet<CellId>>,
    occupants: BTreeMap<CellId, Vec<EntityId>>,
}

pub(super) fn add_uncertainties(
    observation: &Observation,
    bombs: &[ForecastBomb],
    horizon_end: Tick,
    limits: ForecastLimits,
    work: &mut WorkBudget,
    cells: &mut [CellForecast],
) -> Result<(), ForecastFailure> {
    let basis = observation.parts().tick;
    let flame_ticks = RulesConfig::expanded_v5(observation.parts().rules.settings).flame_ticks();
    let MotionIndex {
        positions: motion_positions,
        occupants: possible_occupants,
    } = build_motion_index(observation, bombs, horizon_end, work)?;
    for bomb in bombs.iter().filter(|bomb| !bomb.hypothetical) {
        take(work, 1)?;
        let positions = &motion_positions[&bomb.id];
        let blocking_end = match bomb.trigger {
            Trigger::Timed(at) => Tick(at.0.saturating_add(1)).min(horizon_end),
            Trigger::Paused(_) | Trigger::Remote | Trigger::Contact => horizon_end,
        };
        for cell in positions {
            add_uncertain(
                &mut cells[index(*cell)].uncertainty,
                UncertainWindow {
                    window: TickWindow {
                        start: basis,
                        end: blocking_end,
                    },
                    kind: HazardKind::Blocking,
                    reason: UncertaintyReason::BombMotion(bomb.id),
                },
                limits,
                work,
            )?;
        }

        let (start, reason, broad_time) = match bomb.trigger {
            Trigger::Timed(at) => (at, UncertaintyReason::BombMotion(bomb.id), false),
            Trigger::Paused(at) => (at, UncertaintyReason::Pause(bomb.id), true),
            Trigger::Remote | Trigger::Contact => (
                Tick(
                    basis
                        .0
                        .checked_add(1)
                        .ok_or(ForecastFailure::ClockOverflow)?,
                ),
                UncertaintyReason::Trigger(bomb.id),
                true,
            ),
        };
        if start.0 < horizon_end.0 {
            let end = if broad_time {
                horizon_end
            } else {
                Tick(
                    start
                        .0
                        .checked_add(flame_ticks)
                        .ok_or(ForecastFailure::ClockOverflow)?,
                )
                .min(horizon_end)
            };
            add_blast_closure(
                observation,
                bombs,
                bomb,
                positions.iter().copied(),
                TickWindow { start, end },
                reason,
                horizon_end,
                limits,
                work,
                cells,
                &motion_positions,
                &possible_occupants,
            )?;
        }

        if !matches!(bomb.trigger, Trigger::Contact)
            && basis.0 < horizon_end.0.saturating_sub(1)
            && let Some(ignition) = earliest_fire(positions, cells)
        {
            add_blast_closure(
                observation,
                bombs,
                bomb,
                positions.iter().copied(),
                TickWindow {
                    start: Tick((basis.0 + 1).max(ignition.0)),
                    end: horizon_end,
                },
                UncertaintyReason::BombMotion(bomb.id),
                horizon_end,
                limits,
                work,
                cells,
                &motion_positions,
                &possible_occupants,
            )?;
        }

        // Moving A can remove shielding from otherwise unrelated exact timed
        // blasts. Revisit each such source without temporary blockers, keeping
        // A as provenance so waiving A restores the nominal model as a whole.
        for other in bombs.iter().filter(|other| other.id != bomb.id) {
            let Trigger::Timed(at) = other.trigger else {
                continue;
            };
            if at.0 >= horizon_end.0 {
                continue;
            }
            let end = Tick(
                at.0.checked_add(flame_ticks)
                    .ok_or(ForecastFailure::ClockOverflow)?,
            )
            .min(horizon_end);
            add_blast_closure(
                observation,
                bombs,
                other,
                [other.cell],
                TickWindow { start: at, end },
                UncertaintyReason::BombMotion(bomb.id),
                horizon_end,
                limits,
                work,
                cells,
                &motion_positions,
                &possible_occupants,
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn add_blast_closure(
    observation: &Observation,
    bombs: &[ForecastBomb],
    initial: &ForecastBomb,
    positions: impl IntoIterator<Item = CellId>,
    window: TickWindow,
    reason: UncertaintyReason,
    horizon_end: Tick,
    limits: ForecastLimits,
    work: &mut WorkBudget,
    cells: &mut [CellForecast],
    motion_positions: &BTreeMap<EntityId, BTreeSet<CellId>>,
    possible_occupants: &BTreeMap<CellId, Vec<EntityId>>,
) -> Result<(), ForecastFailure> {
    take(
        work,
        u32::try_from(bombs.len() + observation.parts().oil.len())
            .map_err(|_| ForecastFailure::WorkExhausted)?,
    )?;
    let empty_cells = BTreeSet::new();
    let empty_bombs = BTreeMap::new();
    let field = ForecastField {
        observation,
        blocks: &empty_cells,
        oil: &empty_cells,
        bombs: &empty_bombs,
    };
    let unlit_oil = observation
        .parts()
        .oil
        .iter()
        .filter_map(|oil| (oil.state == OilBarrelState::Unlit).then_some(oil.cell))
        .collect::<BTreeSet<_>>();
    // A target can enter this possible fire on its final active tick and then
    // create a fresh full-duration blast. Widening propagated uncertainty to
    // the horizon closes that late-ignition chain without branching timelines.
    let propagated_window = TickWindow {
        start: window.start,
        end: horizon_end,
    };
    let mut queue = positions
        .into_iter()
        .map(|cell| (*initial, cell, reason, initial.active_from))
        .collect::<VecDeque<_>>();
    let mut expanded = BTreeSet::from([(initial.id, reason_key(reason))]);
    while let Some((source, cell, cause, active_from)) = queue.pop_front() {
        take(work, 1)?;
        let source_window = TickWindow {
            start: propagated_window.start.max(active_from),
            end: propagated_window.end,
        };
        if source_window.start >= source_window.end {
            continue;
        }
        let mut failure = None;
        let result = visit_blast(
            &field,
            BlastSource {
                cell,
                range: source.range,
                kind: source.blast,
            },
            |visit| {
                if let Err(error) = take(work, 1).and_then(|()| {
                    add_uncertain(
                        &mut cells[index(visit.cell)].uncertainty,
                        UncertainWindow {
                            window: source_window,
                            kind: HazardKind::Lethal,
                            reason: cause,
                        },
                        limits,
                        work,
                    )
                }) {
                    failure = Some(error);
                    return ControlFlow::Break(());
                }
                if unlit_oil.contains(&visit.cell)
                    && let Err(error) = add_uncertain(
                        &mut cells[index(visit.cell)].uncertainty,
                        UncertainWindow {
                            window: source_window,
                            kind: HazardKind::Lethal,
                            reason: cause,
                        },
                        limits,
                        work,
                    )
                {
                    failure = Some(error);
                    return ControlFlow::Break(());
                }
                if let Some(ids) = possible_occupants.get(&visit.cell) {
                    for id in ids {
                        let hit = bombs
                            .iter()
                            .find(|bomb| bomb.id == *id)
                            .expect("motion index references forecast bomb");
                        if matches!(hit.trigger, Trigger::Contact) {
                            continue;
                        }
                        enqueue(&mut queue, &mut expanded, *hit, cause, motion_positions);
                        if !hit.hypothetical {
                            enqueue(
                                &mut queue,
                                &mut expanded,
                                *hit,
                                UncertaintyReason::BombMotion(hit.id),
                                motion_positions,
                            );
                        }
                    }
                }
                ControlFlow::Continue(())
            },
        );
        if result.is_break() {
            return Err(failure.unwrap_or(ForecastFailure::WorkExhausted));
        }
    }
    Ok(())
}

fn enqueue(
    queue: &mut VecDeque<(ForecastBomb, CellId, UncertaintyReason, Tick)>,
    expanded: &mut BTreeSet<(EntityId, (u8, u32))>,
    bomb: ForecastBomb,
    reason: UncertaintyReason,
    motion_positions: &BTreeMap<EntityId, BTreeSet<CellId>>,
) {
    if expanded.insert((bomb.id, reason_key(reason))) {
        queue.extend(
            motion_positions[&bomb.id]
                .iter()
                .copied()
                .map(|cell| (bomb, cell, reason, bomb.active_from)),
        );
    }
}

const fn reason_key(reason: UncertaintyReason) -> (u8, u32) {
    match reason {
        UncertaintyReason::BombMotion(id) => (0, id.0),
        UncertaintyReason::Trigger(id) => (1, id.0),
        UncertaintyReason::Pause(id) => (2, id.0),
    }
}

fn build_motion_index(
    observation: &Observation,
    bombs: &[ForecastBomb],
    horizon_end: Tick,
    work: &mut WorkBudget,
) -> Result<MotionIndex, ForecastFailure> {
    let mut positions = BTreeMap::new();
    for bomb in bombs {
        let cells = if bomb.hypothetical {
            take(work, 1)?;
            BTreeSet::from([bomb.cell])
        } else {
            possible_motion_cells(observation, bomb, horizon_end, work)?
        };
        positions.insert(bomb.id, cells);
    }
    let mut occupants: BTreeMap<CellId, Vec<EntityId>> = BTreeMap::new();
    for (id, possible) in &positions {
        for cell in possible {
            take(work, 1)?;
            occupants.entry(*cell).or_default().push(*id);
        }
    }
    Ok(MotionIndex {
        positions,
        occupants,
    })
}

fn earliest_fire(positions: &BTreeSet<CellId>, cells: &[CellForecast]) -> Option<Tick> {
    positions
        .iter()
        .flat_map(|cell| cells[index(*cell)].lethal.iter())
        .map(|window| window.start)
        .min()
}

fn possible_motion_cells(
    observation: &Observation,
    bomb: &ForecastBomb,
    horizon_end: Tick,
    work: &mut WorkBudget,
) -> Result<BTreeSet<CellId>, ForecastFailure> {
    take(work, 1)?;
    let mut cells = BTreeSet::from([bomb.cell]);
    let span = match bomb.trigger {
        Trigger::Timed(at) => at.0.saturating_sub(observation.parts().tick.0),
        Trigger::Paused(_) | Trigger::Remote | Trigger::Contact => {
            horizon_end.0 - observation.parts().tick.0
        }
    };
    let steps = usize::try_from(span.saturating_add(9) / 10).unwrap_or(usize::MAX);
    for (dx, dy) in [(0_i32, -1_i32), (1, 0), (0, 1), (-1, 0)] {
        let mut cell = bomb.cell;
        for _ in 0..steps {
            take(work, 1)?;
            let Some(next) = offset(cell, dx, dy) else {
                break;
            };
            if terrain(observation.parts(), next) != Some(Terrain::Floor) {
                break;
            }
            cells.insert(next);
            cell = next;
        }
    }
    Ok(cells)
}
