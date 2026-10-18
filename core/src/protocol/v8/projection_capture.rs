use crate::game;
use crate::protocol::{Cell, Direction, EntityId, Position, Seat, Tick, WireRoundResult};

use super::projection_facts::{
    EventFacts, ExpandedCapture, ProjectionError, ReportFacts, WorldFacts,
};
use super::projection_validation::{validate_reports, validate_world};
use super::{
    ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks, EffectKind, EffectView,
    EffectViews, EquipmentId, EventKind, FlameView, InventoryView, ItemSet, PickupKind, PickupView,
    RulesVersion, Terrain, VisibleActorView, WeaponId,
};

pub fn capture_expanded(
    state: &game::State,
    reports: &[game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    let public = state.public_view();
    let physical_tick = wire_tick(public.tick)?;
    let rules_version =
        RulesVersion::try_from(public.rules_version).map_err(|_| ProjectionError::InvalidRules)?;
    let actors = [
        capture_actor(state, game::SeatId::One, physical_tick)?,
        capture_actor(state, game::SeatId::Two, physical_tick)?,
    ];
    let world = validate_world(WorldFacts {
        rules_version,
        physical_tick,
        running_updates: u16::try_from(public.running_updates)
            .map_err(|_| ProjectionError::InvalidClock)?,
        remaining_ticks: u16::try_from(public.remaining_ticks)
            .map_err(|_| ProjectionError::InvalidClock)?,
        result: wire_result(public.result),
        arena: capture_arena(&public.arena)?,
        actors,
        bombs: public
            .bombs
            .iter()
            .map(|bomb| capture_bomb(*bomb, public.tick))
            .collect::<Result<_, _>>()?,
        flames: public
            .flames
            .iter()
            .map(|flame| {
                Ok(FlameView {
                    cell: wire_cell(flame.cell)?,
                    expires_at: wire_tick(flame.expires_at)?,
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
        pickups: public
            .pickups
            .iter()
            .map(|pickup| {
                Ok(PickupView {
                    cell: wire_cell(pickup.cell)?,
                    kind: wire_pickup(pickup.kind),
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
        available_items: if rules_version == RulesVersion::ExpandedV5 {
            super::projection_binding::expanded_items()
        } else {
            ItemSet::empty()
        },
        enabled_items: if rules_version == RulesVersion::ExpandedV5 {
            item_set(state.rules().enabled_items())?
        } else {
            ItemSet::empty()
        },
    })?;
    let facts = reports
        .iter()
        .map(capture_report)
        .collect::<Result<Vec<_>, _>>()?;
    let validated_reports = validate_reports(&world, facts)?;
    Ok(ExpandedCapture {
        rules: *state.rules(),
        world,
        reports: validated_reports,
    })
}

fn capture_actor(
    state: &game::State,
    seat: game::SeatId,
    physical_tick: Tick,
) -> Result<VisibleActorView, ProjectionError> {
    let actor = &state.actors()[seat.index()];
    let effects = actor
        .effect_observations()
        .map(|(kind, expires_at)| {
            let remaining = expires_at
                .0
                .checked_sub(physical_tick.get())
                .ok_or(ProjectionError::InvalidEffects)?;
            EffectView::new(
                wire_effect(kind),
                u16::try_from(remaining).map_err(|_| ProjectionError::InvalidEffects)?,
            )
            .map_err(|_| ProjectionError::InvalidEffects)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let grace = match actor.damage_grace_observation() {
        Some(grace) => {
            physical_tick
                .get()
                .checked_sub(grace.received_at().0)
                .ok_or(ProjectionError::InvalidEffects)?;
            u8::try_from(grace.remaining_ticks(game::Tick(physical_tick.get())))
                .map_err(|_| ProjectionError::InvalidEffects)?
        }
        None => 0,
    };
    let inventory = *actor.inventory();
    Ok(VisibleActorView {
        id: EntityId::new(actor.id().0),
        seat: wire_seat(actor.seat()),
        position: wire_position(actor.position())?,
        heading: wire_direction(actor.heading()),
        speed_level: wire_speed(actor.speed()),
        alive: actor.alive(),
        bomb_capacity: u8::try_from(actor.bomb_capacity())
            .map_err(|_| ProjectionError::InvalidActors)?,
        bomb_range: u8::try_from(actor.bomb_range()).map_err(|_| ProjectionError::InvalidActors)?,
        live_bomb_count: u8::try_from(state.live_bomb_count(seat))
            .map_err(|_| ProjectionError::InvalidActors)?,
        inventory: InventoryView {
            equipment: inventory.equipment().map(wire_equipment),
            weapon: inventory.weapon().map(wire_weapon),
        },
        effects: EffectViews::new(effects).map_err(|_| ProjectionError::InvalidEffects)?,
        damage_grace_remaining_ticks: DamageGraceTicks::new(grace)
            .map_err(|_| ProjectionError::InvalidEffects)?,
    })
}

fn capture_arena(arena: &game::ArenaView) -> Result<ArenaView, ProjectionError> {
    Ok(ArenaView {
        width: u8::try_from(arena.width).map_err(|_| ProjectionError::InvalidArena)?,
        height: u8::try_from(arena.height).map_err(|_| ProjectionError::InvalidArena)?,
        terrain: arena
            .terrain
            .iter()
            .map(|terrain| match terrain {
                game::Terrain::Floor => Terrain::Floor,
                game::Terrain::Wall => Terrain::Wall,
            })
            .collect(),
        blocks: arena
            .blocks
            .iter()
            .copied()
            .map(wire_cell)
            .collect::<Result<_, _>>()?,
    })
}

fn capture_bomb(bomb: game::BombView, at: game::Tick) -> Result<BombView, ProjectionError> {
    let trigger = match bomb.trigger {
        game::BombTrigger::Timed { detonates_at } => {
            // Preserve the absolute deadline check before a zero observation can alias
            // another allowed captured duration in a frozen result.
            let remaining = detonates_at
                .0
                .checked_sub(at.0)
                .ok_or(ProjectionError::InvalidBombs)?;
            BombTrigger::timed(
                u16::try_from(remaining).map_err(|_| ProjectionError::InvalidBombs)?,
                false,
            )
            .map_err(|_| ProjectionError::InvalidBombs)?
        }
        game::BombTrigger::Remote => BombTrigger::remote(),
        game::BombTrigger::Contact => BombTrigger::contact(),
    };
    Ok(BombView {
        id: EntityId::new(bomb.id.0),
        cell: wire_cell(bomb.cell)?,
        owner: EntityId::new(bomb.owner.0),
        range: u8::try_from(bomb.range).map_err(|_| ProjectionError::InvalidBombs)?,
        placed_at: wire_tick(bomb.placed_at)?,
        trigger,
        blast: match bomb.blast {
            game::BlastKind::Standard => BlastKind::Standard,
            game::BlastKind::Piercing => BlastKind::Piercing,
            game::BlastKind::Power => BlastKind::Power,
        },
    })
}

fn capture_event_bomb(
    bomb: game::BombView,
    event_tick: game::Tick,
) -> Result<BombView, ProjectionError> {
    if matches!(
        bomb.trigger,
        game::BombTrigger::Timed { detonates_at } if detonates_at.0 < event_tick.0
    ) {
        return Err(ProjectionError::InvalidEvent);
    }
    capture_bomb(bomb, event_tick).map_err(|_| ProjectionError::InvalidEvent)
}

fn capture_report(report: &game::StepReport) -> Result<ReportFacts, ProjectionError> {
    let tick = wire_tick(report.tick)?;
    Ok(ReportFacts {
        tick,
        events: report
            .events
            .iter()
            .map(|event| {
                let kind = match event.kind {
                    game::StepEventKind::BombPlaced(bomb) => EventKind::BombPlaced {
                        bomb: capture_event_bomb(bomb, event.tick)?,
                    },
                    game::StepEventKind::BombDetonated(bomb) => EventKind::BombDetonated {
                        bomb: capture_event_bomb(bomb, event.tick)?,
                    },
                    game::StepEventKind::ActorEliminated { id, seat } => {
                        EventKind::ActorEliminated {
                            id: EntityId::new(id.0),
                            seat: wire_seat(seat),
                        }
                    }
                    game::StepEventKind::PickupCollected {
                        id,
                        seat,
                        cell,
                        kind,
                    } => EventKind::PickupCollected {
                        id: EntityId::new(id.0),
                        seat: wire_seat(seat),
                        cell: wire_cell(cell)?,
                        kind: wire_pickup(kind),
                    },
                    game::StepEventKind::RoundEnded(result) => EventKind::RoundEnded {
                        result: wire_result(Some(result)).ok_or(ProjectionError::InvalidEvent)?,
                    },
                    game::StepEventKind::DamagePrevented { id, seat, source } => {
                        EventKind::DamagePrevented {
                            id: EntityId::new(id.0),
                            seat: wire_seat(seat),
                            source: match source {
                                game::DamagePreventionSource::Heart => {
                                    super::DamagePreventionSource::Heart
                                }
                                game::DamagePreventionSource::Shield => {
                                    super::DamagePreventionSource::Shield
                                }
                            },
                        }
                    }
                };
                Ok(EventFacts {
                    tick: wire_tick(event.tick)?,
                    audience: event.audience,
                    kind,
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
    })
}

fn wire_tick(value: game::Tick) -> Result<Tick, ProjectionError> {
    Tick::new(value.0).map_err(|_| ProjectionError::InvalidClock)
}
fn wire_cell(value: game::CellId) -> Result<Cell, ProjectionError> {
    Cell::new(
        u8::try_from(value.column).map_err(|_| ProjectionError::InvalidArena)?,
        u8::try_from(value.row).map_err(|_| ProjectionError::InvalidArena)?,
    )
    .map_err(|_| ProjectionError::InvalidArena)
}
fn wire_position(value: game::Position) -> Result<Position, ProjectionError> {
    Position::new(
        u16::try_from(value.x).map_err(|_| ProjectionError::InvalidActors)?,
        u16::try_from(value.y).map_err(|_| ProjectionError::InvalidActors)?,
    )
    .map_err(|_| ProjectionError::InvalidActors)
}
fn wire_seat(value: game::SeatId) -> Seat {
    match value {
        game::SeatId::One => Seat::One,
        game::SeatId::Two => Seat::Two,
    }
}
fn wire_direction(value: game::Direction) -> Direction {
    match value {
        game::Direction::North => Direction::North,
        game::Direction::East => Direction::East,
        game::Direction::South => Direction::South,
        game::Direction::West => Direction::West,
    }
}
fn wire_speed(value: game::SpeedLevel) -> u8 {
    match value {
        game::SpeedLevel::Base => 0,
        game::SpeedLevel::One => 1,
        game::SpeedLevel::Two => 2,
        game::SpeedLevel::Three => 3,
        game::SpeedLevel::Four => 4,
    }
}
fn wire_result(value: Option<game::RoundResult>) -> Option<WireRoundResult> {
    value.map(|result| match result {
        game::RoundResult::Winner(seat) => WireRoundResult::Winner {
            seat: wire_seat(seat),
        },
        game::RoundResult::Draw(game::DrawReason::MutualElimination) => WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::MutualElimination,
        },
        game::RoundResult::Draw(game::DrawReason::Timeout) => WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::Timeout,
        },
    })
}
fn wire_pickup(value: game::PickupKind) -> PickupKind {
    match value {
        game::PickupKind::BombCapacity => PickupKind::BombCapacity,
        game::PickupKind::BlastRange => PickupKind::BlastRange,
        game::PickupKind::MovementSpeed => PickupKind::MovementSpeed,
        game::PickupKind::Heart => PickupKind::Heart,
        game::PickupKind::PiercingBomb => PickupKind::PiercingBomb,
        game::PickupKind::PowerBomb => PickupKind::PowerBomb,
        game::PickupKind::Shield => PickupKind::Shield,
        game::PickupKind::GoldenFlame => PickupKind::GoldenFlame,
        game::PickupKind::ShortFuse => PickupKind::ShortFuse,
        game::PickupKind::Diarrhea => PickupKind::Diarrhea,
        game::PickupKind::SlowFoot => PickupKind::SlowFoot,
        game::PickupKind::ShortFlame => PickupKind::ShortFlame,
        game::PickupKind::LongFuse => PickupKind::LongFuse,
        game::PickupKind::ReverseControls => PickupKind::ReverseControls,
    }
}

fn item_set(enabled: game::EnabledItems) -> Result<ItemSet, ProjectionError> {
    let mask = super::ItemMask::new(enabled.bits()).map_err(|_| ProjectionError::InvalidItems)?;
    Ok(ItemSet::from_mask(mask))
}
fn wire_equipment(value: game::Equipment) -> EquipmentId {
    match value {
        game::Equipment::Kick => EquipmentId::Kick,
        game::Equipment::Punch => EquipmentId::Punch,
        game::Equipment::Bracelet => EquipmentId::Bracelet,
        game::Equipment::WallPass => EquipmentId::WallPass,
        game::Equipment::Heart => EquipmentId::Heart,
    }
}
fn wire_weapon(value: game::SpecialWeapon) -> WeaponId {
    match value {
        game::SpecialWeapon::Remote => WeaponId::RemoteBomb,
        game::SpecialWeapon::Piercing => WeaponId::PiercingBomb,
        game::SpecialWeapon::Power => WeaponId::PowerBomb,
        game::SpecialWeapon::Landmine => WeaponId::Landmine,
    }
}
fn wire_effect(value: game::EffectKind) -> EffectKind {
    match value {
        game::EffectKind::Shield => EffectKind::Shield,
        game::EffectKind::Stopwatch => EffectKind::Stopwatch,
        game::EffectKind::InvisibilityCloak => EffectKind::InvisibilityCloak,
        game::EffectKind::GoldenFlame => EffectKind::GoldenFlame,
        game::EffectKind::ShortFuse => EffectKind::ShortFuse,
        game::EffectKind::Diarrhea => EffectKind::Diarrhea,
        game::EffectKind::SlowFoot => EffectKind::SlowFoot,
        game::EffectKind::ShortFlame => EffectKind::ShortFlame,
        game::EffectKind::LongFuse => EffectKind::LongFuse,
        game::EffectKind::ReverseControls => EffectKind::ReverseControls,
        game::EffectKind::InvisibleBombs => EffectKind::InvisibleBombs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_blast_is_captured_without_downgrading() {
        let bomb = game::BombView {
            id: game::EntityId(3),
            cell: game::CellId::new(1, 1),
            owner: game::EntityId(1),
            range: 2,
            placed_at: game::Tick(10),
            trigger: game::BombTrigger::Timed {
                detonates_at: game::Tick(190),
            },
            blast: game::BlastKind::Power,
        };
        assert_eq!(
            capture_bomb(bomb, game::Tick(10)).unwrap().blast,
            BlastKind::Power
        );
    }
}
