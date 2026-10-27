use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, EffectKind, EnabledItems, EventAudience, InputAction, PickupKind, RulesConfig,
    SeatId, SeatInput, State, World,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat, Tick};
use crate::room::RoomSettings;

use super::*;

fn state(exposed: BTreeMap<game::CellId, PickupKind>) -> State {
    state_with(exposed, EnabledItems::INVISIBLE_BOMBS)
}

fn state_with(exposed: BTreeMap<game::CellId, PickupKind>, enabled: EnabledItems) -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap(),
        SeatId::DUEL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 10),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn room(tick: u64, enabled: ItemSet) -> RoomView {
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, enabled).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: super::expanded_items(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(tick).unwrap(),
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

fn publication(state: &State, reports: &[game::StepReport]) -> BoundPublication {
    let capture = capture_expanded(state, reports).unwrap();
    let mut room = room(state.tick().0, capture.world.0.enabled_items);
    if let Some(result) = state.result() {
        let result = match result {
            game::RoundResult::Winner(seat) => crate::protocol::WireRoundResult::Winner {
                seat: Seat::try_from(seat).unwrap(),
            },
            game::RoundResult::Draw(game::DrawReason::MutualElimination) => {
                crate::protocol::WireRoundResult::Draw {
                    reason: crate::protocol::DrawReason::MutualElimination,
                }
            }
            game::RoundResult::Draw(game::DrawReason::Timeout) => {
                crate::protocol::WireRoundResult::Draw {
                    reason: crate::protocol::DrawReason::Timeout,
                }
            }
        };
        if result == (crate::protocol::WireRoundResult::Winner { seat: Seat::Two }) {
            room.scores.two = Wins::new(1).unwrap();
        }
        let ended_at = Tick::new(state.tick().0).unwrap();
        room.last_round = Some(CompletedRound {
            round_id: room.round_id,
            round_number: room.round_id,
            ended_at,
            result,
        });
        room.phase = RoomPhase::Result {
            hold_until: Tick::new(state.tick().0 + 180).unwrap(),
        };
    }
    let enabled = EnabledItems::from_bits(capture.world.0.enabled_items.mask().bits()).unwrap();
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap()
}

fn place(state: &mut State, seat: SeatId) -> game::StepReport {
    state
        .step(&[SeatInput {
            seat,
            action: InputAction::PlaceBomb,
        }])
        .unwrap()
}

#[test]
fn each_blinded_recipient_gets_no_live_bombs_or_cues_while_counts_stay_truthful() {
    let spawns = game::fixed_arena().spawns().to_vec();
    let mut state = state(BTreeMap::from([
        (spawns[0], PickupKind::InvisibleBombs),
        (spawns[1], PickupKind::InvisibleBombs),
    ]));
    state.step(&[]).unwrap();
    let report = place(&mut state, SeatId::Two);
    assert_eq!(report.events[0].audience, EventAudience::Neither);
    let frames = prepare_snapshot_pair(&publication(&state, &[report])).unwrap();
    for (index, frame) in frames.into_iter().enumerate() {
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert!(value["state"]["bombs"].as_array().unwrap().is_empty());
        assert!(value["events"].as_array().unwrap().is_empty());
        assert_eq!(value["state"]["actors"][index]["liveBombCount"], index);
    }
}

#[test]
fn one_blinded_recipient_hides_every_bomb_kind_and_paused_timing_without_hiding_fire() {
    let spawns = game::fixed_arena().spawns().to_vec();
    let enabled = EnabledItems::from_bits(
        EnabledItems::INVISIBLE_BOMBS.bits() | EnabledItems::STOPWATCH.bits(),
    )
    .unwrap();
    let mut state = state_with(
        BTreeMap::from([(spawns[0], PickupKind::InvisibleBombs)]),
        enabled,
    );
    state.step(&[]).unwrap();
    let placed = place(&mut state, SeatId::Two);
    state.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, state.tick());
    state.step(&[]).unwrap();
    let cell = state.world().bombs().values().next().unwrap().cell();
    state.insert_test_flame(game::CellId::new(3, 1), game::Tick(state.tick().0 + 3));
    let frames = prepare_snapshot_pair(&publication(&state, &[placed])).unwrap();
    let blind: serde_json::Value = serde_json::from_str(&frames[0]).unwrap();
    let sighted: serde_json::Value = serde_json::from_str(&frames[1]).unwrap();
    assert!(blind["state"]["bombs"].as_array().unwrap().is_empty());
    assert_eq!(blind["state"]["actors"][1]["liveBombCount"], 1);
    assert_eq!(blind["state"]["flames"].as_array().unwrap().len(), 1);
    assert!(blind["events"].as_array().unwrap().is_empty());
    assert_eq!(sighted["state"]["bombs"].as_array().unwrap().len(), 1);
    assert_eq!(
        sighted["state"]["bombs"][0]["cell"],
        serde_json::json!({"column":cell.column,"row":cell.row})
    );
    assert_eq!(sighted["state"]["bombs"][0]["trigger"]["paused"], true);
    assert_eq!(sighted["events"].as_array().unwrap().len(), 1);
}

