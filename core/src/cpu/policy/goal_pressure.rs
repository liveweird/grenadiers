use crate::cpu::{Observation, VisibleActor, WorkBudget};
use std::collections::VecDeque;

use crate::game::geometry::position_overlaps_cell;
use crate::game::{BlastKind, CellId, Position, SpecialWeapon, Terrain};

use super::{STEPS, SearchMap, cell_index, offset};

pub(super) fn add(
    observation: &Observation,
    actor: &VisibleActor,
    rival: Position,
    map: &mut SearchMap,
    work: &mut WorkBudget,
) -> Option<()> {
    let Some((kind, range)) = weapon_geometry(observation, actor) else {
        return Some(());
    };
    let geometry = crate::cpu::survival::ObservedBlastGeometry::new(observation, work)?;
    if kind == BlastKind::Power {
        return add_nearest_power(observation, actor, rival, range, map, &geometry, work);
    }
    for row in 0..observation.parts().arena.height {
        for column in 0..observation.parts().arena.width {
            work.try_take(1).ok()?;
            let cell = CellId::new(column, row);
            if Some(cell) != rival.cell()
                && geometry.threatens_position(cell, range, kind, rival, work)?
                && map.transitable(observation, cell)
            {
                map.add_goal(cell_index(observation, cell)?, 4);
            }
        }
    }
    Some(())
}

pub(super) fn weapon_geometry(
    observation: &Observation,
    actor: &VisibleActor,
) -> Option<(BlastKind, u16)> {
    let enabled = observation.parts().rules.settings.enabled_items();
    let kind = match actor.weapon {
        None => BlastKind::Standard,
        Some(SpecialWeapon::Piercing) if enabled.contains_piercing_bomb() => BlastKind::Piercing,
        Some(SpecialWeapon::Power) if enabled.contains_power_bomb() => BlastKind::Power,
        Some(
            SpecialWeapon::Remote
            | SpecialWeapon::Landmine
            | SpecialWeapon::Piercing
            | SpecialWeapon::Power,
        ) => return None,
    };
    let placement = crate::game::Tick(observation.parts().tick.0.checked_add(7)?);
    let (range, _) = crate::cpu::routing::derive_bomb_modifiers(
        actor,
        observation.parts().tick,
        placement,
        crate::cpu::rules_config(observation)?,
    )
    .ok()?;
    Some((kind, range))
}

