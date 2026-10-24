use serde::Serialize;

use super::super::{
    Cell, Counter, Direction, DrawReason, EntityId, MAX_COUNTER, Position, RoomCode, RoomId, Seat,
    Tick, WireRoundResult,
};
use super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BlastKind, BombTrigger, BombView, CompletedRound,
    ConnectionStatus, DamageGraceTicks, EffectKind, EffectView, EffectViews, EventIndex, EventKind,
    GameServerMessage, GameSettings, GameState, InventoryView, ItemMask, ItemSet, MapId,
    MatchInterruptReason, MatchResult, MatchWinnerReason, OilBarrelState, OilBarrelView,
    PickupKind, PickupView, ResumeToken, RoomPhase, RoomView, RoundCount, RulesVersion, Scores,
    SeatStatus, SnapshotEvents, Terrain, VisibleActorView, Wins, WireEvent, encode_server_message,
};

const FRAME_BYTES: usize = 65_536;

fn encoded_len(value: &impl Serialize) -> usize {
    serde_json::to_string(value).unwrap().len()
}

fn longest<T: Serialize>(values: impl IntoIterator<Item = T>) -> T {
    values
        .into_iter()
        .max_by_key(encoded_len)
        .expect("closed union has a variant")
}

fn longest_round_result() -> WireRoundResult {
    longest([
        WireRoundResult::Winner { seat: Seat::One },
        WireRoundResult::Winner { seat: Seat::Two },
        WireRoundResult::Draw {
            reason: DrawReason::MutualElimination,
        },
        WireRoundResult::Draw {
            reason: DrawReason::Timeout,
        },
    ])
}

fn longest_match_result() -> MatchResult {
    let mut values = vec![MatchResult::Abandoned];
    for seat in [Seat::One, Seat::Two] {
        for reason in [MatchWinnerReason::RoundWins, MatchWinnerReason::Forfeit] {
            values.push(MatchResult::Winner { seat, reason });
        }
    }
    for reason in [
        MatchInterruptReason::ConnectionLost,
        MatchInterruptReason::ServerOverloaded,
        MatchInterruptReason::ServerStopped,
        MatchInterruptReason::InternalError,
    ] {
        values.push(MatchResult::Interrupted { reason });
    }
    longest(values)
}

fn longest_phase(tick: Tick) -> RoomPhase {
    let result = longest_match_result();
    longest([
        RoomPhase::Waiting,
        RoomPhase::Countdown {
            kind: super::CountdownKind::Initial,
            ends_at: tick,
        },
        RoomPhase::Countdown {
            kind: super::CountdownKind::InterRound,
            ends_at: tick,
        },
        RoomPhase::Running,
        RoomPhase::Result { hold_until: tick },
        RoomPhase::MatchResult {
            ended_at: tick,
            result,
            rematch_at: tick,
        },
        RoomPhase::Interrupted {
            ended_at: tick,
            reason: MatchInterruptReason::ConnectionLost,
        },
        RoomPhase::Interrupted {
            ended_at: tick,
            reason: MatchInterruptReason::ServerOverloaded,
        },
        RoomPhase::Interrupted {
            ended_at: tick,
            reason: MatchInterruptReason::ServerStopped,
        },
        RoomPhase::Interrupted {
            ended_at: tick,
            reason: MatchInterruptReason::InternalError,
        },
    ])
}

fn longest_connection(tick: Tick) -> ConnectionStatus {
    longest([
        ConnectionStatus::Vacant,
        ConnectionStatus::Connected,
        ConnectionStatus::Disconnected { expires_at: tick },
        ConnectionStatus::Expired,
    ])
}

fn longest_equipment() -> super::EquipmentId {
    longest([
        super::EquipmentId::Kick,
        super::EquipmentId::Punch,
        super::EquipmentId::Bracelet,
        super::EquipmentId::WallPass,
        super::EquipmentId::Heart,
    ])
}

fn longest_weapon() -> super::WeaponId {
    longest([
        super::WeaponId::RemoteBomb,
        super::WeaponId::PiercingBomb,
        super::WeaponId::PowerBomb,
        super::WeaponId::Landmine,
    ])
}

