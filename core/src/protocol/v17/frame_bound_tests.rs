use jsonschema::validator_for;
use serde::Serialize;
use serde_json::Value;

use super::*;
use crate::protocol::v14::{DamagePreventionSource, EventIndex, EventKind};
use crate::protocol::{
    Cell, Counter, Direction, EntityId, MAX_COUNTER, MAX_SAFE_TICK, Position, Tick,
};

// Independently summed from the recursive v17 schema, including every object key and
// punctuation byte, widest closed-union variant, maximum scalar width, and bounded collection.
const SCHEMA_RECURSIVE_SNAPSHOT_UPPER_BOUND: usize = 142_065;
const SCHEMA_RECURSIVE_ADMISSION_UPPER_BOUND: usize = 74_617;
const SCHEMA_RECURSIVE_STATE_UPPER_BOUND: usize = 71_687;
const _: () = assert!(SCHEMA_RECURSIVE_SNAPSHOT_UPPER_BOUND <= MAX_SERVER_MESSAGE_BYTES);
const _: () = assert!(SCHEMA_RECURSIVE_ADMISSION_UPPER_BOUND <= MAX_SERVER_MESSAGE_BYTES);
const _: () = assert!(SCHEMA_RECURSIVE_STATE_UPPER_BOUND <= MAX_SERVER_MESSAGE_BYTES);

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

fn largest_actor(tick: Tick) -> ActorView {
    ActorView::Visible {
        id: EntityId::new(u32::MAX),
        seat: Seat::Three,
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
        storm_slow_remaining_ticks: 180,
    }
}

fn longest_event(cell: Cell, bomb: BombView) -> EventKind {
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
            kind: PickupKind::InvisibilityCloak,
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
    let environment = Environment {
        storm_cloud: true,
        lairs: Lairs {
            enabled: true,
            spawn_seconds: 60,
            max_living_mobs: 1,
        },
    };
    room.settings = RoomSettings::new(
        GameSettings::new(4, 256, 9, 300, room.available_items).unwrap(),
        environment,
    );
    room.settings_revision = Counter::new(MAX_COUNTER).unwrap();
    room.settings_editor = Some(Seat::Three);
    room.configuration = RoomConfiguration::from_domain(crate::room::LairRoomConfiguration::new(
        crate::room::RoomConfiguration::new(
            &[
                crate::room::SeatControl::Human,
                crate::room::SeatControl::Cpu,
                crate::room::SeatControl::Cpu,
                crate::room::SeatControl::Cpu,
            ],
            Some(crate::room::CpuDifficulty::Standard),
            crate::room::BoardSize::FifteenByThirteen,
        )
        .unwrap(),
        crate::game::StormCloudConfig::Single,
        crate::room::LairTuning::new(true, crate::room::LairSettings::new(60, 1).unwrap()),
    ))
    .unwrap();
    room.revision = Counter::new(MAX_COUNTER).unwrap();
    room.round_id = Counter::new(MAX_COUNTER).unwrap();
    room.tick = tick;
    room.round_number = RoundCount::new(MAX_COUNTER).unwrap();
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
    let result = longest([
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
            result,
            rematch_at: tick,
        },
        RoomPhase::Interrupted {
            ended_at: tick,
            reason: MatchInterruptReason::ServerOverloaded,
        },
    ]);
}

fn assert_schema_valid(message: &GameServerMessage, encoded: &str) {
    let value: Value = serde_json::from_str(encoded).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v17.schema.json")).unwrap();
    let validator = validator_for(&schema).unwrap();
    let errors = validator
        .iter_errors(&value)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
    assert_eq!(encode_server_message(message).unwrap(), encoded);
}

fn assert_schema_definition_valid(definition: &str, value: Value) {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v17.schema.json")).unwrap();
    schema["$ref"] = Value::String(format!("#/$defs/{definition}"));
    assert!(
        validator_for(&schema).unwrap().is_valid(&value),
        "invalid {definition}"
    );
}

