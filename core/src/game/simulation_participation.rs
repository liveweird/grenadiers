use super::{ActorInit, DrawReason, RoundResult, State, StateError, StepError};
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
    pub(super) fn decide_result(&mut self) -> Option<RoundResult> {
        let mut living = self.actors.iter().filter(|actor| actor.alive);
        let first = living.next().map(|actor| actor.seat);
        let second = living.next();
        self.result = match (first, second) {
            (None, _) => Some(RoundResult::Draw(DrawReason::MutualElimination)),
            (Some(winner), None) => Some(RoundResult::Winner(winner)),
            (Some(_), Some(_)) if self.running_updates >= self.rules.round_ticks() => {
                Some(RoundResult::Draw(DrawReason::Timeout))
            }
            (Some(_), Some(_)) => None,
        };
        if self.result.is_some() {
            self.clear_storm_slow();
        }
        self.result
    }

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
