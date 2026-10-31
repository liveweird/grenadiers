use crate::cpu::{Observation, WorkBudget};
use crate::game::{
    CellId, MovementSegment, Position, SpikePitClock, SpikePitPhase, SpikePitView, Tick,
    position_contacts_spike_pit, segment_contacts_spike_pit,
};
use crate::room::{SpikePitConfig, SpikePitSettings};

use super::RouteFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpikePitGuard {
    at: Tick,
    config: SpikePitConfig,
    expected: Vec<SpikePitView>,
}

impl SpikePitGuard {
    #[must_use]
    pub const fn at(&self) -> Tick {
        self.at
    }

    #[must_use]
    pub fn expected(&self) -> &[SpikePitView] {
        &self.expected
    }

    #[must_use]
    pub const fn config(&self) -> SpikePitConfig {
        self.config
    }

    pub fn matches(
        &self,
        observation: &Observation,
        work: &mut WorkBudget,
    ) -> Result<bool, RouteFailure> {
        let actual_config = observation
            .spike_pits()
            .map_or(SpikePitConfig::Disabled, |pits| pits.config());
        let actual = observation.spike_pits().map_or(&[][..], |pits| pits.pits());
        let units = self.expected.len().max(actual.len()).saturating_add(1);
        work.try_take(u32::try_from(units).map_err(|_| RouteFailure::Capacity)?)
            .map_err(|_| RouteFailure::WorkExhausted)?;
        Ok(observation.parts().tick == self.at
            && actual_config == self.config
            && actual == self.expected)
    }
}

#[derive(Clone)]
pub(super) struct PitTrajectory {
    settings: Option<SpikePitSettings>,
    sites: Vec<PitSite>,
    at: Tick,
    protected_until: Option<Tick>,
}

#[derive(Clone)]
struct PitSite {
    cell: CellId,
    clock: SpikePitClock,
}

impl PitTrajectory {
    pub(super) fn from_observation(
        observation: &Observation,
        work: &mut WorkBudget,
    ) -> Result<Self, RouteFailure> {
        let at = observation.parts().tick;
        let Some(observed) = observation.spike_pits() else {
            return Ok(Self {
                settings: None,
                sites: Vec::new(),
                at,
                protected_until: None,
            });
        };
        let SpikePitConfig::Enabled(settings) = observed.config() else {
            return Err(RouteFailure::InvalidRequest);
        };
        work.try_take(u32::try_from(observed.pits().len()).map_err(|_| RouteFailure::Capacity)?)
            .map_err(|_| RouteFailure::WorkExhausted)?;
        let sites = observed
            .pits()
            .iter()
            .map(|pit| {
                SpikePitClock::from_observation(
                    pit.phase,
                    pit.remaining_ticks,
                    observation.parts().tick,
                    settings,
                )
                .map(|clock| PitSite {
                    cell: pit.cell,
                    clock,
                })
                .map_err(|_| RouteFailure::InvalidRequest)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            settings: Some(settings),
            sites,
            at,
            protected_until: damage_protected_until(observation),
        })
    }

    pub(super) fn advance(
        &mut self,
        tick: Tick,
        start: Position,
        segments: &[MovementSegment],
        work: &mut WorkBudget,
    ) -> Result<(), RouteFailure> {
        if tick <= self.at {
            return Err(RouteFailure::InvalidRequest);
        }
        let Some(settings) = self.settings else {
            self.at = tick;
            return Ok(());
        };
        for site in &mut self.sites {
            charge(work, 1)?; // site
            let standing = position_contacts_spike_pit(start, site.cell);
            let swept = segments
                .iter()
                .any(|segment| segment_contacts_spike_pit(*segment, site.cell));
            if standing || swept {
                charge(work, 1)?; // contact
            }
            if site
                .clock
                .next_deadline()
                .is_some_and(|deadline| deadline <= tick)
            {
                charge(work, 1)?; // deadline
            }
            let phase = site
                .clock
                .update(tick, standing || swept, settings)
                .map_err(|_| RouteFailure::InvalidRequest)?;
            if phase == SpikePitPhase::Raised
                && (standing || swept)
                && self.protected_until.is_none_or(|expiry| tick >= expiry)
            {
                return Err(RouteFailure::NoRoute);
            }
        }
        self.at = tick;
        Ok(())
    }

