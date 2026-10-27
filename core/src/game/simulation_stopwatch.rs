use super::{State, StepError};
use crate::game::{BombTrigger, Tick};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StopwatchContext {
    pub(super) frozen: Vec<bool>,
    pub(super) pause_timed: bool,
}

impl State {
    pub(super) fn stopwatch_context(&self, at: Tick) -> StopwatchContext {
        let holders: Vec<_> = self
            .player_count
            .seats()
            .iter()
            .map(|&seat| {
                let actor = &self.actors[seat.index()];
                actor.alive && actor.effects.stopwatch_active(at)
            })
            .collect();
        StopwatchContext {
            frozen: (0..self.actors.len())
                .map(|index| {
                    holders
                        .iter()
                        .enumerate()
                        .any(|(holder, &active)| holder != index && active)
                        && self.actors[index].alive
                })
                .collect(),
            pause_timed: holders.iter().any(|&active| active),
        }
    }

    pub(super) fn preflight_timed_pause(
        &self,
        context: &StopwatchContext,
    ) -> Result<(), StepError> {
        if !context.pause_timed {
            return Ok(());
        }
        for bomb in self.world.bombs.values() {
            if matches!(bomb.trigger(), BombTrigger::Timed { .. }) {
                let mut checked = *bomb;
                checked
                    .pause_one_tick()
                    .map_err(|_| StepError::BombPauseOverflow(bomb.id()))?;
            }
        }
        Ok(())
    }

    pub(super) fn apply_timed_pause(&mut self, context: &StopwatchContext) {
        if !context.pause_timed {
            return;
        }
        for bomb in self.world.bombs.values_mut() {
            if matches!(bomb.trigger(), BombTrigger::Timed { .. }) {
                bomb.pause_one_tick()
                    .expect("timed pause was preflight before step mutation");
            }
        }
    }

    pub(super) fn clear_frozen_movement(&mut self, frozen: &[bool]) {
        for &seat in self.player_count.seats() {
            if frozen[seat.index()] {
                let actor = &mut self.actors[seat.index()];
                actor.pending_turn = None;
                actor.residual = 0;
            }
        }
    }
}
