use serde::Serialize;

use super::*;
use crate::protocol::{
    Cell, Counter, Direction, EntityId, MAX_COUNTER, MAX_SAFE_TICK, Position, Tick,
};

// Independently summed from the v14 schema: every object key/punctuation byte, the widest
// closed-union variant, maximum scalar width, and every bounded collection cardinality. The
// typed construction below is a second check that exercises those choices through serde.
const SCHEMA_RECURSIVE_SNAPSHOT_UPPER_BOUND: usize = 93_911;
const SCHEMA_RECURSIVE_ADMISSION_UPPER_BOUND: usize = 43_359;
const _: () = assert!(SCHEMA_RECURSIVE_SNAPSHOT_UPPER_BOUND <= MAX_SERVER_MESSAGE_BYTES);
const _: () = assert!(SCHEMA_RECURSIVE_ADMISSION_UPPER_BOUND <= MAX_SERVER_MESSAGE_BYTES);

fn encoded_len(value: &impl Serialize) -> usize {
    serde_json::to_string(value).unwrap().len()
}

fn longest<T: Serialize>(values: impl IntoIterator<Item = T>) -> T {
    values
        .into_iter()
        .max_by_key(encoded_len)
        .expect("closed union has a variant")
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

fn largest_actor(seat: Seat, tick: Tick) -> ActorView {
    VisibleActorView {
        id: EntityId::new(u32::MAX),
        seat,
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
            equipment: Some(longest([
                EquipmentId::Kick,
                EquipmentId::Punch,
                EquipmentId::Bracelet,
                EquipmentId::WallPass,
                EquipmentId::Heart,
            ])),
            weapon: Some(longest([
                WeaponId::RemoteBomb,
                WeaponId::PiercingBomb,
                WeaponId::PowerBomb,
                WeaponId::Landmine,
            ])),
        },
        effects: EffectViews::new(vec![longest_effect(); 16]).unwrap(),
        damage_grace_remaining_ticks: DamageGraceTicks::new(60).unwrap(),
        voluntary_actions_frozen: false,
        last_teleport_tick: Some(tick),
    }
    .into()
}

fn longest_event_kind(cell: Cell, bomb: BombView) -> EventKind {
    let id = EntityId::new(u32::MAX);
    let result = longest([
        RoundResult::Winner { seat: Seat::Three },
        RoundResult::Draw {
            reason: DrawReason::MutualElimination,
        },
        RoundResult::Draw {
            reason: DrawReason::Timeout,
        },
    ]);
    longest([
        EventKind::BombPlaced { bomb },
        EventKind::BombDetonated { bomb },
        EventKind::ActorEliminated {
            id,
            seat: Seat::Three,
        },
        EventKind::PickupCollected {
            id,
            seat: Seat::Three,
            cell,
            kind: longest([
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
            ]),
        },
        EventKind::RoundEnded { result },
        EventKind::DamagePrevented {
            id,
            seat: Seat::Three,
            source: longest([
                DamagePreventionSource::Heart,
                DamagePreventionSource::Shield,
            ]),
        },
    ])
}

fn maximize_room(room: &mut RoomView, tick: Tick) {
    room.settings = GameSettings::new(4, 256, 9, 300, room.available_items).unwrap();
    room.settings_revision = Counter::new(MAX_COUNTER).unwrap();
    room.revision = Counter::new(MAX_COUNTER).unwrap();
    room.round_id = Counter::new(MAX_COUNTER).unwrap();
    room.tick = tick;
    room.round_number = RoundCount::new(MAX_COUNTER).unwrap();
    room.settings_editor = Some(Seat::Three);
    let mut configurations = Vec::new();
    for board in [
        crate::room::BoardSize::ElevenByNine,
        crate::room::BoardSize::ThirteenByEleven,
        crate::room::BoardSize::FifteenByThirteen,
    ] {
        configurations.push(
            RoomConfiguration::from_domain(
                crate::room::RoomConfiguration::new(
                    &[crate::room::SeatControl::Human; 4],
                    None,
                    board,
                )
                .unwrap(),
            )
            .unwrap(),
        );
        for cpu_count in 1..=3 {
            let mut controls = vec![crate::room::SeatControl::Human; 4];
            for control in controls.iter_mut().rev().take(cpu_count) {
                *control = crate::room::SeatControl::Cpu;
            }
            for difficulty in [
                crate::room::CpuDifficulty::Easy,
                crate::room::CpuDifficulty::Standard,
            ] {
                configurations.push(
                    RoomConfiguration::from_domain(
                        crate::room::RoomConfiguration::new(&controls, Some(difficulty), board)
                            .unwrap(),
                    )
                    .unwrap(),
                );
            }
        }
    }
    room.configuration = longest(configurations);
    for score in &mut room.scores {
        score.seat = Seat::Three;
        score.wins = Wins::new(9).unwrap();
    }
    room.last_round = Some(CompletedRound {
        round_id: Counter::new(MAX_COUNTER).unwrap(),
        round_number: Counter::new(MAX_COUNTER).unwrap(),
        ended_at: tick,
        result: RoundResult::Draw {
            reason: DrawReason::MutualElimination,
        },
    });
    for seat in &mut room.seats {
        seat.seat = Seat::Three;
        seat.occupied = false;
        seat.ready = false;
        seat.rematch_ready = false;
        seat.connection = longest([
            ConnectionStatus::Vacant,
            ConnectionStatus::Connected,
            ConnectionStatus::Disconnected { expires_at: tick },
            ConnectionStatus::Expired,
            ConnectionStatus::ServerControlled,
        ]);
        seat.controller = longest([
            Controller::Human,
            Controller::Cpu {
                difficulty: Difficulty::Easy,
            },
            Controller::Cpu {
                difficulty: Difficulty::Standard,
            },
        ]);
    }
    let match_result = longest([
        MatchResult::Winner {
            seat: Seat::Three,
            reason: MatchWinnerReason::RoundWins,
        },
        MatchResult::Winner {
            seat: Seat::Three,
            reason: MatchWinnerReason::Forfeit,
        },
        MatchResult::Abandoned,
        MatchResult::Interrupted {
            reason: MatchInterruptReason::ConnectionLost,
        },
        MatchResult::Interrupted {
            reason: MatchInterruptReason::ServerOverloaded,
        },
        MatchResult::Interrupted {
            reason: MatchInterruptReason::ServerStopped,
        },
        MatchResult::Interrupted {
            reason: MatchInterruptReason::InternalError,
        },
    ]);
    room.phase = longest([
        RoomPhase::Waiting,
        RoomPhase::Countdown {
            kind: CountdownKind::InterRound,
            ends_at: tick,
        },
        RoomPhase::Running,
        RoomPhase::Result { hold_until: tick },
        RoomPhase::MatchResult {
            ended_at: tick,
            result: match_result,
            rematch_at: tick,
        },
        RoomPhase::Interrupted {
            ended_at: tick,
            reason: longest([
                MatchInterruptReason::ConnectionLost,
                MatchInterruptReason::ServerOverloaded,
                MatchInterruptReason::ServerStopped,
                MatchInterruptReason::InternalError,
            ]),
        },
    ]);
}

