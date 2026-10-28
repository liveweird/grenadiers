use grenadiers_core::game::{
    ActorInit, ArenaGenerationConfig, Direction, EntityId, Position, RoundActorInit,
    RoundAssignment, SeatId, State, World, generate_multiplayer_arena,
    generate_variable_multiplayer_arena,
};
use grenadiers_core::room::BoardSize;

use super::{
    error::RosterRuntimeError,
    runtime::{RosterRuntime, SimRound},
};

impl RosterRuntime {
    pub(super) fn start_round(
        &mut self,
        assignment: RoundAssignment,
    ) -> Result<(), RosterRuntimeError> {
        let generation_seed = self
            .round_seed_source
            .next_seed()
            .map_err(|_| RosterRuntimeError::SeedUnavailable)?;
        let config = ArenaGenerationConfig {
            player_count: self.identity.configuration.player_count(),
            board: self.identity.configuration.board(),
            assignment,
        };
        let generated = if config.board == BoardSize::FifteenByThirteen {
            generate_multiplayer_arena(
                generation_seed,
                self.lifecycle.settings().enabled_items(),
                config,
            )
        } else {
            generate_variable_multiplayer_arena(
                generation_seed,
                self.lifecycle.settings().enabled_items(),
                config,
            )
        }
        .map_err(|_| RosterRuntimeError::UnsupportedConfiguration)?;
        let spawns = assignment.spawns(generated.arena());
        let actors = self
            .identity
            .configuration
            .player_count()
            .seats()
            .iter()
            .copied()
            .map(|seat| RoundActorInit {
                actor: ActorInit {
                    id: EntityId(u32::try_from(seat.index() + 1).unwrap()),
                    seat,
                    position: Position::center(spawns[seat.index()]),
                    heading: Direction::East,
                    speed: self.lifecycle.settings().starting_speed(),
                },
                participating: self
                    .lifecycle
                    .eligible_seats()
                    .any(|eligible| eligible == seat),
            })
            .collect::<Vec<_>>();
        let (world, generation) =
            World::from_generated(generated).map_err(|_| RosterRuntimeError::Publication)?;
        let epoch = self.lifecycle.round_attempt();
        let simulation_seed =
            (self.simulation_seed)(epoch).map_err(|()| RosterRuntimeError::SeedUnavailable)?;
        let state = State::new_round(
            grenadiers_core::game::RulesConfig::multiplayer_v6(self.lifecycle.settings()),
            world,
            actors,
            self.lifecycle.tick(),
            assignment,
            Some(simulation_seed),
        )
        .map_err(|_| RosterRuntimeError::Publication)?;
        self.inputs.reset_round();
        self.reports.clear();
        self.cpu_publication = None;
        self.sim = Some(SimRound {
            epoch,
            state,
            generation,
        });
        Ok(())
    }

    pub(super) fn eliminate_departed(
        &mut self,
        eliminated: [bool; 4],
    ) -> Result<(), RosterRuntimeError> {
        let Some(sim) = self.sim.as_mut() else {
            return Ok(());
        };
        let events = sim
            .state
            .eliminate_departed(eliminated)
            .map_err(|_| RosterRuntimeError::Publication)?;
        if !events.is_empty() {
            let report = self
                .reports
                .last_mut()
                .ok_or(RosterRuntimeError::Publication)?;
            report.events.extend(events);
        }
        Ok(())
    }
}

pub(super) fn mark_eliminated(target: &mut [bool; 4], seats: impl Iterator<Item = SeatId>) {
    for seat in seats {
        target[seat.index()] = true;
    }
}
