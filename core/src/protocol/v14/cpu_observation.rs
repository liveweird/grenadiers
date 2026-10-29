use crate::cpu::{
    BombKnowledge, Observation, ObservationError, ObservationParts, ObservedActor, ObservedArena,
    ObservedBomb, ObservedBombTrigger, ObservedEffect, ObservedFlame, ObservedOil, ObservedPickup,
    ObservedRules, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use crate::{game, protocol};

use super::{ActorView, BoundPublication, GameState, ProjectionError, Seat};

pub fn observe_cpu_recipient(
    publication: &BoundPublication,
    recipient: Seat,
    world: WorldId,
) -> Result<Observation, ProjectionError> {
    let projected = super::project_recipient(publication, recipient)?;
    let state = projected.state;
    let settings = crate::protocol::v11::expanded_settings(publication.room.settings)
        .map_err(|_| ProjectionError::InvalidRules)?;
    let rules = match publication.room.rules_version.get() {
        6 => game::RulesConfig::multiplayer_v6(settings),
        _ => game::RulesConfig::expanded_v5(settings),
    };
    Observation::new(observation_parts(
        state,
        recipient,
        world,
        publication.room.rules_version.get().into(),
        rules,
        settings,
    )?)
    .map_err(observation_error)
}

pub(crate) fn observation_parts(
    state: GameState,
    recipient: Seat,
    world: WorldId,
    rules_version: u16,
    rules: game::RulesConfig,
    settings: crate::room::RoomSettings,
) -> Result<ObservationParts, ProjectionError> {
    let tick = game::Tick(state.physical_tick.get());
    let bombs_unavailable = hides_bombs(&state, recipient)?;
    Ok(ObservationParts {
        world,
        tick,
        recipient: domain_seat(recipient),
        rules: ObservedRules {
            version: rules_version,
            tick_rate: rules.tick_rate(),
            running_updates: state.running_updates,
            remaining_ticks: state.remaining_ticks,
            settings,
        },
        arena: arena(&state),
        actors: state.actors.into_iter().map(actor).collect(),
        bombs: if bombs_unavailable {
            BombKnowledge::Unavailable
        } else {
            BombKnowledge::Visible(state.bombs.into_iter().map(bomb).collect())
        },
        flames: state
            .flames
            .into_iter()
            .map(|flame| {
                Ok(ObservedFlame {
                    cell: cell(flame.cell),
                    remaining_ticks: remaining(flame.expires_at.get(), tick.0)
                        .ok_or(ProjectionError::InvalidFlames)?,
                })
            })
            .collect::<Result<_, _>>()?,
        oil: state
            .oil_barrels
            .into_iter()
            .map(|value| ObservedOil {
                cell: cell(value.cell),
                state: match value.state {
                    super::OilBarrelState::Unlit => game::OilBarrelState::Unlit,
                    super::OilBarrelState::Burning => game::OilBarrelState::Burning,
                },
            })
            .collect(),
        pickups: state
            .pickups
            .into_iter()
            .map(|value| ObservedPickup {
                cell: cell(value.cell),
                kind: super::cpu_observation_types::domain_pickup(value.kind),
            })
            .collect(),
    })
}

fn hides_bombs(state: &GameState, recipient: Seat) -> Result<bool, ProjectionError> {
    match state
        .actors
        .iter()
        .find(|actor| actor_seat(actor) == recipient)
    {
        Some(ActorView::Visible { effects, .. }) => Ok(effects
            .as_slice()
            .iter()
            .any(|effect| effect.kind() == super::EffectKind::InvisibleBombs)),
        _ => Err(ProjectionError::InvalidActors),
    }
}

fn actor_seat(actor: &ActorView) -> Seat {
    match actor {
        ActorView::Visible { seat, .. } | ActorView::Hidden { seat, .. } => *seat,
    }
}

fn arena(state: &GameState) -> ObservedArena {
    ObservedArena {
        width: state.arena.width.into(),
        height: state.arena.height.into(),
        terrain: state
            .arena
            .terrain
            .iter()
            .copied()
            .map(|value| match value {
                super::Terrain::Floor => game::Terrain::Floor,
                super::Terrain::Wall => game::Terrain::Wall,
            })
            .collect(),
        blocks: state.arena.blocks.iter().copied().map(cell).collect(),
    }
}

fn actor(value: ActorView) -> ObservedActor {
    match value {
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
                x: position.x().into(),
                y: position.y().into(),
            },
            heading: match heading {
                protocol::Direction::North => game::Direction::North,
                protocol::Direction::East => game::Direction::East,
                protocol::Direction::South => game::Direction::South,
                protocol::Direction::West => game::Direction::West,
            },
            speed: match speed_level {
                0 => game::SpeedLevel::Base,
                1 => game::SpeedLevel::One,
                2 => game::SpeedLevel::Two,
                3 => game::SpeedLevel::Three,
                4 => game::SpeedLevel::Four,
                _ => unreachable!(),
            },
            alive,
            bomb_capacity: bomb_capacity.into(),
            bomb_range: bomb_range.into(),
            live_bomb_count: live_bomb_count.into(),
            equipment: inventory.equipment.map(|value| match value {
                super::EquipmentId::Kick => game::Equipment::Kick,
                super::EquipmentId::Punch => game::Equipment::Punch,
                super::EquipmentId::Bracelet => game::Equipment::Bracelet,
                super::EquipmentId::WallPass => game::Equipment::WallPass,
                super::EquipmentId::Heart => game::Equipment::Heart,
            }),
            weapon: inventory.weapon.map(|value| match value {
                super::WeaponId::RemoteBomb => game::SpecialWeapon::Remote,
                super::WeaponId::PiercingBomb => game::SpecialWeapon::Piercing,
                super::WeaponId::PowerBomb => game::SpecialWeapon::Power,
                super::WeaponId::Landmine => game::SpecialWeapon::Landmine,
            }),
            effects: effects
                .as_slice()
                .iter()
                .copied()
                .map(|effect| ObservedEffect {
                    kind: super::cpu_observation_types::domain_effect(effect.kind()),
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
        placed_at: game::Tick(value.placed_at.get()),
        cell: cell(value.cell),
        owner: entity(value.owner),
        range: value.range.into(),
        trigger,
        blast: match value.blast {
            super::BlastKind::Standard => game::BlastKind::Standard,
            super::BlastKind::Piercing => game::BlastKind::Piercing,
            super::BlastKind::Power => game::BlastKind::Power,
        },
        unknown: UnknownBombFacts::ALL,
    }
}

fn remaining(deadline: u64, now: u64) -> Option<u16> {
    u16::try_from(deadline.checked_sub(now)?).ok()
}
fn cell(value: protocol::Cell) -> game::CellId {
    game::CellId::new(value.column().into(), value.row().into())
}
fn entity(value: protocol::EntityId) -> game::EntityId {
    game::EntityId(value.get())
}
const fn domain_seat(value: Seat) -> game::SeatId {
    match value {
        Seat::One => game::SeatId::One,
        Seat::Two => game::SeatId::Two,
        Seat::Three => game::SeatId::Three,
        Seat::Four => game::SeatId::Four,
    }
}
pub(crate) const fn observation_error(value: ObservationError) -> ProjectionError {
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