    /// Advances a fixed pose through `[at + 1, end)` by visiting only contact and
    /// phase-deadline events. This keeps long terminal suffixes bounded.
    pub(super) fn stationary_until(
        &mut self,
        end: Tick,
        position: Position,
        work: &mut WorkBudget,
    ) -> Result<(), RouteFailure> {
        let Some(last) = end.0.checked_sub(1).map(Tick) else {
            return Err(RouteFailure::InvalidRequest);
        };
        if last <= self.at {
            return Ok(());
        }
        let segment = MovementSegment::new(position, position);
        let mut next = Tick(
            self.at
                .0
                .checked_add(1)
                .ok_or(RouteFailure::InvalidRequest)?,
        );
        while next <= last {
            let contacted = |site: &PitSite| position_contacts_spike_pit(position, site.cell);
            let contact_needs_update = self
                .sites
                .iter()
                .any(|site| site.clock.phase() == SpikePitPhase::Idle && contacted(site));
            let first_unsafe = self.sites.iter().any(|site| {
                site.clock.phase() == SpikePitPhase::Raised
                    && contacted(site)
                    && self.protected_until.is_none_or(|expiry| next >= expiry)
            });
            let deadline = self
                .sites
                .iter()
                .filter_map(|site| site.clock.next_deadline())
                .filter(|deadline| *deadline >= next && *deadline <= last)
                .min();
            let protection_expiry = self
                .protected_until
                .filter(|expiry| *expiry >= next && *expiry <= last)
                .filter(|_| {
                    self.sites
                        .iter()
                        .any(|site| site.clock.phase() == SpikePitPhase::Raised && contacted(site))
                });
            let event = if contact_needs_update || first_unsafe {
                next
            } else {
                deadline
                    .into_iter()
                    .chain(protection_expiry)
                    .min()
                    .unwrap_or(last)
            };
            self.advance(event, position, std::slice::from_ref(&segment), work)?;
            next = Tick(event.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?);
        }
        Ok(())
    }

    pub(super) fn guard(&self) -> Result<SpikePitGuard, RouteFailure> {
        let expected = self
            .sites
            .iter()
            .map(|site| {
                Ok(SpikePitView {
                    cell: site.cell,
                    phase: site
                        .clock
                        .phase_at(self.at)
                        .map_err(|_| RouteFailure::InvalidRequest)?,
                    remaining_ticks: site
                        .clock
                        .remaining_ticks(self.at)
                        .map_err(|_| RouteFailure::InvalidRequest)?,
                })
            })
            .collect::<Result<Vec<_>, RouteFailure>>()?;
        Ok(SpikePitGuard {
            at: self.at,
            config: self
                .settings
                .map_or(SpikePitConfig::Disabled, SpikePitConfig::Enabled),
            expected,
        })
    }

    pub(super) fn state_key(&self) -> Vec<(CellId, u8, Option<u64>, u64)> {
        self.sites
            .iter()
            .map(|site| {
                let phase = match site.clock.phase() {
                    SpikePitPhase::Idle => 0,
                    SpikePitPhase::Warning => 1,
                    SpikePitPhase::Raised => 2,
                };
                (
                    site.cell,
                    phase,
                    site.clock.next_deadline().map(|tick| tick.0),
                    self.at.0,
                )
            })
            .collect()
    }

    pub(super) fn wait_boundaries(&self, work: &mut WorkBudget) -> Result<Vec<Tick>, RouteFailure> {
        let mut boundaries = Vec::new();
        for deadline in self
            .sites
            .iter()
            .filter_map(|site| site.clock.next_deadline())
        {
            charge(work, 1)?;
            boundaries.push(deadline);
            if let Some(rearm) = deadline.0.checked_add(1).map(Tick) {
                boundaries.push(rearm);
            }
        }
        Ok(boundaries)
    }
}

fn damage_protected_until(observation: &Observation) -> Option<Tick> {
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor)
                if actor.seat == observation.parts().recipient =>
            {
                Some(actor)
            }
            _ => None,
        })?;
    let basis = observation.parts().tick.0;
    actor
        .effects
        .iter()
        .filter(|effect| effect.kind == crate::game::EffectKind::Shield)
        .map(|effect| u64::from(effect.remaining_ticks))
        .chain(std::iter::once(u64::from(
            actor.damage_grace_remaining_ticks,
        )))
        .filter(|remaining| *remaining > 0)
        .filter_map(|remaining| basis.checked_add(remaining).map(Tick))
        .max()
}

fn charge(work: &mut WorkBudget, units: u32) -> Result<(), RouteFailure> {
    work.try_take(units)
        .map_err(|_| RouteFailure::WorkExhausted)
}

#[cfg(test)]
#[path = "spike_pit_trajectory_tests.rs"]
mod tests;
