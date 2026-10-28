use jsonschema::validator_for;
use serde_json::Value;

use super::*;
use crate::game::{ArenaGenerationConfig, EnabledItems, RoundAssignment};
use crate::protocol::{Cell, Counter, EntityId, MAX_COUNTER, MAX_SAFE_TICK, Tick};

fn maximal_state() -> (RoomView, GameState, BombView) {
    let mut domain = super::tests::multiplayer_state(crate::game::PlayerCount::Four);
    let report = domain.step(&[]).unwrap();
    let capture = capture_expanded(&domain, &[report]).unwrap();
    let room =
        super::tests::running_room(crate::game::PlayerCount::Four, capture.world.physical_tick);
    let generation = crate::game::generate_multiplayer_arena(
        [9; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: crate::game::PlayerCount::Four,
            board: crate::room::BoardSize::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap()
    .record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    let mut state = project_recipient(&publication, Seat::One).unwrap().state;
    let tick = Tick::new(MAX_SAFE_TICK).unwrap();
    let cells: Vec<_> = (0..13)
        .flat_map(|row| (0..15).map(move |column| Cell::new(column, row).unwrap()))
        .collect();
    let bomb = BombView {
        id: EntityId::new(u32::MAX),
        cell: Cell::new(14, 12).unwrap(),
        owner: EntityId::new(10),
        range: 8,
        placed_at: tick,
        trigger: BombTrigger::timed(600, true).unwrap(),
        blast: BlastKind::Piercing,
    };
    state.running_updates = 10_800;
    state.remaining_ticks = 10_800;
    state.physical_tick = tick;
    state.arena.terrain = vec![Terrain::Floor; 195];
    state.arena.blocks = cells.clone();
    state.bombs = vec![bomb; MAX_BOMBS];
    state.flames = cells
        .iter()
        .copied()
        .map(|cell| FlameView {
            cell,
            expires_at: tick,
        })
        .collect();
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
        .into_iter()
        .map(|cell| PickupView {
            cell,
            kind: PickupKind::InvisibilityCloak,
        })
        .collect();
    (room, state, bomb)
}

fn assert_schema_valid(message: &GameServerMessage) -> usize {
    let encoded = encode_server_message(message).unwrap();
    let value: Value = serde_json::from_str(&encoded).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v14.schema.json")).unwrap();
    assert!(validator_for(&schema).unwrap().is_valid(&value));
    assert!(encoded.len() < MAX_SERVER_MESSAGE_BYTES);
    encoded.len()
}

#[test]
fn typed_four_actor_maximal_snapshot_and_admission_fit_v14_limit() {
    let (mut room, state, bomb) = maximal_state();
    room.settings_revision = Counter::new(MAX_COUNTER).unwrap();
    room.revision = Counter::new(MAX_COUNTER).unwrap();
    let events = (0..MAX_SNAPSHOT_EVENTS)
        .map(|index| WireEvent {
            tick: Tick::new(MAX_SAFE_TICK).unwrap(),
            index: EventIndex::new(index as u8).unwrap(),
            kind: EventKind::BombDetonated { bomb },
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
        you_seat: Seat::Four,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_action_id: Some(Counter::new(MAX_COUNTER).unwrap()),
        state: Some(state),
    };
    assert!(assert_schema_valid(&snapshot) < MAX_SERVER_MESSAGE_BYTES);
    assert!(assert_schema_valid(&admitted) < MAX_SERVER_MESSAGE_BYTES);
}
