use std::collections::{BTreeSet, VecDeque};

use crate::cpu::forecast::HazardForecast;
use crate::cpu::policy::ControlPlan;
use crate::cpu::{BombKnowledge, Observation, WorkBudget};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{CellId, Position, Terrain};

pub(super) fn certified_pulse_progresses(
    plan: &super::SurvivalPlan,
    initial_position: Position,
) -> bool {
    let Some(lease) = &plan.lease else {
        return false;
    };
    if lease.movement() == crate::cpu::MovementIntent::Stop {
        return false;
    }
    let end = lease.pose_after(lease.hold_ticks());
    end.min() != initial_position || end.max() != initial_position
}

/// Static guidance only. Every selected action still requires its own pulse
/// certificate; this search grants no timed-route or escape guarantee.
pub(super) fn target(
    observation: &Observation,
    forecast: &HazardForecast,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<CellId> {
    if forecast.dimensions()
        != (
            observation.parts().arena.width,
            observation.parts().arena.height,
        )
    {
        return None;
    }
    let position = super::compatibility::actor(observation)?.position;
    let current = position.cell()?;
    work.try_take(u32::try_from(observation.parts().arena.cell_count()).ok()?)
        .ok()?;
    let threatened = forecast.cells().iter().enumerate().any(|(index, hazards)| {
        let width = usize::from(observation.parts().arena.width);
        let cell = CellId::new((index % width) as u16, (index / width) as u16);
        position_overlaps_cell(position, cell) && !hazards.lethal.is_empty()
    });
    if !threatened {
        return ordinary
            .next_memory
            .route()
            .iter()
            .flatten()
            .copied()
            .find(|cell| *cell != current)
            .or(Some(current));
    }
    let occupied = if observation.parts().rules.version == 8 {
        let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
            return None;
        };
        work.try_take(u32::try_from(bombs.len()).ok()?).ok()?;
        bombs.iter().map(|bomb| bomb.cell).collect()
    } else {
        BTreeSet::new()
    };
    refuge_step(observation, forecast, current, &occupied, work)
}

pub(super) fn origin_target(
    observation: &Observation,
    forecast: &HazardForecast,
    origin: CellId,
    work: &mut WorkBudget,
) -> Option<CellId> {
    if forecast.dimensions()
        != (
            observation.parts().arena.width,
            observation.parts().arena.height,
        )
    {
        return None;
    }
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        return None;
    };
    work.try_take(u32::try_from(bombs.len()).ok()?).ok()?;
    let occupied = bombs
        .iter()
        .filter(|bomb| bomb.cell != origin)
        .map(|bomb| bomb.cell)
        .collect();
    refuge_step(observation, forecast, origin, &occupied, work)
}

fn refuge_step(
    observation: &Observation,
    forecast: &HazardForecast,
    root: CellId,
    occupied: &BTreeSet<CellId>,
    work: &mut WorkBudget,
) -> Option<CellId> {
    let mut queue = VecDeque::from([(root, root)]);
    let mut visited = BTreeSet::from([root]);
    while let Some((cell, first)) = queue.pop_front() {
        work.try_take(1).ok()?;
        let reached = cell != root && forecast.cell(cell)?.lethal.is_empty();
        if reached {
            return Some(first);
        }
        for next in adjacent(cell) {
            work.try_take(1).ok()?;
            if visited.insert(next) && !occupied.contains(&next) && traversable(observation, next) {
                queue.push_back((next, if cell == root { next } else { first }));
            }
        }
    }
    None
}

fn adjacent(cell: CellId) -> impl Iterator<Item = CellId> {
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .filter_map(move |(dc, dr)| {
            Some(CellId::new(
                u16::try_from(i32::from(cell.column) + dc).ok()?,
                u16::try_from(i32::from(cell.row) + dr).ok()?,
            ))
        })
}

fn traversable(observation: &Observation, cell: CellId) -> bool {
    let parts = observation.parts();
    if cell.column >= parts.arena.width || cell.row >= parts.arena.height {
        return false;
    }
    let index = usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
    parts.arena.terrain.get(index) == Some(&Terrain::Floor)
        && (crate::cpu::capabilities::can_pass_known_soft_blocks(observation)
            || !parts.arena.blocks.contains(&cell))
        && !parts.oil.iter().any(|oil| oil.cell == cell)
        && !parts.pickups.iter().any(|pickup| pickup.cell == cell)
        && !observation.spike_pits().is_some_and(|pits| {
            pits.pits()
                .iter()
                .any(|pit| pit.cell == cell && pit.phase != crate::game::SpikePitPhase::Idle)
        })
}

