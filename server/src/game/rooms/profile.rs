use grenadiers_core::{
    game::{
        ActorInit, Direction, EntityId, GenerationRecord, Position, RoundAssignment, RulesConfig,
        SeatId, SpeedLevel, State, Tick, World, generate_arena,
    },
    protocol::{MapId, RulesVersion},
};

use crate::round_seed::RoundSeedSource;

/// Immutable construction and publication identity for the actual room rules.
#[derive(Clone, Copy, Debug)]
pub(super) enum RoomProfile {
    #[cfg(test)]
    FixedClassicV1,
    GeneratedClassicV2,
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
        }
    }

    pub(super) const fn map_id(self) -> MapId {
        match self {
            #[cfg(test)]
            Self::FixedClassicV1 => MapId::FixedV1,
            Self::GeneratedClassicV2 => MapId::GeneratedV1,
        }
    }

    pub(super) fn initial_round(
        self,
        tick: Tick,
        round_seed_source: &RoundSeedSource,
    ) -> Option<InitialRound> {
        match self {
            #[cfg(test)]
            Self::FixedClassicV1 => {
                let arena = grenadiers_core::game::fixed_arena();
                let assignment = RoundAssignment::first();
                let actors = actors_at_assignment_spawns(&arena, assignment);
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
            Self::GeneratedClassicV2 => {
                let seed = round_seed_source.next_seed().ok()?;
                let (world, generation_record) =
                    World::from_generated(generate_arena(seed)).ok()?;
                let assignment = RoundAssignment::first();
                let actors = actors_at_assignment_spawns(world.arena(), assignment);
                State::new_at_tick_with_assignment(
                    RulesConfig::classic_v2(),
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
            speed: SpeedLevel::Base,
        }
    })
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
