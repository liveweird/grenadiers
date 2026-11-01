use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::ControlFlow;

use crate::cpu::routing::PlantCertificate;
use crate::cpu::survival::ObservedBlastGeometry;
use crate::cpu::{BombKnowledge, MemoryKey, Observation, WorkBudget};
use crate::game::blast_geometry::{BlastSource, visit_blast};
use crate::game::geometry::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{BlastKind, CellId, PickupKind, Position, Terrain, Tick};

use super::{MobContactEnvelope, Source, SourceKind, source_reaches_segment, travel_units};

pub(crate) const PLANT_CONTACT_RENEWAL_ALLOWANCE: u32 = 8_192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BoundSource {
    source: Source,
    component: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlantContactProof {
    key: MemoryKey,
    basis_tick: Tick,
    safe_until: Tick,
    retained_blocks: BTreeSet<CellId>,
    components: BTreeMap<CellId, u16>,
    sources: Vec<BoundSource>,
    envelope: MobContactEnvelope,
    blocked_births: bool,
}

impl PlantContactProof {
    pub(crate) fn certify(
        observation: &Observation,
        plant: &PlantCertificate,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        Self::certify_inner(observation, plant, false, work)
    }

    pub(crate) fn certify_blocked_births(
        observation: &Observation,
        plant: &PlantCertificate,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        Self::certify_inner(observation, plant, true, work)
    }

    fn certify_inner(
        observation: &Observation,
        plant: &PlantCertificate,
        allow_blocked_births: bool,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        let parts = observation.parts();
        if parts.rules.version != 10
            || parts.rules.running_updates == 0
            || plant.route().basis_tick() != parts.tick
            || plant.route().key()
                != (MemoryKey {
                    world: parts.world,
                    cpu_seat: parts.recipient,
                })
            || plant.placement().blast != BlastKind::Standard
            || !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
            || !parts.flames.is_empty()
            || !parts.oil.is_empty()
        {
            return None;
        }
        let initial_scans = parts.pickups.len().checked_add(parts.actors.len())?;
        work.try_take(u32::try_from(initial_scans).ok()?).ok()?;
        if parts
            .pickups
            .iter()
            .any(|pickup| pickup.kind == PickupKind::Teleporter)
        {
            return None;
        }
        observation.respawn_lairs()?;
        let envelope = MobContactEnvelope::new(observation, work).ok()?;
        let opened = opened_blocks(observation, plant, work)?;
        work.try_take(u32::try_from(parts.arena.blocks.len().checked_add(opened.len())?).ok()?)
            .ok()?;
        let blocks = parts.arena.blocks.iter().copied().collect::<BTreeSet<_>>();
        let retained_blocks = blocks.difference(&opened).copied().collect::<BTreeSet<_>>();
        let components = label_components(observation, &retained_blocks, work)?;
        let sources = bind_sources(
            &envelope,
            &retained_blocks,
            &components,
            allow_blocked_births,
            work,
        )?;
        let blocked_births = sources.is_empty();
        if !blocked_births {
            let dilations = component_dilations(
                &sources,
                &components,
                parts.arena.width,
                parts.arena.height,
                work,
            )?;
            route_avoids(
                plant,
                &sources,
                &dilations,
                parts.arena.width,
                parts.arena.height,
                work,
            )?;
        }
        Some(Self {
            key: plant.route().key(),
            basis_tick: parts.tick,
            safe_until: plant.route().survive_until(),
            retained_blocks,
            components,
            sources,
            envelope,
            blocked_births,
        })
    }

    pub(crate) const fn safe_until(&self) -> Tick {
        self.safe_until
    }

    pub(crate) const fn requires_blocked_birth_support(&self) -> bool {
        self.blocked_births
    }
}

#[path = "lair_contact_topology_renewal.rs"]
mod renewal;

fn opened_blocks(
    observation: &Observation,
    plant: &PlantCertificate,
    work: &mut WorkBudget,
) -> Option<BTreeSet<CellId>> {
    let geometry = ObservedBlastGeometry::new(observation, work)?;
    let mut opened = BTreeSet::new();
    let mut exhausted = false;
    let result = visit_blast(
        &geometry,
        BlastSource {
            cell: plant.placement().cell,
            range: plant.placement().range,
            kind: plant.placement().blast,
        },
        |visit| {
            if work.try_take(1).is_err() {
                exhausted = true;
                return ControlFlow::Break(());
            }
            if visit.soft_block {
                opened.insert(visit.cell);
            }
            ControlFlow::Continue(())
        },
    );
    (!exhausted && result.is_continue()).then_some(opened)
}

fn label_components(
    observation: &Observation,
    blocked: &BTreeSet<CellId>,
    work: &mut WorkBudget,
) -> Option<BTreeMap<CellId, u16>> {
    let parts = observation.parts();
    let mut labels = BTreeMap::new();
    let mut next = 0_u16;
    for row in 0..parts.arena.height {
        for column in 0..parts.arena.width {
            work.try_take(1).ok()?;
            let start = CellId::new(column, row);
            if labels.contains_key(&start) || !passable(observation, start, blocked) {
                continue;
            }
            let mut queue = VecDeque::from([start]);
            while let Some(cell) = queue.pop_front() {
                work.try_take(1).ok()?;
                if labels.insert(cell, next).is_some() {
                    continue;
                }
                for neighbor in neighbors(cell, parts.arena.width, parts.arena.height) {
                    work.try_take(1).ok()?;
                    if !labels.contains_key(&neighbor) && passable(observation, neighbor, blocked) {
                        queue.push_back(neighbor);
                    }
                }
            }
            next = next.checked_add(1)?;
        }
    }
    Some(labels)
}

fn bind_sources(
    envelope: &MobContactEnvelope,
    retained: &BTreeSet<CellId>,
    components: &BTreeMap<CellId, u16>,
    allow_empty: bool,
    work: &mut WorkBudget,
) -> Option<Vec<BoundSource>> {
    let mut result = Vec::new();
    work.try_take(u32::try_from(envelope.sources().len()).ok()?)
        .ok()?;
    for source in envelope.sources() {
        let cell = source.origin.cell()?;
        if matches!(source.kind, SourceKind::PossibleSpawn { .. }) && retained.contains(&cell) {
            continue;
        }
        result.push(BoundSource {
            source: *source,
            component: *components.get(&cell)?,
        });
    }
    (!result.is_empty() || (allow_empty && !envelope.sources().is_empty())).then_some(result)
}

#[cfg(test)]
pub(crate) fn blocked_birth_binding_for_test(
    observation: &Observation,
    allowance: u32,
) -> (bool, u32) {
    let mut envelope_work = WorkBudget::new(crate::cpu::survival::WORK_ALLOWANCE);
    let envelope = MobContactEnvelope::new(observation, &mut envelope_work).unwrap();
    let retained = observation
        .parts()
        .arena
        .blocks
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut work = WorkBudget::new(allowance);
    let result = bind_sources(&envelope, &retained, &BTreeMap::new(), true, &mut work).is_some();
    (result, work.spent())
}

fn component_dilations(
    sources: &[BoundSource],
    components: &BTreeMap<CellId, u16>,
    width: u16,
    height: u16,
    work: &mut WorkBudget,
) -> Option<BTreeMap<u16, BTreeSet<CellId>>> {
    work.try_take(u32::try_from(sources.len()).ok()?).ok()?;
    let wanted = sources
        .iter()
        .map(|source| source.component)
        .collect::<BTreeSet<_>>();
    let mut result = BTreeMap::<u16, BTreeSet<CellId>>::new();
    work.try_take(u32::try_from(components.len()).ok()?).ok()?;
    for (cell, component) in components {
        if !wanted.contains(component) {
            continue;
        }
        result.entry(*component).or_default().insert(*cell);
        for neighbor in neighbors(*cell, width, height) {
            work.try_take(1).ok()?;
            result.entry(*component).or_default().insert(neighbor);
        }
    }
    Some(result)
}

fn route_avoids(
    plant: &PlantCertificate,
    sources: &[BoundSource],
    dilations: &BTreeMap<u16, BTreeSet<CellId>>,
    width: u16,
    height: u16,
    work: &mut WorkBudget,
) -> Option<()> {
    let route = plant.route();
    let mut checked = BTreeSet::new();
    for first in route.start_window().start.0..route.start_window().end.0 {
        let lane = route.lane(Tick(first))?;
        let mut previous = lane.pose_at(route.basis_tick())?.0;
        for tick in route.basis_tick().0 + 1..route.survive_until().0 {
            let current = lane.pose_at(Tick(tick))?.0;
            let segment = MovementSegment::new(previous, current);
            for source in sources {
                work.try_take(1).ok()?;
                if Tick(tick) < source.source.begins_at
                    || !source_reaches_segment(
                        source.source,
                        travel_units(source.source.begins_at, Tick(tick)).ok()?,
                        segment,
                    )
                {
                    continue;
                }
                let key = (rectangle_key(previous, current), source.component);
                if checked.insert(key)
                    && !rectangle_clear(
                        segment,
                        dilations.get(&source.component)?,
                        width,
                        height,
                        work,
                    )?
                {
                    return None;
                }
            }
            previous = current;
        }
    }
    Some(())
}

fn rectangle_clear(
    segment: MovementSegment,
    dilation: &BTreeSet<CellId>,
    width: u16,
    height: u16,
    work: &mut WorkBudget,
) -> Option<bool> {
    let start = segment.start();
    let end = segment.end();
    let min_column = (start.x.min(end.x) - HALF_FOOTPRINT)
        .div_euclid(UNITS_PER_CELL)
        .max(0);
    let max_column = (start.x.max(end.x) + HALF_FOOTPRINT - 1)
        .div_euclid(UNITS_PER_CELL)
        .min(i32::from(width) - 1);
    let min_row = (start.y.min(end.y) - HALF_FOOTPRINT)
        .div_euclid(UNITS_PER_CELL)
        .max(0);
    let max_row = (start.y.max(end.y) + HALF_FOOTPRINT - 1)
        .div_euclid(UNITS_PER_CELL)
        .min(i32::from(height) - 1);
    let count = (max_column - min_column + 1)
        .checked_mul(max_row - min_row + 1)
        .and_then(|value| u32::try_from(value).ok())?;
    work.try_take(count).ok()?;
    for row in min_row..=max_row {
        for column in min_column..=max_column {
            let cell = CellId::new(column as u16, row as u16);
            if segment.overlaps_cell(cell) && dilation.contains(&cell) {
                return Some(false);
            }
        }
    }
    Some(true)
}

fn rectangle_key(start: Position, end: Position) -> (i32, i32, i32, i32) {
    (
        start.x.min(end.x),
        start.x.max(end.x),
        start.y.min(end.y),
        start.y.max(end.y),
    )
}

fn passable(observation: &Observation, cell: CellId, blocked: &BTreeSet<CellId>) -> bool {
    observation.parts().arena.index(cell).is_some_and(|index| {
        observation.parts().arena.terrain.get(index) == Some(&Terrain::Floor)
            && !blocked.contains(&cell)
    })
}

fn neighbors(cell: CellId, width: u16, height: u16) -> impl Iterator<Item = CellId> {
    (-1_i32..=1).flat_map(move |row_delta| {
        (-1_i32..=1).filter_map(move |column_delta| {
            if column_delta == 0 && row_delta == 0 {
                return None;
            }
            let column = i32::from(cell.column).checked_add(column_delta)?;
            let row = i32::from(cell.row).checked_add(row_delta)?;
            (column >= 0 && column < i32::from(width) && row >= 0 && row < i32::from(height))
                .then(|| CellId::new(column as u16, row as u16))
        })
    })
}