#[test]
fn captured_history_does_not_follow_final_blindness_or_reappear_after_death() {
    let spawns = game::fixed_arena().spawns().to_vec();
    let mut state = state(BTreeMap::from([(spawns[0], PickupKind::InvisibleBombs)]));
    let public_before_collection = place(&mut state, SeatId::Two);
    let collected = state.step(&[]).unwrap();
    let hidden_after_collection = place(&mut state, SeatId::One);
    let actor_cell = state.actors()[0].position().cell().unwrap();
    state.insert_test_flame(actor_cell, game::Tick(state.tick().0 + 2));
    let death = state.step(&[]).unwrap();
    assert!(!state.actors()[0].alive());
    let frames = prepare_snapshot_pair(&publication(
        &state,
        &[collected, hidden_after_collection, death],
    ))
    .unwrap();
    let formerly_blind: serde_json::Value = serde_json::from_str(&frames[0]).unwrap();
    let opponent: serde_json::Value = serde_json::from_str(&frames[1]).unwrap();
    assert_eq!(opponent["state"]["actors"][0]["alive"], false);
    assert_eq!(
        formerly_blind["state"]["bombs"].as_array().unwrap().len(),
        1
    );
    assert_eq!(formerly_blind["state"]["bombs"][0]["owner"], 11);
    assert_eq!(opponent["state"]["bombs"].as_array().unwrap().len(), 1);
    assert_eq!(opponent["state"]["bombs"][0]["owner"], 11);
    assert!(
        formerly_blind["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| {
                event["kind"]["type"] != "bombPlaced" || event["kind"]["bomb"]["owner"] != 10
            })
    );

    let public = publication(&state, &[public_before_collection]);
    let after_collection: serde_json::Value =
        serde_json::from_str(&prepare_snapshot_pair(&public).unwrap()[0]).unwrap();
    assert_eq!(after_collection["events"][0]["kind"]["type"], "bombPlaced");
}

#[test]
fn binding_rejects_blind_audiences_without_capability_and_invalid_hidden_facts() {
    let mut ordinary = state(BTreeMap::new());
    let mut report = place(&mut ordinary, SeatId::One);
    report.events[0].audience = EventAudience::Neither;
    let mut capture = capture_expanded(&ordinary, &[report]).unwrap();
    capture.world.0.enabled_items = ItemSet::empty();
    capture.world.0.available_items = super::expanded_items();
    let no_item_room = room(ordinary.tick().0, ItemSet::empty());
    assert_eq!(
        bind_expanded(
            &no_item_room,
            no_item_room.round_id,
            &capture,
            Some(
                game::generate_expanded_arena([0; 32], EnabledItems::from_bits(0).unwrap())
                    .record()
            ),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let spawns = game::fixed_arena().spawns().to_vec();
    let mut blinded = state(BTreeMap::from([(spawns[0], PickupKind::InvisibleBombs)]));
    blinded.step(&[]).unwrap();
    place(&mut blinded, SeatId::Two);
    let mut capture = capture_expanded(&blinded, &[]).unwrap();
    capture.world.0.bombs[0].owner = crate::protocol::EntityId::new(999);
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world(capture.world.0),
        Err(ProjectionError::InvalidBombs)
    );
}
