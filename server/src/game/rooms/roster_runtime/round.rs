use grenadiers_core::game::{
    ActorInit, ArenaGenerationConfig, Direction, EntityId, Position, RoundActorInit,
    RoundAssignment, SeatId, State, World,
};

use super::{
    error::RosterRuntimeError,
    runtime::{RosterRuntime, SimRound},
};

impl RosterRuntime {
    pub(super) fn start_round(
        &mut self,
        assignment: RoundAssignment,
    ) -> Result<(), RosterRuntimeError> {
        self.settings_locked = true;
        let generation_seed = self
            .round_seed_source
            .next_seed()
            .map_err(|_| RosterRuntimeError::SeedUnavailable)?;
        let config = ArenaGenerationConfig {
            player_count: self.identity.configuration.player_count(),
            board: self.identity.configuration.board(),
            assignment,
        };
        let lairs_enabled = match self.environment_profile {
            Some(super::runtime::EnvironmentProfile::LairsV8 { tuning, .. }) => tuning.enabled(),
            Some(super::runtime::EnvironmentProfile::RespawnV9 { tuning, .. }) => tuning.enabled(),
            Some(super::runtime::EnvironmentProfile::SpikeV10 { lairs, .. }) => lairs.enabled(),
            Some(super::runtime::EnvironmentProfile::StormV7(_)) | None => false,
        };
        let generated = self
            .construction_profile
            .generate(
                generation_seed,
                self.lifecycle.settings().enabled_items(),
                config,
                self.lifecycle.settings().bomb_capacity_drop_multiplier(),
                lairs_enabled,
            )
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
        let rules = self
            .environment_profile
            .map_or_else(
                || grenadiers_core::game::RulesConfig::multiplayer_v6(self.lifecycle.settings()),
                |profile| match profile {
                    super::runtime::EnvironmentProfile::StormV7(environment) => {
                        grenadiers_core::game::RulesConfig::environment_v7(
                            self.lifecycle.settings(),
                            environment,
                        )
                    }
                    super::runtime::EnvironmentProfile::LairsV8 { cloud, tuning } => {
                        grenadiers_core::game::RulesConfig::environment_v8(
                            self.lifecycle.settings(),
                            cloud,
                            tuning.config(),
                        )
                    }
                    super::runtime::EnvironmentProfile::RespawnV9 { cloud, tuning } => {
                        grenadiers_core::game::RulesConfig::environment_v9_with_mob_navigation(
                            self.lifecycle.settings(),
                            cloud,
                            tuning.config(),
                            self.mob_navigation,
                        )
                    }
                    super::runtime::EnvironmentProfile::SpikeV10 {
                        cloud,
                        lairs,
                        spike_pits,
                    } => grenadiers_core::game::RulesConfig::environment_v10_with_mob_navigation(
                        self.lifecycle.settings(),
                        cloud,
                        lairs.config(),
                        spike_pits.config(),
                        self.mob_navigation,
                    ),
                },
            )
            .with_punch_activation_policy(self.punch_activation)
            .with_spike_pit_layout_policy(self.construction_profile.spike_pit_layout_policy());
        let state = State::new_round(
            rules,
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
        self.environment_cpu_publication = None;
        self.lair_cpu_publication = None;
        self.respawn_cpu_publication = None;
        self.spike_cpu_publication = None;
        self.spike_v19_cpu_publication = None;
        self.spike_v20_cpu_publication = None;
        self.spike_v21_cpu_publication = None;
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