fn add_nearest_power(
    observation: &Observation,
    actor: &VisibleActor,
    rival: Position,
    range: u16,
    map: &mut SearchMap,
    geometry: &crate::cpu::survival::ObservedBlastGeometry<'_>,
    work: &mut WorkBudget,
) -> Option<()> {
    let start = actor.position.cell()?;
    let target_cells = (0..observation.parts().arena.height)
        .flat_map(|row| {
            (0..observation.parts().arena.width).map(move |column| CellId::new(column, row))
        })
        .filter(|cell| {
            observation.parts().arena.index(*cell).is_some_and(|index| {
                observation.parts().arena.terrain[index] == Terrain::Floor
                    && position_overlaps_cell(rival, *cell)
            })
        })
        .collect::<Vec<_>>();
    work.try_take(u32::try_from(observation.parts().arena.cell_count()).ok()?)
        .ok()?;
    let mut queue = VecDeque::from([start]);
    let mut visited = [false; crate::game::arena::ARENA_CELL_COUNT];
    visited[cell_index(observation, start)?] = true;
    while let Some(cell) = queue.pop_front() {
        work.try_take(1).ok()?;
        let in_range = target_cells.iter().any(|target| {
            cell.column.abs_diff(target.column) + cell.row.abs_diff(target.row) <= range
        });
        if Some(cell) != rival.cell()
            && in_range
            && map.transitable(observation, cell)
            && geometry.threatens_position(cell, range, BlastKind::Power, rival, work)?
        {
            map.add_goal(cell_index(observation, cell)?, 4);
            return Some(());
        }
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            let Some(index) = cell_index(observation, next) else {
                continue;
            };
            if !visited[index] && map.transitable(observation, next) {
                visited[index] = true;
                queue.push_back(next);
            }
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use crate::cpu::{Observation, ObservedActor, ObservedEffect, ObservedEnvironment, WorkBudget};
    use crate::game::{
        CellId, EffectKind, EnabledItems, EntityId, Position, SeatId, SpecialWeapon,
        StormCloudConfig,
    };

    use super::weapon_geometry;

    fn scene(
        weapon: Option<SpecialWeapon>,
        enabled: EnabledItems,
        target: CellId,
        blocks: Vec<CellId>,
    ) -> Observation {
        let mut parts = crate::cpu::policy::tests::observation(CellId::new(7, 7)).into_parts();
        parts.rules.version = 9;
        parts.rules.settings = parts.rules.settings.with_enabled_items(enabled);
        parts.arena.blocks = blocks;
        let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
            unreachable!()
        };
        cpu.weapon = weapon;
        cpu.bomb_range = 4;
        let mut rival = cpu.clone();
        rival.id = EntityId(1);
        rival.seat = SeatId::One;
        rival.position = Position::center(target);
        rival.weapon = None;
        parts.actors[0] = ObservedActor::Visible(rival);
        Observation::new_rules_v9(
            parts,
            ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0), Some(0)]),
            None,
        )
        .unwrap()
    }

    fn threat(observation: &Observation, firing: CellId, work: u32) -> Option<bool> {
        let actor = observation
            .parts()
            .actors
            .iter()
            .find_map(|actor| match actor {
                ObservedActor::Visible(actor) if actor.seat == SeatId::Two => Some(actor),
                _ => None,
            })
            .unwrap();
        let rival = observation
            .parts()
            .actors
            .iter()
            .find_map(|actor| match actor {
                ObservedActor::Visible(actor) if actor.seat == SeatId::One => Some(actor.position),
                _ => None,
            })
            .unwrap();
        let mut work = WorkBudget::new(work);
        let geometry = crate::cpu::survival::ObservedBlastGeometry::new(observation, &mut work)?;
        let (kind, range) = weapon_geometry(observation, actor)?;
        geometry.threatens_position(firing, range, kind, rival, &mut work)
    }

    #[test]
    fn power_uses_shared_diagonal_geometry_and_piercing_crosses_soft_blocks() {
        let power = scene(
            Some(SpecialWeapon::Power),
            EnabledItems::POWER_BOMB,
            CellId::new(9, 9),
            vec![CellId::new(8, 8)],
        );
        assert_eq!(threat(&power, CellId::new(7, 7), 65_536), Some(true));

        let standard = scene(
            None,
            EnabledItems::EMPTY,
            CellId::new(9, 7),
            vec![CellId::new(8, 7)],
        );
        assert_eq!(threat(&standard, CellId::new(7, 7), 65_536), Some(false));
        let piercing = scene(
            Some(SpecialWeapon::Piercing),
            EnabledItems::PIERCING_BOMB,
            CellId::new(9, 7),
            vec![CellId::new(8, 7)],
        );
        assert_eq!(threat(&piercing, CellId::new(7, 7), 65_536), Some(true));
    }

    #[test]
    fn pressure_uses_placement_time_range_modifiers_and_bounded_work() {
        let enabled = EnabledItems::from_bits(
            EnabledItems::POWER_BOMB.bits() | EnabledItems::GOLDEN_FLAME.bits(),
        )
        .unwrap();
        let mut observation = scene(
            Some(SpecialWeapon::Power),
            enabled,
            CellId::new(13, 7),
            Vec::new(),
        );
        let environment = observation.environment().unwrap().clone();
        let mut parts = observation.into_parts();
        let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
            unreachable!()
        };
        cpu.bomb_range = 2;
        cpu.effects.push(ObservedEffect {
            kind: EffectKind::GoldenFlame,
            remaining_ticks: 8,
        });
        observation = Observation::new_rules_v9(parts, environment, None).unwrap();
        assert_eq!(threat(&observation, CellId::new(7, 7), 65_536), Some(true));
        assert_eq!(threat(&observation, CellId::new(7, 7), 0), None);
    }
}