#[test]
fn typed_maximum_v14_snapshot_and_admission_fit_the_declared_cap() {
    let (mut room, mut state, _) = super::max_frame_tests::maximal_state();
    let tick = Tick::new(MAX_SAFE_TICK).unwrap();
    let cell = Cell::new(14, 12).unwrap();
    let bomb = BombView {
        id: EntityId::new(u32::MAX),
        cell,
        owner: EntityId::new(u32::MAX),
        range: 8,
        placed_at: tick,
        trigger: longest([
            BombTrigger::timed(600, true).unwrap(),
            BombTrigger::timed(600, false).unwrap(),
            BombTrigger::remote(),
            BombTrigger::contact(),
        ]),
        blast: longest([BlastKind::Standard, BlastKind::Piercing, BlastKind::Power]),
    };
    maximize_room(&mut room, tick);
    state.actors = vec![largest_actor(Seat::Three, tick); 4];
    state.bombs = vec![bomb; MAX_BOMBS];
    state.arena.blocks = vec![cell; crate::protocol::MAX_BLOCKS];
    state.flames = vec![
        FlameView {
            cell,
            expires_at: tick,
        };
        crate::protocol::MAX_FLAMES
    ];
    state.oil_barrels = vec![
        OilBarrelView {
            cell,
            state: OilBarrelState::Burning,
        };
        crate::game::MAX_OIL_BARRELS
    ];
    state.pickups = vec![
        PickupView {
            cell,
            kind: PickupKind::InvisibilityCloak,
        };
        crate::protocol::MAX_PICKUPS
    ];
    let kind = longest_event_kind(cell, bomb);
    let events = (0..MAX_SNAPSHOT_EVENTS)
        .map(|_| WireEvent {
            tick,
            index: EventIndex::new(MAX_EVENT_INDEX).unwrap(),
            kind: kind.clone(),
        })
        .collect();
    let snapshot = GameServerMessage::Snapshot {
        room: room.clone(),
        state: state.clone(),
        events: SnapshotEvents::new(events).unwrap(),
    };
    let admitted = GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room,
        you_seat: Seat::Three,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_action_id: Some(Counter::new(MAX_COUNTER).unwrap()),
        state: Some(state),
    };
    let snapshot_json = serde_json::to_string(&snapshot).unwrap();
    let admitted_json = serde_json::to_string(&admitted).unwrap();
    eprintln!(
        "v14 typed upper bounds: snapshot={} admitted={}",
        snapshot_json.len(),
        admitted_json.len()
    );
    assert!(snapshot_json.len() <= MAX_SERVER_MESSAGE_BYTES);
    assert!(admitted_json.len() <= MAX_SERVER_MESSAGE_BYTES);
    assert!(snapshot_json.len() <= SCHEMA_RECURSIVE_SNAPSHOT_UPPER_BOUND);
    assert!(admitted_json.len() <= SCHEMA_RECURSIVE_ADMISSION_UPPER_BOUND);
    assert_eq!(encode_server_message(&snapshot).unwrap(), snapshot_json);
    assert_eq!(encode_server_message(&admitted).unwrap(), admitted_json);
}
