use grenadiers_core::{
    game::{
        ActorInit, Direction, EntityId, Position, RulesConfig, SeatId, SpeedLevel, State, Tick,
        World, fixed_arena,
    },
    protocol::{MapId, RulesVersion},
};

/// Immutable construction and publication identity for the actual room rules.
#[derive(Clone, Copy, Debug)]
pub(super) enum RoomProfile {
    FixedClassicV1,
}

impl RoomProfile {
    pub(super) const fn rules_version(self) -> RulesVersion {
        match self {
            Self::FixedClassicV1 => RulesVersion::ClassicV1,
        }
    }

    pub(super) const fn map_id(self) -> MapId {
        match self {
            Self::FixedClassicV1 => MapId::FixedV1,
        }
    }

    pub(super) fn initial_state(self, tick: Tick) -> Option<State> {
        match self {
            Self::FixedClassicV1 => {
                let arena = fixed_arena();
                let spawns = arena.spawns();
                let actors = SeatId::ALL.map(|seat| ActorInit {
                    id: EntityId(seat.index() as u32 + 1),
                    seat,
                    position: Position::center(spawns[seat.index()]),
                    heading: if seat == SeatId::One {
                        Direction::East
                    } else {
                        Direction::West
                    },
                    speed: SpeedLevel::Base,
                });
                State::new_at_tick(RulesConfig::classic_v1(), World::new(arena), actors, tick).ok()
            }
        }
    }
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
