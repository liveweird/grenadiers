use crate::game::{
    self, ActorInit, Direction, EffectKind, EnabledItems, EntityId, InputAction, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, State, World,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat, Tick};
use crate::room::RoomSettings;

use super::*;

fn state(enabled: EnabledItems) -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn place(state: &mut State) -> game::StepReport {
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
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

#[test]
fn v9_captures_live_pause_history_while_v8_rejects_the_resumed_alias() {
    let mut active = state(EnabledItems::STOPWATCH);
    place(&mut active);
    let at = active.tick();
    active.apply_test_effect(SeatId::One, EffectKind::Stopwatch, at);
    active.step(&[]).unwrap();
    let capture = capture_expanded(&active, &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    let timing = capture.world.0.timed_bombs[&bomb.id];
    assert!(timing.paused);
    assert_eq!(timing.paused_ticks, 1);
    assert_eq!(timing.captured_fuse_ticks, 180);

    let mut resumed = state(EnabledItems::from_bits(0).unwrap());
    place(&mut resumed);
    let at = resumed.tick();
    resumed.apply_test_effect(SeatId::One, EffectKind::Stopwatch, at);
    resumed.step(&[]).unwrap();
    resumed.clear_test_effects(SeatId::One);
    assert_eq!(
        crate::protocol::v8::capture_expanded(&resumed, &[]),
        Err(ProjectionError::InvalidBombs)
    );
}

#[test]
fn event_only_pause_facts_keep_their_emission_context_after_holder_expiry() {
    let mut event_state = state(EnabledItems::STOPWATCH);
    place(&mut event_state);
    let at = event_state.tick();
    event_state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, at);
    let cell = event_state.world().bombs().values().next().unwrap().cell();
    event_state.insert_test_flame(cell, game::Tick(event_state.tick().0 + 2));
    let report = event_state.step(&[]).unwrap();
    assert!(event_state.world().bombs().is_empty());
    event_state.clear_test_effects(SeatId::One);
    let capture = capture_expanded(&event_state, &[report]).unwrap();
    let event = capture.reports.0[0]
        .events
        .iter()
        .find(|event| matches!(event.kind, EventKind::BombDetonated { .. }))
        .unwrap();
    assert!(event.timed_bomb.unwrap().paused);
    assert_eq!(event.timed_bomb.unwrap().paused_ticks, 1);
}

#[test]
fn missing_or_extra_timing_facts_are_rejected_before_recipient_filtering() {
    let mut live_state = state(EnabledItems::STOPWATCH);
    place(&mut live_state);
    let mut capture = capture_expanded(&live_state, &[]).unwrap();
    capture.world.0.timed_bombs.clear();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world(capture.world.0),
        Err(ProjectionError::InvalidBombs)
    );

    let mut state = state(EnabledItems::STOPWATCH);
    let report = place(&mut state);
    let mut capture = capture_expanded(&state, &[report]).unwrap();
    capture.reports.0[0].events[0].timed_bomb = None;
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &capture.world,
            capture.reports.0,
        ),
        Err(ProjectionError::InvalidEvent)
    );
}

#[test]
fn frozen_status_is_derived_from_both_full_actors_before_cloak_filtering() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::STOPWATCH.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap();
    let mut state = state(enabled);
    let at = state.tick();
    state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, at);
    state.apply_test_effect(SeatId::One, EffectKind::InvisibilityCloak, at);
    state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = room(
        state.tick().0,
        ItemSet::new([ItemId::Stopwatch, ItemId::InvisibilityCloak]).unwrap(),
    );
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    let frames = prepare_snapshot_pair(&publication).unwrap();
    let holder: serde_json::Value = serde_json::from_str(&frames[0]).unwrap();
    let victim: serde_json::Value = serde_json::from_str(&frames[1]).unwrap();
    assert_eq!(
        holder["state"]["actors"][0]["voluntaryActionsFrozen"],
        false
    );
    assert_eq!(holder["state"]["actors"][1]["voluntaryActionsFrozen"], true);
    assert_eq!(
        victim["state"]["actors"][0],
        serde_json::json!({"type":"hidden","id":10,"seat":"one","alive":true})
    );
    assert_eq!(victim["state"]["actors"][1]["voluntaryActionsFrozen"], true);
}
