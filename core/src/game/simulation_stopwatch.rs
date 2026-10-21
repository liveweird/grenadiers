use super::{State, StepError};
use crate::game::{BombTrigger, SeatId, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StopwatchContext {
    pub(super) frozen: [bool; 2],
    pub(super) pause_timed: bool,
}

impl State {
    pub(super) fn stopwatch_context(&self, at: Tick) -> StopwatchContext {
        let holders = SeatId::ALL.map(|seat| {
            let actor = &self.actors[seat.index()];
            actor.alive && actor.effects.stopwatch_active(at)
        });
        StopwatchContext {
            frozen: [
                holders[1] && self.actors[0].alive,
                holders[0] && self.actors[1].alive,
            ],
            pause_timed: holders.into_iter().any(|active| active),
        }
    }

    pub(super) fn preflight_timed_pause(&self, context: StopwatchContext) -> Result<(), StepError> {
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

    pub(super) fn apply_timed_pause(&mut self, context: StopwatchContext) {
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

    pub(super) fn clear_frozen_movement(&mut self, frozen: [bool; 2]) {
        for seat in SeatId::ALL {
            if frozen[seat.index()] {
                let actor = &mut self.actors[seat.index()];
                actor.pending_turn = None;
                actor.residual = 0;
            }
        }
    }
}
