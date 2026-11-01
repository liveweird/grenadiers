//! Opt-in farm planting through a jointly route- and contact-safe refuge hint.

use crate::cpu::policy::{ControlPlan, StrategicTarget, UtilityMemory};
use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant_to_refuge};
use crate::cpu::{Intent, ManualIntent, MobContactLease, MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, Direction, Tick};

use super::compatibility::actor;
use super::plant_tactic::PlantSupport;
use super::{ExecutionCertificate, LairSurvivalPlan, SurvivalPlan};

const MAX_HINT_WORK: u32 = 8_192;

#[allow(clippy::too_many_arguments)]
pub(super) fn certified_farm_plant(
    observation: &Observation,
    history: &crate::cpu::forecast::ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    support: PlantSupport,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let intent = ordinary.next_memory.strategic_intent()?;
    let StrategicTarget::Block { cell: target } = intent.target else {
        return None;
    };
    let origin = intent.approach_cell?;
    let actor = actor(observation)?;
    if actor.weapon.is_some()
        || actor.position.cell() != Some(origin)
        || ordinary.intent.manual == Some(ManualIntent::Equipment)
        || !super::plant_tactic::teleporter_plant_supported(observation, support, work)
        || !super::plant_value::could_be_useful(observation, work)
    {
        return None;
    }
    let (movement, hints) = refuge_hints(observation, origin, target, actor.bomb_range)?;
    let allowance = MAX_HINT_WORK.min(work.remaining() / 8);
    let mut child = WorkBudget::new(allowance);
    child.try_take(2).ok()?;
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    for refuge in hints {
        let certificate_result = plan_lair_ordinary_plant_to_refuge(
            observation,
            history,
            PlantRequest {
                execution_tick,
                settled_stop_tick,
                first_movement: movement,
                limits: RouteLimits::default(),
            },
            refuge,
            &mut child,
        );
        let certificate = certificate_result.ok();
        let Some(certificate) = certificate else {
            continue;
        };
        let Some(useful) = super::plant_value::useful_plant_for(
            observation,
            certificate.placement(),
            intent.target,
            &mut child,
        ) else {
            break;
        };
        if !useful {
            continue;
        }
        let Ok(contact) =
            MobContactLease::certify_route(observation, certificate.route(), &mut child)
        else {
            continue;
        };
        work.try_take(child.spent()).ok()?;
        return Some(LairSurvivalPlan {
            survival: SurvivalPlan {
                control: ControlPlan {
                    intent: Intent {
                        movement,
                        manual: Some(ManualIntent::Bomb),
                    },
                    hold_ticks: 1,
                    next_memory: UtilityMemory::new(memory.key())
                        .carry_progress(ordinary.next_memory)
                        .mark_strategic_phase(
                            crate::cpu::policy::StrategicPhase::Plant,
                            intent.last_switch_reason,
                            observation.parts().tick,
                        ),
                },
                lease: None,
                execution: Some(ExecutionCertificate::Plant(Box::new(certificate))),
            },
            contact: Some(contact),
        });
    }
    work.try_take(child.spent()).ok()?;
    None
}

fn refuge_hints(
    observation: &Observation,
    origin: CellId,
    target: CellId,
    range: u16,
) -> Option<(MovementIntent, [CellId; 2])> {
    let (away_column, away_row, lateral_column, lateral_row, direction) = match (
        target.column as i32 - origin.column as i32,
        target.row as i32 - origin.row as i32,
    ) {
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
        let column = base_column + lateral_column * lateral * sign;
        let row = base_row + lateral_row * lateral * sign;
        (column >= 0
            && row >= 0
            && column < i32::from(observation.parts().arena.width)
            && row < i32::from(observation.parts().arena.height))
        .then(|| CellId::new(column as u16, row as u16))
    };
    let mut hints = [make(-1)?, make(1)?];
    hints.sort_by_key(|cell| {
        (
            std::cmp::Reverse(public_danger_clearance(observation, *cell)),
            *cell,
        )
    });
    Some((MovementIntent::Direction(direction), hints))
}

fn public_danger_clearance(observation: &Observation, cell: CellId) -> u32 {
    let lair_cells = observation
        .respawn_lairs()
        .into_iter()
        .flat_map(|lairs| lairs.lairs().iter().map(|lair| lair.cell));
    let mob_cells = observation
        .respawn_lairs()
        .into_iter()
        .flat_map(|lairs| lairs.mobs().iter().filter_map(|mob| mob.position.cell()));
    lair_cells
        .chain(mob_cells)
        .map(|danger| {
            u32::from(cell.column.abs_diff(danger.column))
                + u32::from(cell.row.abs_diff(danger.row))
        })
        .min()
        .unwrap_or(u32::MAX)
}
