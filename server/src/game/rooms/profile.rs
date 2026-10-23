use grenadiers_core::{
    game::{
        ActorInit, Direction, EntityId, GenerationRecord, Position, RoundAssignment, RulesConfig,
        SeatId, SpeedLevel, State, Tick, World, generate_arena,
    },
    protocol::v10::{MapId, RulesVersion},
    room::RoomSettings,
};

use crate::round_seed::RoundSeedSource;

/// Immutable construction and publication identity for the actual room rules.
#[derive(Clone, Copy, Debug)]
pub(super) enum RoomProfile {
    #[cfg(test)]
    FixedClassicV1,
    #[cfg_attr(not(test), allow(dead_code))]
    GeneratedClassicV2,
    #[cfg_attr(not(test), allow(dead_code))]
    GeneratedClassicV3,
    #[cfg_attr(not(test), allow(dead_code))]
    GeneratedConfigurableV4,
    ExpandedV5,
}

pub(super) struct InitialRound {
    pub(super) state: State,
    pub(super) generation_record: Option<GenerationRecord>,
}

impl RoomProfile {
    pub(super) const fn rules_version(self) -> RulesVersion {
        match self {
            #[cfg(test)]
            Self::FixedClassicV1 => RulesVersion::ClassicV1,
            Self::GeneratedClassicV2 => RulesVersion::ClassicV2,
            Self::GeneratedClassicV3 => RulesVersion::ClassicV3,
            Self::GeneratedConfigurableV4 => RulesVersion::ConfigurableV4,
            Self::ExpandedV5 => RulesVersion::ExpandedV5,
        }
    }

    pub(super) const fn map_id(self) -> MapId {
        match self {
            #[cfg(test)]
            Self::FixedClassicV1 => MapId::FixedV1,
            Self::GeneratedClassicV2 | Self::GeneratedClassicV3 | Self::GeneratedConfigurableV4 => {
                MapId::GeneratedV1
            }
            Self::ExpandedV5 => MapId::GeneratedV2,
        }
    }

    pub(super) fn initial_settings(self) -> RoomSettings {
        let settings =
            grenadiers_core::protocol::v10::GameSettings::for_rules(self.rules_version());
        grenadiers_core::protocol::v10::expanded_settings(settings)
            .expect("profile defaults contain supported items")
    }

    pub(super) const fn initial_editor(self) -> Option<SeatId> {
        match self {
            Self::GeneratedConfigurableV4 | Self::ExpandedV5 => Some(SeatId::One),
            #[cfg(test)]
            Self::FixedClassicV1 => None,
            Self::GeneratedClassicV2 | Self::GeneratedClassicV3 => None,
        }
    }

    pub(super) const fn rules_config(self, settings: RoomSettings) -> RulesConfig {
        match self {
            #[cfg(test)]
            Self::FixedClassicV1 => RulesConfig::classic_v1(),
            Self::GeneratedClassicV2 => RulesConfig::classic_v2(),
            Self::GeneratedClassicV3 => RulesConfig::classic_v3(),
            Self::GeneratedConfigurableV4 => RulesConfig::configurable_v4(settings),
            Self::ExpandedV5 => RulesConfig::expanded_v5(settings),
        }
    }

    #[cfg(test)]
    pub(super) fn initial_round(
        self,
        tick: Tick,
        assignment: RoundAssignment,
        round_seed_source: &RoundSeedSource,
    ) -> Option<InitialRound> {
        self.initial_round_with_settings(
            self.initial_settings(),
            tick,
            assignment,
            round_seed_source,
        )
    }

    pub(super) fn initial_round_with_settings(
        self,
        settings: RoomSettings,
        tick: Tick,
        assignment: RoundAssignment,
        round_seed_source: &RoundSeedSource,
    ) -> Option<InitialRound> {
        match self {
            #[cfg(test)]
            Self::FixedClassicV1 => {
                let arena = grenadiers_core::game::fixed_arena();
                let actors = actors_at_assignment_spawns(&arena, assignment, SpeedLevel::Base);
                State::new_at_tick_with_assignment(
                    RulesConfig::classic_v1(),
                    World::new(arena),
                    actors,
                    tick,
                    assignment,
                )
                .ok()
                .map(|state| InitialRound {
                    state,
                    generation_record: None,
                })
            }
            Self::GeneratedClassicV2 | Self::GeneratedClassicV3 | Self::GeneratedConfigurableV4 => {
                let seed = round_seed_source.next_seed().ok()?;
                let (world, generation_record) =
                    World::from_generated(generate_arena(seed)).ok()?;
                let rules = self.rules_config(settings);
                let speed = match self {
                    Self::GeneratedConfigurableV4 => settings.starting_speed(),
                    _ => SpeedLevel::Base,
                };
                let actors = actors_at_assignment_spawns(world.arena(), assignment, speed);
                State::new_at_tick_with_assignment(rules, world, actors, tick, assignment)
                    .ok()
                    .map(|state| InitialRound {
                        state,
                        generation_record: Some(generation_record),
                    })
            }
            Self::ExpandedV5 => {
                let seed = round_seed_source.next_seed().ok()?;
                let (world, generation_record) = World::from_generated(
                    grenadiers_core::game::generate_expanded_arena(seed, settings.enabled_items()),
                )
                .ok()?;
                let actors = actors_at_assignment_spawns(
                    world.arena(),
                    assignment,
                    settings.starting_speed(),
                );
                State::new_at_tick_with_assignment(
                    RulesConfig::expanded_v5(settings),
                    world,
                    actors,
                    tick,
                    assignment,
                )
                .ok()
                .map(|state| InitialRound {
                    state,
                    generation_record: Some(generation_record),
                })
            }
        }
    }
}

fn actors_at_assignment_spawns(
    arena: &grenadiers_core::game::Arena,
    assignment: RoundAssignment,
    speed: SpeedLevel,
) -> [ActorInit; 2] {
    let assigned = assignment.spawns(arena);
    let mut corners = arena.spawns();
    corners.sort();
    SeatId::ALL.map(|seat| {
        let spawn = assigned[seat.index()];
        ActorInit {
            id: EntityId(seat.index() as u32 + 1),
            seat,
            position: Position::center(spawn),
            heading: if spawn == corners[0] {
                Direction::East
            } else {
                Direction::West
            },
            speed,
        }
    })
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
