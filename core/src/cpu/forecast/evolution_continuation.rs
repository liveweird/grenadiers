use crate::cpu::{BombKnowledge, Observation, WorkBudget};

use super::ForecastEvolution;

impl ForecastEvolution {
    /// Charges the complete observed-bomb scene comparison before traversing it.
    /// Continuation starts without finite flames or Oil; predicted later flames
    /// and bomb removals remain valid, while foreign changes fail scene matching.
    pub(crate) fn continuation_scene_matches(
        &self,
        basis: &Observation,
        current: &Observation,
        work: &mut WorkBudget,
    ) -> bool {
        let Some(units) = self.continuation_scene_work(basis, current) else {
            return false;
        };
        work.try_take(units).is_ok() && self.matches_scene(basis, current)
    }

    fn continuation_scene_work(&self, basis: &Observation, current: &Observation) -> Option<u32> {
        let old = basis.parts();
        let new = current.parts();
        let (BombKnowledge::Visible(old_bombs), BombKnowledge::Visible(new_bombs)) =
            (&old.bombs, &new.bombs)
        else {
            return None;
        };
        if !old.flames.is_empty()
            || !old.oil.is_empty()
            || !new.oil.is_empty()
            || new_bombs.len() > old_bombs.len()
        {
            return None;
        }
        let blocks = old.arena.blocks.len().checked_add(new.arena.blocks.len())?;
        let block_removals = old
            .arena
            .blocks
            .len()
            .checked_mul(self.block_removals.len())?;
        // bombs_match filters the old set twice through removed(), then scans
        // current candidates for each survivor. No hypothetical ID mapping is used.
        let removals = self.bomb_removals.len().checked_add(1)?;
        let bomb_filters = old_bombs.len().checked_mul(removals)?.checked_mul(2)?;
        let bomb_pairs = old_bombs
            .len()
            .checked_mul(new_bombs.len())?
            .checked_mul(2)?;
        let units = 32_usize
            .checked_add(old.arena.terrain.len().max(new.arena.terrain.len()))?
            .checked_add(old.pickups.len().checked_add(new.pickups.len())?)?
            .checked_add(blocks.checked_mul(2)?)?
            .checked_add(block_removals)?
            .checked_add(bomb_filters)?
            .checked_add(bomb_pairs)?
            .checked_add(new_bombs.len().checked_mul(2)?)?
            .checked_add(self.flame_changes.len().checked_mul(4)?)?
            .checked_add(new.flames.len().checked_mul(3)?)?;
        u32::try_from(units).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::forecast::tests::{observation, run, timed};
    use crate::cpu::forecast::{ForecastLimits, HazardForecast};
    use crate::cpu::{BombKnowledge, Observation, ObservedBombTrigger, ObservedFlame, WorkBudget};
    use crate::game::{BlastKind, CellId, Tick};

    fn fixture() -> (Observation, HazardForecast) {
        let basis = observation(
            0,
            BombKnowledge::Visible(vec![
                timed(1, CellId::new(5, 1), 100, BlastKind::Standard),
                timed(2, CellId::new(9, 5), 180, BlastKind::Standard),
            ]),
        );
        let forecast = run(&basis, ForecastLimits::default()).unwrap();
        (basis, forecast)
    }

    fn current_at(basis: &Observation, forecast: &HazardForecast, tick: u16) -> Observation {
        let mut parts = basis.parts().clone();
        parts.tick = Tick(u64::from(tick));
        parts.rules.running_updates = tick;
        parts.rules.remaining_ticks -= tick;
        let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
            unreachable!()
        };
        bombs.retain_mut(|bomb| {
            let ObservedBombTrigger::Timed {
                remaining_ticks, ..
            } = &mut bomb.trigger
            else {
                unreachable!()
            };
            if *remaining_ticks <= tick {
                return false;
            }
            *remaining_ticks -= tick;
            true
        });
        parts.flames = forecast
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(index, cell)| {
                let window = cell
                    .lethal
                    .iter()
                    .find(|window| window.start <= parts.tick && parts.tick < window.end)?;
                Some(ObservedFlame {
                    cell: CellId::new((index % 15) as u16, (index / 15) as u16),
                    remaining_ticks: u16::try_from(window.end.0 - parts.tick.0).unwrap(),
                })
            })
            .collect();
        Observation::new(parts).unwrap()
    }

    #[test]
    fn charged_continuation_scene_accepts_two_bombs_and_exact_allowance() {
        let (basis, forecast) = fixture();
        let current = current_at(&basis, &forecast, 10);
        let mut work = WorkBudget::new(8_192);
        let evolution = forecast.evolution();
        assert!(evolution.continuation_scene_matches(&basis, &current, &mut work));
        let required = work.spent();
        assert!((1..=8_192).contains(&required));
        assert!(evolution.continuation_scene_matches(
            &basis,
            &current,
            &mut WorkBudget::new(required),
        ));
        assert!(!evolution.continuation_scene_matches(
            &basis,
            &current,
            &mut WorkBudget::new(required - 1),
        ));

        for change in 0..3 {
            let mut changed = current.parts().clone();
            let BombKnowledge::Visible(bombs) = &mut changed.bombs else {
                unreachable!()
            };
            match change {
                0 => {
                    bombs.remove(0);
                }
                1 => bombs[0].cell = CellId::new(6, 1),
                2 => bombs[0].id = crate::game::EntityId(99),
                _ => unreachable!(),
            }
            assert!(!evolution.continuation_scene_matches(
                &basis,
                &Observation::new(changed).unwrap(),
                &mut WorkBudget::new(8_192),
            ));
        }
    }

    #[test]
    fn charged_continuation_scene_accepts_predicted_detonation_and_flame_expiry() {
        let (basis, forecast) = fixture();
        for tick in [101, 181, 211] {
            let current = current_at(&basis, &forecast, tick);
            assert!(forecast.evolution().continuation_scene_matches(
                &basis,
                &current,
                &mut WorkBudget::new(8_192),
            ));
        }
        let mut changed = current_at(&basis, &forecast, 10).into_parts();
        changed.flames.push(ObservedFlame {
            cell: CellId::new(1, 1),
            remaining_ticks: 5,
        });
        assert!(!forecast.evolution().continuation_scene_matches(
            &basis,
            &Observation::new(changed).unwrap(),
            &mut WorkBudget::new(8_192),
        ));
    }
}