fn longest_pickup() -> PickupKind {
    longest([
        PickupKind::BombCapacity,
        PickupKind::BlastRange,
        PickupKind::MovementSpeed,
        PickupKind::Kick,
        PickupKind::Punch,
        PickupKind::Bracelet,
        PickupKind::WallPass,
        PickupKind::Heart,
        PickupKind::RemoteBomb,
        PickupKind::PiercingBomb,
        PickupKind::PowerBomb,
        PickupKind::Landmine,
        PickupKind::Shield,
        PickupKind::Stopwatch,
        PickupKind::InvisibilityCloak,
        PickupKind::GoldenFlame,
        PickupKind::ShortFuse,
        PickupKind::Diarrhea,
        PickupKind::SlowFoot,
        PickupKind::ShortFlame,
        PickupKind::LongFuse,
        PickupKind::ReverseControls,
        PickupKind::InvisibleBombs,
        PickupKind::OilBarrel,
        PickupKind::Teleporter,
    ])
}

fn longest_effect() -> EffectView {
    longest(
        [
            EffectKind::Shield,
            EffectKind::Stopwatch,
            EffectKind::InvisibilityCloak,
            EffectKind::GoldenFlame,
            EffectKind::ShortFuse,
            EffectKind::Diarrhea,
            EffectKind::SlowFoot,
            EffectKind::ShortFlame,
            EffectKind::LongFuse,
            EffectKind::ReverseControls,
            EffectKind::InvisibleBombs,
        ]
        .map(|kind| {
            EffectView::new(
                kind,
                match kind {
                    EffectKind::Shield => 300,
                    EffectKind::Stopwatch => 180,
                    _ => 600,
                },
            )
            .unwrap()
        }),
    )
}

fn largest_bomb(tick: Tick, cell: Cell) -> BombView {
    let trigger = longest([
        BombTrigger::timed(600, true).unwrap(),
        BombTrigger::timed(600, false).unwrap(),
        BombTrigger::remote(),
        BombTrigger::contact(),
    ]);
    let blast = longest([BlastKind::Standard, BlastKind::Piercing, BlastKind::Power]);
    BombView {
        id: EntityId::new(u32::MAX),
        cell,
        owner: EntityId::new(u32::MAX),
        range: 8,
        placed_at: tick,
        trigger,
        blast,
    }
}

fn largest_state(tick: Tick, cell: Cell, bomb: BombView) -> GameState {
    let effect = longest_effect();
    let visible: ActorView = VisibleActorView {
        id: EntityId::new(u32::MAX),
        seat: Seat::One,
        position: Position::new(15_359, 13_311).unwrap(),
        heading: longest([
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]),
        speed_level: 4,
        alive: false,
        bomb_capacity: 8,
        bomb_range: 8,
        live_bomb_count: 8,
        inventory: InventoryView {
            equipment: longest([None, Some(longest_equipment())]),
            weapon: longest([None, Some(longest_weapon())]),
        },
        effects: EffectViews::new(vec![effect; 16]).unwrap(),
        damage_grace_remaining_ticks: DamageGraceTicks::new(60).unwrap(),
        voluntary_actions_frozen: false,
        last_teleport_tick: Some(tick),
    }
    .into();
    let hidden: ActorView = super::HiddenActorView {
        id: EntityId::new(u32::MAX),
        seat: Seat::One,
        alive: false,
    }
    .into();
    let actor = longest([visible, hidden]);
    let terrain = longest([Terrain::Floor, Terrain::Wall]);
    let pickup = PickupView {
        cell,
        kind: longest_pickup(),
    };
    GameState {
        running_updates: 10_800,
        remaining_ticks: 10_800,
        physical_tick: tick,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![terrain; ARENA_TERRAIN_CELLS],
            blocks: vec![cell; 195],
        },
        actors: [actor.clone(), actor],
        bombs: vec![bomb; 16],
        flames: vec![
            super::FlameView {
                cell,
                expires_at: tick
            };
            195
        ],
        oil_barrels: vec![
            OilBarrelView {
                cell,
                state: OilBarrelState::Burning
            };
            8
        ],
        pickups: vec![pickup; 195],
    }
}

