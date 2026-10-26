use crate::cpu::{
    BombKnowledge, Observation, ObservationError, ObservationParts, ObservedActor, ObservedArena,
    ObservedBomb, ObservedBombTrigger, ObservedEffect, ObservedFlame, ObservedOil, ObservedPickup,
    ObservedRules, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use crate::game;
use crate::protocol::Seat;

use super::{ActorView, BoundPublication, GameState, ProjectionError};

/// Projects the exact v11 recipient view into the transport-neutral CPU contract.
///
/// `world` is authoritative server context: callers bind it to the room instance
/// and physical simulation round. Protocol room counters are not physical-world
/// identities and deliberately are not used here.
pub fn observe_cpu_recipient(
    publication: &BoundPublication,
    recipient: Seat,
    world: WorldId,
) -> Result<Observation, ProjectionError> {
    let state = super::projection_recipient::recipient_state(publication, recipient);
    let tick = game::Tick(state.physical_tick.get());
    let bombs_unavailable = recipient_hides_bombs(&state, recipient)?;
    let settings = super::expanded_settings(publication.room.settings)?;
    let parts = ObservationParts {
        world,
        tick,
        recipient: domain_seat(recipient),
        rules: ObservedRules {
            version: publication.room.rules_version.get(),
            tick_rate: game::RulesConfig::expanded_v5(settings).tick_rate(),
            running_updates: state.running_updates,
            remaining_ticks: state.remaining_ticks,
            settings,
        },
        arena: arena(&state),
        actors: state.actors.map(actor),
        bombs: if bombs_unavailable {
            BombKnowledge::Unavailable
        } else {
            BombKnowledge::Visible(state.bombs.iter().copied().map(bomb).collect())
        },
        flames: state
            .flames
            .iter()
            .copied()
            .map(|flame| {
                Ok(ObservedFlame {
                    cell: cell(flame.cell),
                    remaining_ticks: remaining(flame.expires_at.get(), tick.0)
                        .ok_or(ProjectionError::InvalidFlames)?,
                })
            })
            .collect::<Result<_, _>>()?,
        oil: state.oil_barrels.iter().copied().map(oil).collect(),
        pickups: state.pickups.iter().copied().map(pickup).collect(),
    };
    Observation::new(parts).map_err(observation_error)
}

fn recipient_hides_bombs(state: &GameState, recipient: Seat) -> Result<bool, ProjectionError> {
    let Some(actor) = state
        .actors
        .iter()
        .find(|actor| actor_seat(actor) == recipient)
    else {
        return Err(ProjectionError::InvalidActors);
    };
    match actor {
        ActorView::Visible { effects, .. } => Ok(effects
            .as_slice()
            .iter()
            .any(|effect| effect.kind() == super::EffectKind::InvisibleBombs)),
        ActorView::Hidden { .. } => Err(ProjectionError::InvalidActors),
    }
}

fn actor_seat(actor: &ActorView) -> Seat {
    match actor {
        ActorView::Visible { seat, .. } | ActorView::Hidden { seat, .. } => *seat,
    }
}

fn arena(state: &GameState) -> ObservedArena {
    ObservedArena {
        width: u16::from(state.arena.width),
        height: u16::from(state.arena.height),
        terrain: state
            .arena
            .terrain
            .iter()
            .copied()
            .map(domain_terrain)
            .collect(),
        blocks: state.arena.blocks.iter().copied().map(cell).collect(),
    }
}

fn actor(actor: ActorView) -> ObservedActor {
    match actor {
        ActorView::Hidden { id, seat, alive } => ObservedActor::Hidden {
            id: entity(id),
            seat: domain_seat(seat),
            alive,
        },
        ActorView::Visible {
            id,
            seat,
            position,
            heading,
            speed_level,
            alive,
            bomb_capacity,
            bomb_range,
            live_bomb_count,
            inventory,
            effects,
            damage_grace_remaining_ticks,
            voluntary_actions_frozen,
            last_teleport_tick,
        } => ObservedActor::Visible(VisibleActor {
            id: entity(id),
            seat: domain_seat(seat),
            position: game::Position {
                x: i32::from(position.x()),
                y: i32::from(position.y()),
            },
            heading: domain_direction(heading),
            speed: domain_speed(speed_level),
            alive,
            bomb_capacity: u16::from(bomb_capacity),
            bomb_range: u16::from(bomb_range),
            live_bomb_count: u16::from(live_bomb_count),
            equipment: inventory.equipment.map(domain_equipment),
            weapon: inventory.weapon.map(domain_weapon),
            effects: effects
                .as_slice()
                .iter()
                .copied()
                .map(|effect| ObservedEffect {
                    kind: domain_effect(effect.kind()),
                    remaining_ticks: effect.remaining_ticks(),
                })
                .collect(),
            damage_grace_remaining_ticks: damage_grace_remaining_ticks.get(),
            voluntary_actions_frozen,
            last_teleport_tick: last_teleport_tick.map(|value| game::Tick(value.get())),
            unknown: UnknownActorFacts::ALL,
        }),
    }
}

fn bomb(value: super::BombView) -> ObservedBomb {
    let trigger = match value.trigger.observation() {
        crate::protocol::v8::BombTriggerObservation::Timed {
            remaining_ticks,
            paused,
        } => ObservedBombTrigger::Timed {
            remaining_ticks,
            paused,
        },
        crate::protocol::v8::BombTriggerObservation::Remote => ObservedBombTrigger::Remote,
        crate::protocol::v8::BombTriggerObservation::Contact => ObservedBombTrigger::Contact,
    };
    ObservedBomb {
        id: entity(value.id),
        cell: cell(value.cell),
        owner: entity(value.owner),
        range: u16::from(value.range),
        trigger,
        blast: match value.blast {
            super::BlastKind::Standard => game::BlastKind::Standard,
            super::BlastKind::Piercing => game::BlastKind::Piercing,
            super::BlastKind::Power => game::BlastKind::Power,
        },
        unknown: UnknownBombFacts::ALL,
    }
}

fn oil(value: super::OilBarrelView) -> ObservedOil {
    ObservedOil {
        cell: cell(value.cell),
        state: match value.state {
            super::OilBarrelState::Unlit => game::OilBarrelState::Unlit,
            super::OilBarrelState::Burning => game::OilBarrelState::Burning,
        },
    }
}

fn pickup(value: super::PickupView) -> ObservedPickup {
    ObservedPickup {
        cell: cell(value.cell),
        kind: super::cpu_observation_types::domain_pickup(value.kind),
    }
}

fn remaining(deadline: u64, now: u64) -> Option<u16> {
    u16::try_from(deadline.checked_sub(now)?).ok()
}

fn cell(value: crate::protocol::Cell) -> game::CellId {
    game::CellId::new(u16::from(value.column()), u16::from(value.row()))
}

fn entity(value: crate::protocol::EntityId) -> game::EntityId {
    game::EntityId(value.get())
}

const fn domain_seat(value: Seat) -> game::SeatId {
    match value {
        Seat::One => game::SeatId::One,
        Seat::Two => game::SeatId::Two,
    }
}

const fn domain_direction(value: crate::protocol::Direction) -> game::Direction {
    match value {
        crate::protocol::Direction::North => game::Direction::North,
        crate::protocol::Direction::East => game::Direction::East,
        crate::protocol::Direction::South => game::Direction::South,
        crate::protocol::Direction::West => game::Direction::West,
    }
}

const fn domain_speed(value: u8) -> game::SpeedLevel {
    match value {
        0 => game::SpeedLevel::Base,
        1 => game::SpeedLevel::One,
        2 => game::SpeedLevel::Two,
        3 => game::SpeedLevel::Three,
        4 => game::SpeedLevel::Four,
        _ => unreachable!(),
    }
}

const fn domain_terrain(value: super::Terrain) -> game::Terrain {
    match value {
        super::Terrain::Floor => game::Terrain::Floor,
        super::Terrain::Wall => game::Terrain::Wall,
    }
}

const fn domain_equipment(value: super::EquipmentId) -> game::Equipment {
    match value {
        super::EquipmentId::Kick => game::Equipment::Kick,
        super::EquipmentId::Punch => game::Equipment::Punch,
        super::EquipmentId::Bracelet => game::Equipment::Bracelet,
        super::EquipmentId::WallPass => game::Equipment::WallPass,
        super::EquipmentId::Heart => game::Equipment::Heart,
    }
}

const fn domain_weapon(value: super::WeaponId) -> game::SpecialWeapon {
    match value {
        super::WeaponId::RemoteBomb => game::SpecialWeapon::Remote,
        super::WeaponId::PiercingBomb => game::SpecialWeapon::Piercing,
        super::WeaponId::PowerBomb => game::SpecialWeapon::Power,
        super::WeaponId::Landmine => game::SpecialWeapon::Landmine,
    }
}

fn domain_effect(value: super::EffectKind) -> game::EffectKind {
    super::cpu_observation_types::domain_effect(value)
}

const fn observation_error(value: ObservationError) -> ProjectionError {
    match value {
        ObservationError::InvalidClock => ProjectionError::InvalidClock,
        ObservationError::InvalidArena => ProjectionError::InvalidArena,
        ObservationError::InvalidActors | ObservationError::InvalidEffects => {
            ProjectionError::InvalidActors
        }
        ObservationError::InvalidBombs => ProjectionError::InvalidBombs,
        ObservationError::InvalidFlames | ObservationError::InvalidOil => {
            ProjectionError::InvalidFlames
        }
        ObservationError::InvalidPickups => ProjectionError::InvalidPickups,
    }
}
