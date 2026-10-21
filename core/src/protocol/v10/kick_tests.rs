use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    self, ActorInit, Arena, BombTrigger as GameBombTrigger, CellId, Direction as GameDirection,
    EnabledItems, EntityId as GameEntityId, InputAction, PickupKind as GamePickupKind,
    Position as GamePosition, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    Terrain as GameTerrain, Tick as GameTick, World,
};
use crate::protocol::{Cell, Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::*;

#[path = "kick_tests/active_event_tests.rs"]
mod active_event_tests;
#[path = "kick_tests/archive_tests.rs"]
mod archive_tests;
#[path = "kick_tests/live_tests.rs"]
mod live_tests;

fn enabled(remote: bool) -> EnabledItems {
    let bits = EnabledItems::KICK.bits()
        | if remote {
            EnabledItems::REMOTE_BOMB.bits()
        } else {
            0
        };
    EnabledItems::from_bits(bits).unwrap()
}

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [GameTerrain::Floor; game::arena::ARENA_CELL_COUNT];
    for row in 0..game::arena::ARENA_HEIGHT {
        for column in 0..game::arena::ARENA_WIDTH {
            if row == 0
                || column == 0
                || row == game::arena::ARENA_HEIGHT - 1
                || column == game::arena::ARENA_WIDTH - 1
            {
                terrain[usize::from(row * game::arena::ARENA_WIDTH + column)] = GameTerrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn kick_state(trigger: GameBombTrigger, blocks: BTreeSet<CellId>) -> State {
    let bomb_cell = CellId::new(4, 5);
    let pickup_cell = CellId::new(3, 5);
    let center = GamePosition::center(bomb_cell);
    let mut state = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT
                .with_enabled_items(enabled(matches!(trigger, GameBombTrigger::Remote))),
        ),
        World::with_pickups(
            arena(blocks),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(pickup_cell, GamePickupKind::Kick)]),
        )
        .unwrap(),
        [
            ActorInit {
                id: GameEntityId(1),
                seat: SeatId::One,
                position: GamePosition {
                    x: i32::from(bomb_cell.column) * 1_024 - 320,
                    y: center.y,
                },
                heading: GameDirection::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: GameEntityId(2),
                seat: SeatId::Two,
                position: GamePosition::center(CellId::new(13, 11)),
                heading: GameDirection::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    state.step(&[]).unwrap();
    game::test_support::add_test_bomb_with_trigger(
        &mut state,
        GameEntityId(9),
        bomb_cell,
        GameEntityId(1),
        2,
        GameTick(0),
        trigger,
    )
    .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(GameDirection::East),
        }])
        .unwrap();
    state
}

fn timed_state(blocked: bool) -> State {
    let blocks = if blocked {
        BTreeSet::from([CellId::new(5, 5)])
    } else {
        BTreeSet::new()
    };
    let mut state = kick_state(
        GameBombTrigger::Timed {
            detonates_at: GameTick(180),
        },
        blocks,
    );
    if blocked {
        state.step(&[]).unwrap();
    }
    state
}

pub(super) fn room(enabled: ItemSet, tick: crate::protocol::Tick) -> RoomView {
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, enabled).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: super::expanded_items(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick,
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: true,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Connected,
        }),
        phase: RoomPhase::Running,
    }
}
