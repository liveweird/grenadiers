use super::{ActorInit, State, StateError, StepError};
use crate::game::{
    EventAudience, RoundAssignment, RulesConfig, SeatId, SimulationSeed, StepEvent, StepEventKind,
    Tick, World,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundActorInit {
    pub actor: ActorInit,
    pub participating: bool,
}

impl State {
    pub fn new_round(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<RoundActorInit>>,
        tick: Tick,
        assignment: RoundAssignment,
        seed: Option<SimulationSeed>,
    ) -> Result<Self, StateError> {
        let actors = actors.into();
        let mut participation = [None; 4];
        let actor_inits: Vec<_> = actors
            .into_iter()
            .map(|init| {
                participation[init.actor.seat.index()] = Some(init.participating);
                init.actor
            })
            .collect();
        let mut state = match seed {
            Some(seed) => Self::new_at_tick_with_assignment_and_seed(
                rules,
                world,
                actor_inits,
                tick,
                assignment,
                seed,
            )?,
            None => Self::new_at_tick_with_assignment(rules, world, actor_inits, tick, assignment)?,
        };
        let participant_count = state
            .player_count
            .seats()
            .iter()
            .filter(|seat| participation[seat.index()] == Some(true))
            .count();
        if participant_count < 2 {
            return Err(StateError::InsufficientParticipants(participant_count));
        }
        for actor in &mut state.actors {
            if participation[actor.seat.index()] != Some(true) {
                actor.kill();
                actor.bomb_grace.clear();
                actor.soft_wall_grace.clear();
                actor.unlit_oil_grace.clear();
            }
        }
        Ok(state)
    }

    pub fn eliminate_departed(&mut self, departed: [bool; 4]) -> Result<Vec<StepEvent>, StepError> {
        for &seat in &SeatId::ALL[self.player_count.get()..] {
            if departed[seat.index()] {
                return Err(StepError::SeatNotParticipating(seat));
            }
        }
        if self.result.is_some() {
            return Ok(Vec::new());
        }
        let audience = EventAudience::all(self.player_count);
        let mut events = Vec::new();
        for &seat in self.player_count.seats() {
            let actor = &mut self.actors[seat.index()];
            if departed[seat.index()] && actor.alive {
                actor.kill();
                events.push(StepEvent::new(
                    self.tick,
                    audience,
                    StepEventKind::ActorEliminated { id: actor.id, seat },
                ));
            }
        }
        if events.is_empty() {
            return Ok(events);
        }
        if let Some(result) = self.decide_result() {
            events.push(StepEvent::new(
                self.tick,
                audience,
                StepEventKind::RoundEnded(result),
            ));
        }
        Ok(events)
    }
}