fn largest_room(tick: Tick) -> RoomView {
    let items = ItemSet::from_mask(ItemMask::new((1 << 22) - 1).unwrap());
    let connection = longest_connection(tick);
    let completed = CompletedRound {
        round_id: Counter::new(MAX_COUNTER).unwrap(),
        round_number: Counter::new(MAX_COUNTER).unwrap(),
        ended_at: tick,
        result: longest_round_result(),
    };
    RoomView {
        settings: GameSettings::new(4, 256, 9, 300, items).unwrap(),
        settings_revision: Counter::new(MAX_COUNTER).unwrap(),
        settings_editor: longest([None, Some(Seat::One), Some(Seat::Two)]),
        available_items: items,
        room_id: RoomId::new("ffffffffffffffffffffffffffffffff").unwrap(),
        room_code: RoomCode::new("ZZZZZZZZ").unwrap(),
        round_id: Counter::new(MAX_COUNTER).unwrap(),
        revision: Counter::new(MAX_COUNTER).unwrap(),
        tick,
        rules_version: RulesVersion::ExpandedV5,
        map_id: longest([MapId::FixedV1, MapId::GeneratedV1, MapId::GeneratedV2]),
        round_number: RoundCount::new(MAX_COUNTER).unwrap(),
        scores: Scores {
            one: Wins::new(9).unwrap(),
            two: Wins::new(9).unwrap(),
        },
        last_round: longest([None, Some(completed)]),
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: false,
            ready: false,
            rematch_ready: false,
            connection,
        }),
        phase: longest_phase(tick),
    }
}

fn longest_event(tick: Tick, cell: Cell, bomb: BombView) -> WireEvent {
    let result = longest_round_result();
    let mut kinds = vec![
        EventKind::BombPlaced { bomb },
        EventKind::BombDetonated { bomb },
        EventKind::ActorEliminated {
            id: EntityId::new(u32::MAX),
            seat: Seat::One,
        },
        EventKind::PickupCollected {
            id: EntityId::new(u32::MAX),
            seat: Seat::One,
            cell,
            kind: longest_pickup(),
        },
        EventKind::RoundEnded { result },
    ];
    for source in [
        super::DamagePreventionSource::Heart,
        super::DamagePreventionSource::Shield,
    ] {
        kinds.push(EventKind::DamagePrevented {
            id: EntityId::new(u32::MAX),
            seat: Seat::One,
            source,
        });
    }
    WireEvent {
        tick,
        index: EventIndex::new(95).unwrap(),
        kind: longest(kinds),
    }
}

#[test]
fn synthetic_typed_overapproximation_proves_v11_frames_fit() {
    let tick = Tick::new(9_007_199_254_740_991).unwrap();
    let cell = Cell::new(14, 12).unwrap();
    let bomb = largest_bomb(tick, cell);
    let state = largest_state(tick, cell, bomb);
    let room = largest_room(tick);
    let events = SnapshotEvents::new(vec![longest_event(tick, cell, bomb); 96]).unwrap();
    let snapshot = GameServerMessage::Snapshot {
        room: room.clone(),
        state: state.clone(),
        events,
    };
    let admitted = GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room,
        you_seat: Seat::One,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_action_id: longest([None, Some(Counter::new(MAX_COUNTER).unwrap())]),
        state: longest([None, Some(state)]),
    };

    let snapshot_json = serde_json::to_string(&snapshot).unwrap();
    let admitted_json = serde_json::to_string(&admitted).unwrap();
    let GameServerMessage::Snapshot {
        room,
        state,
        events,
    } = &snapshot
    else {
        unreachable!()
    };
    eprintln!(
        "v11 synthetic upper bounds: snapshot={} admitted={} room={} state={} events={}",
        snapshot_json.len(),
        admitted_json.len(),
        encoded_len(room),
        encoded_len(state),
        encoded_len(events)
    );
    assert!(
        snapshot_json.len() < FRAME_BYTES,
        "snapshot bound is {}",
        snapshot_json.len()
    );
    assert!(
        admitted_json.len() < FRAME_BYTES,
        "admitted bound is {}",
        admitted_json.len()
    );
    assert_eq!(encode_server_message(&snapshot).unwrap(), snapshot_json);
    assert_eq!(encode_server_message(&admitted).unwrap(), admitted_json);
}