pub(super) fn score(lease: &crate::cpu::routing::PulseLease, target: CellId) -> u32 {
    let bounds = lease.pose_after(lease.hold_ticks());
    let target = Position::center(target);
    bounds
        .min()
        .x
        .abs_diff(target.x)
        .max(bounds.max().x.abs_diff(target.x))
        + bounds
            .min()
            .y
            .abs_diff(target.y)
            .max(bounds.max().y.abs_diff(target.y))
}

pub(super) fn distance(position: Position, target: CellId) -> u32 {
    let target = Position::center(target);
    position
        .x
        .abs_diff(target.x)
        .saturating_add(position.y.abs_diff(target.y))
}

#[cfg(test)]
mod origin_tests {
    use super::*;
    use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
    use crate::cpu::routing::tests::{key, observation};
    use crate::cpu::{
        BombKnowledge, ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedPickup,
        UnknownBombFacts,
    };
    use crate::game::{BlastKind, EnabledItems, EntityId, Equipment, PickupKind, Tick};

    fn guided(observation: &Observation, work: u32) -> Option<CellId> {
        let mut forecast_work = WorkBudget::new(65_536);
        let hazards = forecast(
            observation,
            &ObservedHazardMemory::new(key()),
            None,
            ForecastLimits::default(),
            &mut forecast_work,
        )
        .unwrap();
        origin_target(
            observation,
            &hazards,
            CellId::new(2, 1),
            &mut WorkBudget::new(work),
        )
    }

    #[test]
    fn rules_v8_threatened_guidance_never_routes_through_a_visible_bomb() {
        let observed = crate::cpu::lair_contact::tests::observation(Vec::new(), Tick(300))
            .map_parts(|parts| {
                parts.arena.blocks.push(CellId::new(1, 2));
                parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                    id: EntityId(91),
                    placed_at: Tick(90),
                    cell: CellId::new(2, 1),
                    owner: EntityId(1),
                    range: 1,
                    trigger: ObservedBombTrigger::Timed {
                        remaining_ticks: 60,
                        paused: false,
                    },
                    blast: BlastKind::Standard,
                    unknown: UnknownBombFacts::ALL,
                }]);
            })
            .unwrap();
        let key = crate::cpu::MemoryKey {
            world: observed.parts().world,
            cpu_seat: observed.parts().recipient,
        };
        let hazards = forecast(
            &observed,
            &ObservedHazardMemory::new(key),
            None,
            ForecastLimits::default(),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        let ordinary = ControlPlan::stop(crate::cpu::policy::UtilityMemory::new(key));

        assert_eq!(
            target(&observed, &hazards, &ordinary, &mut WorkBudget::new(128)),
            None
        );
    }

    #[test]
    fn origin_guidance_reuses_static_eligibility_and_excludes_other_bombs() {
        let clear = observation(Vec::new(), Vec::new());
        assert_eq!(guided(&clear, 128), Some(CellId::new(3, 1)));
        assert_eq!(guided(&clear, 0), None);

        let mut blocked = clear.clone().into_parts();
        blocked.arena.blocks.push(CellId::new(3, 1));
        assert_eq!(
            guided(&Observation::new(blocked.clone()).unwrap(), 128),
            Some(CellId::new(2, 2))
        );

        blocked.rules.settings = blocked
            .rules
            .settings
            .with_enabled_items(EnabledItems::WALL_PASS);
        let ObservedActor::Visible(actor) = &mut blocked.actors[1] else {
            unreachable!()
        };
        actor.equipment = Some(Equipment::WallPass);
        assert_eq!(
            guided(&Observation::new(blocked).unwrap(), 128),
            Some(CellId::new(3, 1))
        );

        let mut pickup = clear.clone().into_parts();
        pickup.pickups.push(ObservedPickup {
            cell: CellId::new(3, 1),
            kind: PickupKind::BombCapacity,
        });
        assert_eq!(
            guided(&Observation::new(pickup).unwrap(), 128),
            Some(CellId::new(2, 2))
        );

        let mut bomb = clear.into_parts();
        bomb.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(91),
            placed_at: Tick(0),
            cell: CellId::new(3, 1),
            owner: EntityId(1),
            range: 1,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: 60,
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]);
        assert_eq!(
            guided(&Observation::new(bomb).unwrap(), 128),
            Some(CellId::new(2, 2))
        );
    }

    #[test]
    fn fixed_refuge_progress_rejects_the_observed_two_pulse_cycle() {
        let refuge = CellId::new(13, 11);
        let start = Position {
            x: 13_824,
            y: 10_599,
        };
        let aligned = Position {
            x: 13_824,
            y: 10_752,
        };
        let continued = Position {
            x: 13_824,
            y: 10_905,
        };
        assert_eq!(distance(start, refuge), 1_177);
        assert_eq!(distance(aligned, refuge), 1_024);
        assert_eq!(distance(continued, refuge), 871);
        assert!(distance(continued, refuge) < distance(aligned, refuge));
        assert!(distance(start, refuge) >= distance(aligned, refuge));
    }
}