#[test]
fn typed_maximum_v17_snapshot_and_admission_fit_the_declared_cap() {
    let case = super::mine_contract_tests::dense_mine_case(false);
    let publication = bind_expanded(
        &case.room,
        case.room.round_id,
        &case.capture,
        case.generation,
    )
    .unwrap();
    let mut room = case.room;
    let mut state = project_recipient(&publication, Seat::One).unwrap().state;
    let tick = Tick::new(MAX_SAFE_TICK).unwrap();
    let cell = Cell::new(14, 12).unwrap();
    let cells = (0..13)
        .flat_map(|row| (0..15).map(move |column| Cell::new(column, row).unwrap()))
        .collect::<Vec<_>>();
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
    state.running_updates = 10_800;
    state.remaining_ticks = 10_800;
    state.physical_tick = tick;
    state.arena.terrain = vec![Terrain::Floor; 195];
    state.arena.blocks = cells.clone();
    state.actors = vec![largest_actor(tick); 4];
    state.bombs = vec![bomb; 195];
    state.flames = vec![
        FlameView {
            cell,
            expires_at: tick
        };
        195
    ];
    state.oil_barrels = cells
        .iter()
        .take(8)
        .copied()
        .map(|cell| OilBarrelView {
            cell,
            state: OilBarrelState::Burning,
        })
        .collect();
    state.pickups = cells
        .iter()
        .copied()
        .map(|cell| PickupView {
            cell,
            kind: PickupKind::InvisibilityCloak,
        })
        .collect();
    state.storm_cloud = Some(StormCloudView {
        position: Position::new(15_359, 13_311).unwrap(),
        direction: Direction::North,
        leg_remaining_ticks: 128,
    });
    state.lairs = vec![
        LairView {
            id: 1,
            cell,
            spawn_remaining_ticks: Some(3_600),
            living_mobs: 1,
        };
        2
    ];
    state.mobs = vec![
        MobView {
            id: EntityId::new(u32::MAX),
            lair_id: 1,
            position: Position::new(15_359, 13_311).unwrap(),
            heading: Direction::North,
            frozen: false,
        };
        2
    ];
    let events = vec![
        WireEvent {
            tick,
            index: EventIndex::new_for_rules(215, 9).unwrap(),
            kind: longest_event(cell, bomb),
        };
        256
    ];
    let snapshot = GameServerMessage::Snapshot {
        room: room.clone(),
        state: state.clone(),
        events: SnapshotEvents::new_for_rules(events, 9).unwrap(),
    };
    let admission = GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room,
        you_seat: Seat::Three,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_action_id: Some(Counter::new(MAX_COUNTER).unwrap()),
        state: Some(state.clone()),
    };
    let snapshot_json = serde_json::to_string(&snapshot).unwrap();
    let admission_json = serde_json::to_string(&admission).unwrap();
    assert_schema_definition_valid(
        "RoomView",
        serde_json::to_value(&snapshot).unwrap()["room"].clone(),
    );
    assert_schema_definition_valid("GameState", serde_json::to_value(&state).unwrap());
    eprintln!(
        "v17 typed schema bounds: state={} snapshot={} admission={}",
        encoded_len(&state),
        snapshot_json.len(),
        admission_json.len()
    );
    assert!(encoded_len(&state) <= SCHEMA_RECURSIVE_STATE_UPPER_BOUND);
    assert!(snapshot_json.len() <= SCHEMA_RECURSIVE_SNAPSHOT_UPPER_BOUND);
    assert!(admission_json.len() <= SCHEMA_RECURSIVE_ADMISSION_UPPER_BOUND);
    assert!(snapshot_json.len() <= MAX_SERVER_MESSAGE_BYTES);
    assert!(admission_json.len() <= MAX_SERVER_MESSAGE_BYTES);
    assert_schema_valid(&snapshot, &snapshot_json);
    assert_schema_valid(&admission, &admission_json);
}
