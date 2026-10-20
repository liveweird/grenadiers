use jsonschema::validator_for;
use serde_json::Value;

use super::super::{
    Cell, Counter, Direction, DrawReason, EncodeServerMessageError, EntityId, MAX_COUNTER,
    Position, RoomCode, RoomId, Seat, Tick, WireRoundResult,
};
use super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BlastKind, BombTrigger, BombView, CompletedRound,
    ConnectionStatus, DamageGraceTicks, EffectKind, EffectView, EffectViews, EventIndex, EventKind,
    GameServerMessage, GameSettings, GameState, InventoryView, ItemMask, ItemSet, MapId,
    MatchResult, MatchWinnerReason, PickupKind, PickupView, ResumeToken, RoomPhase, RoomView,
    RoundCount, RulesVersion, Scores, SeatStatus, SnapshotEvents, Terrain, VisibleActorView, Wins,
    WireEvent, encode_server_message,
};

fn schema() -> jsonschema::Validator {
    let value: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v10.schema.json")).unwrap();
    validator_for(&value).unwrap()
}

fn cells() -> Vec<Cell> {
    (0..13)
        .flat_map(|row| (0..15).map(move |column| Cell::new(column, row).unwrap()))
        .collect()
}

fn representative_body() -> (RoomView, GameState, BombView) {
    let tick = Tick::new(9_007_199_254_740_991).unwrap();
    let bomb = BombView {
        id: EntityId::new(u32::MAX),
        cell: Cell::new(14, 12).unwrap(),
        owner: EntityId::new(u32::MAX),
        range: 8,
        placed_at: tick,
        trigger: BombTrigger::timed(600, true).unwrap(),
        blast: BlastKind::Piercing,
    };
    let effects = EffectViews::new(
        [
            EffectKind::Shield,
            EffectKind::Stopwatch,
            EffectKind::InvisibilityCloak,
            EffectKind::GoldenFlame,
            EffectKind::ShortFuse,
            EffectKind::Diarrhea,
            EffectKind::ShortFlame,
            EffectKind::LongFuse,
            EffectKind::ReverseControls,
            EffectKind::InvisibleBombs,
        ]
        .into_iter()
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
        })
        .chain((0..6).map(|_| EffectView::new(EffectKind::SlowFoot, 600).unwrap()))
        .collect(),
    )
    .unwrap();
    let visible: ActorView = VisibleActorView {
        id: EntityId::new(u32::MAX),
        seat: Seat::One,
        position: Position::new(15_359, 13_311).unwrap(),
        heading: Direction::North,
        speed_level: 4,
        alive: false,
        bomb_capacity: 8,
        bomb_range: 8,
        live_bomb_count: 8,
        inventory: InventoryView {
            equipment: super::EquipmentId::Bracelet.into(),
            weapon: super::WeaponId::PiercingBomb.into(),
        },
        effects,
        damage_grace_remaining_ticks: DamageGraceTicks::new(60).unwrap(),
        voluntary_actions_frozen: false,
        last_teleport_tick: Some(tick),
    }
    .into();
    let all_cells = cells();
    let state = GameState {
        running_updates: 10_800,
        remaining_ticks: 10_800,
        physical_tick: tick,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![Terrain::Floor; ARENA_TERRAIN_CELLS],
            blocks: all_cells.clone(),
        },
        actors: [visible.clone(), visible],
        bombs: vec![bomb; 16],
        flames: all_cells
            .iter()
            .copied()
            .map(|cell| super::FlameView {
                cell,
                expires_at: tick,
            })
            .collect(),
        pickups: all_cells
            .into_iter()
            .map(|cell| PickupView {
                cell,
                kind: PickupKind::InvisibilityCloak,
            })
            .collect(),
    };
    let items = ItemSet::from_mask(ItemMask::new((1 << 22) - 1).unwrap());
    let room = RoomView {
        settings: GameSettings::new(4, 256, 9, 300, items).unwrap(),
        settings_revision: Counter::new(MAX_COUNTER).unwrap(),
        settings_editor: Some(Seat::Two),
        available_items: items,
        room_id: RoomId::new("ffffffffffffffffffffffffffffffff").unwrap(),
        room_code: RoomCode::new("ZZZZZZZZ").unwrap(),
        round_id: Counter::new(MAX_COUNTER).unwrap(),
        revision: Counter::new(MAX_COUNTER).unwrap(),
        tick,
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(MAX_COUNTER).unwrap(),
        scores: Scores {
            one: Wins::new(9).unwrap(),
            two: Wins::new(9).unwrap(),
        },
        last_round: Some(CompletedRound {
            round_id: Counter::new(MAX_COUNTER).unwrap(),
            round_number: Counter::new(MAX_COUNTER).unwrap(),
            ended_at: tick,
            result: WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            },
        }),
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: false,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Disconnected { expires_at: tick },
        }),
        phase: RoomPhase::MatchResult {
            ended_at: tick,
            result: MatchResult::Winner {
                seat: Seat::Two,
                reason: MatchWinnerReason::RoundWins,
            },
            rematch_at: tick,
        },
    };
    (room, state, bomb)
}

fn assert_representative_frame(message: &GameServerMessage, label: &str) -> usize {
    let encoded = encode_server_message(message).unwrap();
    let value: Value = serde_json::from_str(&encoded).unwrap();
    assert!(schema().is_valid(&value), "schema rejected {label}");
    assert!(
        encoded.len() < 65_536,
        "{label} encoded {} bytes",
        encoded.len()
    );
    eprintln!(
        "Schema-valid v10 representative {label}: {} UTF-8 bytes",
        encoded.len()
    );
    encoded.len()
}

#[test]
fn schema_valid_representative_snapshot_and_admitted_fit_the_frame() {
    let (room, state, bomb) = representative_body();
    let event = WireEvent {
        tick: room.tick,
        index: EventIndex::new(95).unwrap(),
        kind: EventKind::BombDetonated { bomb },
    };
    let snapshot = GameServerMessage::Snapshot {
        room: room.clone(),
        state: state.clone(),
        events: SnapshotEvents::new(vec![event; 96]).unwrap(),
    };
    let admitted = GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room: room.clone(),
        you_seat: Seat::Two,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_action_id: Some(Counter::new(MAX_COUNTER).unwrap()),
        state: Some(state.clone()),
    };
    let snapshot_size = assert_representative_frame(&snapshot, "snapshot");
    let admitted_size = assert_representative_frame(&admitted, "admitted");
    assert!(snapshot_size < 65_536);
    assert!(admitted_size < 65_536);

    let nullable = GameServerMessage::Admitted {
        sequence: Counter::new(1).unwrap(),
        room,
        you_seat: Seat::One,
        resume_token: ResumeToken::new("a".repeat(64)).unwrap(),
        last_action_id: None,
        state: None,
    };
    let value = serde_json::to_value(nullable).unwrap();
    assert_eq!(value["lastActionId"], Value::Null);
    assert_eq!(value["state"], Value::Null);
    assert!(schema().is_valid(&value));
    for field in ["lastActionId", "state"] {
        let mut missing = value.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(!schema().is_valid(&missing));
    }
}

#[test]
fn encoder_rejects_an_oversized_message_without_truncating() {
    let (room, mut state, bomb) = representative_body();
    state.bombs = vec![bomb; 1_024];
    let oversized = GameServerMessage::Snapshot {
        room,
        state,
        events: SnapshotEvents::default(),
    };
    assert_eq!(
        encode_server_message(&oversized),
        Err(EncodeServerMessageError::MessageTooLarge)
    );
}
