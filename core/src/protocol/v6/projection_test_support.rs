use crate::game::{
    ActorInit, Direction, EntityId, Position, RoundResult, RulesConfig, SeatId, SpeedLevel, State,
    Tick as DomainTick, World,
};
use crate::protocol::{Counter, MapId, RoomCode, RoomId, RulesVersion};
use crate::room::{COUNTDOWN_TICKS, MatchLifecycle};

use super::{RoomMetadata, RoomView, project_room_view};

pub(super) fn metadata() -> RoomMetadata {
    RoomMetadata {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        revision: Counter::new(1).unwrap(),
        rules_version: RulesVersion::ClassicV2,
        map_id: MapId::FixedV1,
        settings: super::GameSettings::for_rules(RulesVersion::ClassicV2),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
    }
}

pub(super) fn occupied() -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::new();
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle
}

pub(super) fn project(lifecycle: &MatchLifecycle) -> RoomView {
    project_room_view(metadata(), lifecycle).unwrap()
}

pub(super) fn advance_idle(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

pub(super) fn advance_running(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.complete_running_tick(None).unwrap();
    }
}

pub(super) fn start(lifecycle: &mut MatchLifecycle) {
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    advance_idle(lifecycle, COUNTDOWN_TICKS);
}

pub(super) fn result_after_wins(wins: u8) -> MatchLifecycle {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    for completed in 0..wins {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        if completed + 1 < wins {
            advance_idle(&mut lifecycle, COUNTDOWN_TICKS * 2);
        }
    }
    lifecycle
}

pub(super) fn running_with_wins(wins: u8) -> MatchLifecycle {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    for _ in 0..wins {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        advance_idle(&mut lifecycle, COUNTDOWN_TICKS * 2);
    }
    lifecycle
}

pub(super) fn state_at(tick: u64) -> State {
    let arena = crate::game::fixtures::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    State::new_at_tick(
        RulesConfig::classic_v2(),
        World::new(arena),
        actors,
        DomainTick(tick),
    )
    .unwrap()
}
