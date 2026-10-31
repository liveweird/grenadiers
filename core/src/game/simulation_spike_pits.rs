use std::collections::BTreeSet;

use crate::room::{SpikePitConfig, SpikePitSettings};

use super::State;
use super::contract::ForcedMovement;
use crate::game::{
    Arena, CellId, EntityId, MovementTrace, Position, SpikePitClock, SpikePitClockError,
    SpikePitPhase, SpikePitView, StepEvent, Tick, position_contacts_spike_pit,
    segment_contacts_spike_pit, spike_pit_layout_v1, trace_contacts_spike_pit,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SpikePitSite {
    cell: CellId,
    clock: SpikePitClock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SpikePitState {
    settings: SpikePitSettings,
    sites: Vec<SpikePitSite>,
    updated_at: Tick,
}

impl SpikePitState {
    pub(super) fn new(
        config: SpikePitConfig,
        arena: &Arena,
        excluded: impl IntoIterator<Item = CellId>,
        at: Tick,
    ) -> Option<Self> {
        let SpikePitConfig::Enabled(settings) = config else {
            return None;
        };
        let sites = spike_pit_layout_v1(arena, excluded)
            .into_iter()
            .map(|cell| {
                let mut clock = SpikePitClock::new();
                clock
                    .update(at, false, settings)
                    .expect("initial idle spike-pit clocks cannot overflow");
                SpikePitSite { cell, clock }
            })
            .collect();
        Some(Self {
            settings,
            sites,
            updated_at: at,
        })
    }

    pub(super) fn cells(&self) -> impl Iterator<Item = CellId> + '_ {
        self.sites.iter().map(|site| site.cell)
    }

    pub(super) fn non_idle_cells(&self) -> BTreeSet<CellId> {
        self.sites
            .iter()
            .filter_map(|site| (site.clock.phase() != SpikePitPhase::Idle).then_some(site.cell))
            .collect()
    }

    pub(super) fn update(
        &mut self,
        at: Tick,
        contacted: &BTreeSet<CellId>,
    ) -> Result<BTreeSet<CellId>, SpikePitClockError> {
        let mut raised = BTreeSet::new();
        for site in &mut self.sites {
            let phase = site
                .clock
                .update(at, contacted.contains(&site.cell), self.settings)?;
            if phase == SpikePitPhase::Raised {
                raised.insert(site.cell);
            }
        }
        self.updated_at = at;
        Ok(raised)
    }

    pub(super) fn views(&self) -> Vec<SpikePitView> {
        self.sites
            .iter()
            .map(|site| SpikePitView {
                cell: site.cell,
                phase: site.clock.phase(),
                remaining_ticks: site
                    .clock
                    .remaining_ticks(self.updated_at)
                    .expect("state horizon matches every spike-pit clock"),
            })
            .collect()
    }
}

impl State {
    #[cfg(test)]
    pub(crate) fn set_test_spike_pit_cell(&mut self, index: usize, cell: CellId) {
        self.spike_pits
            .as_mut()
            .expect("test enables spike pits")
            .sites[index]
            .cell = cell;
    }

    pub(super) fn advance_spike_pits_at_start(
        &mut self,
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) -> Result<(), super::StepError> {
        let Some(pits) = &self.spike_pits else {
            return Ok(());
        };
        let cells = pits.cells().collect::<Vec<_>>();
        let actor_positions = self
            .actors
            .iter()
            .filter(|actor| actor.alive)
            .map(|actor| actor.position)
            .collect::<Vec<_>>();
        let mob_positions = self
            .lairs
            .as_ref()
            .into_iter()
            .flat_map(|lairs| lairs.mobs())
            .map(|mob| mob.position)
            .collect::<Vec<_>>();
        let contacted = cells
            .iter()
            .copied()
            .filter(|&cell| {
                actor_positions
                    .iter()
                    .chain(&mob_positions)
                    .any(|&position| position_contacts_spike_pit(position, cell))
            })
            .collect();
        let raised = self.update_spike_pits(tick, &contacted)?;
        self.damage_players_on_raised_positions(&raised, tick, events, shield_prevented);
        self.kill_mobs_on_raised_positions(&raised, tick)
    }

    fn update_spike_pits(
        &mut self,
        tick: Tick,
        contacted: &BTreeSet<CellId>,
    ) -> Result<BTreeSet<CellId>, super::StepError> {
        self.spike_pits.as_mut().map_or_else(
            || Ok(BTreeSet::new()),
            |pits| {
                pits.update(tick, contacted)
                    .map_err(super::StepError::SpikePitClock)
            },
        )
    }

    fn damage_players_on_raised_positions(
        &mut self,
        raised: &BTreeSet<CellId>,
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        let hits = self
            .actors
            .iter()
            .map(|actor| {
                actor.alive
                    && raised
                        .iter()
                        .any(|&cell| position_contacts_spike_pit(actor.position, cell))
            })
            .collect::<Vec<_>>();
        for (index, hit) in hits.into_iter().enumerate() {
            if hit {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
    }

    fn kill_mobs_on_raised_positions(
        &mut self,
        raised: &BTreeSet<CellId>,
        tick: Tick,
    ) -> Result<(), super::StepError> {
        let Some(lairs) = &mut self.lairs else {
            return Ok(());
        };
        let killed = lairs
            .mobs()
            .iter()
            .filter(|mob| {
                raised
                    .iter()
                    .any(|&cell| position_contacts_spike_pit(mob.position, cell))
            })
            .map(|mob| mob.id)
            .collect();
        lairs
            .kill_mobs(&killed, tick)
            .map_err(|()| super::StepError::LairTimerOverflow)
    }

    pub(super) fn advance_spike_pits_for_mob_traces(
        &mut self,
        traces: &[(EntityId, MovementTrace)],
        tick: Tick,
    ) -> Result<(), super::StepError> {
        if self.spike_pits.is_none() {
            return Ok(());
        }
        let cells = self
            .spike_pits
            .as_ref()
            .into_iter()
            .flat_map(SpikePitState::cells)
            .collect::<Vec<_>>();
        let contacted = cells
            .iter()
            .copied()
            .filter(|&cell| {
                traces
                    .iter()
                    .any(|(_, trace)| trace_contacts_spike_pit(trace, cell))
            })
            .collect();
        let raised = self.update_spike_pits(tick, &contacted)?;
        let killed = traces
            .iter()
            .filter(|(_, trace)| {
                raised
                    .iter()
                    .any(|&cell| trace_contacts_spike_pit(trace, cell))
            })
            .map(|(id, _)| *id)
            .collect();
        if let Some(lairs) = &mut self.lairs {
            lairs
                .kill_mobs(&killed, tick)
                .map_err(|()| super::StepError::LairTimerOverflow)?;
        }
        Ok(())
    }

    pub(super) fn advance_spike_pits_for_player_traces(
        &mut self,
        traces: &[MovementTrace],
        forced_movements: &[ForcedMovement],
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) -> Result<(), super::StepError> {
        if self.spike_pits.is_none() {
            return Ok(());
        }
        let cells = self
            .spike_pits
            .as_ref()
            .into_iter()
            .flat_map(SpikePitState::cells)
            .collect::<Vec<_>>();
        let actor_contacts = |index: usize, cell: CellId| {
            trace_contacts_spike_pit(&traces[index], cell)
                || forced_movements.iter().any(|forced| {
                    forced.seat.index() == index && segment_contacts_spike_pit(forced.segment, cell)
                })
        };
        let contacted = cells
            .iter()
            .copied()
            .filter(|&cell| {
                self.actors
                    .iter()
                    .enumerate()
                    .any(|(index, actor)| actor.alive && actor_contacts(index, cell))
            })
            .collect();
        let raised = self.update_spike_pits(tick, &contacted)?;
        let hits = self
            .actors
            .iter()
            .enumerate()
            .map(|(index, actor)| {
                actor.alive && raised.iter().any(|&cell| actor_contacts(index, cell))
            })
            .collect::<Vec<_>>();
        for (index, hit) in hits.into_iter().enumerate() {
            if hit {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
        Ok(())
    }

    pub(super) fn advance_spike_pits_for_positions(
        &mut self,
        positions: &[(usize, Position)],
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) -> Result<(), super::StepError> {
        if self.spike_pits.is_none() || positions.is_empty() {
            return Ok(());
        }
        let cells = self
            .spike_pits
            .as_ref()
            .into_iter()
            .flat_map(SpikePitState::cells)
            .collect::<Vec<_>>();
        let contacted = cells
            .iter()
            .copied()
            .filter(|&cell| {
                positions
                    .iter()
                    .any(|(_, position)| position_contacts_spike_pit(*position, cell))
            })
            .collect();
        let raised = self.update_spike_pits(tick, &contacted)?;
        for &(index, position) in positions {
            if self.actors[index].alive
                && raised
                    .iter()
                    .any(|&cell| position_contacts_spike_pit(position, cell))
            {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
        Ok(())
    }

    pub(super) fn advance_spike_pits_for_newborns(
        &mut self,
        ids: &[EntityId],
        tick: Tick,
    ) -> Result<(), super::StepError> {
        if self.spike_pits.is_none() || ids.is_empty() {
            return Ok(());
        }
        let newborns = self
            .lairs
            .as_ref()
            .into_iter()
            .flat_map(|lairs| lairs.mobs())
            .filter(|mob| ids.contains(&mob.id))
            .map(|mob| (mob.id, MovementTrace::new(mob.position)))
            .collect::<Vec<_>>();
        self.advance_spike_pits_for_mob_traces(&newborns, tick)
    }

    pub(super) fn non_idle_spike_pit_cells(&self) -> BTreeSet<CellId> {
        self.spike_pits
            .as_ref()
            .map_or_else(BTreeSet::new, SpikePitState::non_idle_cells)
    }
}
