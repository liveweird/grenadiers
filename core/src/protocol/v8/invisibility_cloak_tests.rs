use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, EnabledItems, EventAudience, InputAction, PickupKind, RulesConfig, SeatId,
    SeatInput, State, World,
};
use crate::room::RoomSettings;

use super::super::*;

fn cloak_state(both: bool) -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let exposed = if both {
        BTreeMap::from([
            (spawns[0], PickupKind::InvisibilityCloak),
            (spawns[1], PickupKind::InvisibilityCloak),
        ])
    } else {
        BTreeMap::from([(spawns[0], PickupKind::InvisibilityCloak)])
    };
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::INVISIBILITY_CLOAK),
        ),
        world,
        SeatId::DUEL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn publication_window_state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let exposed = BTreeMap::from([
        (spawns[0], PickupKind::InvisibilityCloak),
        (game::CellId::new(3, 1), PickupKind::BombCapacity),
    ]);
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::INVISIBILITY_CLOAK),
        ),
        world,
        SeatId::DUEL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn cloak_room(tick: u64) -> RoomView {
    let mut room = super::classic_room(tick);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::InvisibilityCloak]).unwrap());
    room
}

fn publication(state: &State, reports: &[game::StepReport]) -> BoundPublication {
    let mut room = cloak_room(state.tick().0);
    if let Some(result) = state.result() {
        let result = match result {
            game::RoundResult::Winner(seat) => crate::protocol::WireRoundResult::Winner {
                seat: crate::protocol::Seat::try_from(seat).unwrap(),
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
        if result
            == (crate::protocol::WireRoundResult::Winner {
                seat: crate::protocol::Seat::Two,
            })
        {
            room.scores.two = Wins::new(1).unwrap();
        }
        let ended_at = crate::protocol::Tick::new(state.tick().0).unwrap();
        room.last_round = Some(CompletedRound {
            round_id: room.round_id,
            round_number: room.round_id,
            ended_at,
            result,
        });
        room.phase = RoomPhase::Result {
            hold_until: crate::protocol::Tick::new(state.tick().0 + 180).unwrap(),
        };
    }
    let record = game::generate_expanded_arena([0; 32], EnabledItems::INVISIBILITY_CLOAK).record();
    let capture = capture_expanded(state, reports).unwrap();
    bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap()
}

#[test]
fn owner_is_fully_visible_while_opponent_gets_exact_hidden_actor_and_no_private_cue() {
    let mut state = cloak_state(false);
    let collected = state.step(&[]).unwrap();
    let placed = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let frames = prepare_snapshot_pair(&publication(&state, &[collected, placed])).unwrap();
    let owner: serde_json::Value = serde_json::from_str(&frames[0]).unwrap();
    let opponent: serde_json::Value = serde_json::from_str(&frames[1]).unwrap();
    assert_eq!(owner["state"]["actors"][0]["type"], "visible");
    assert_eq!(
        owner["state"]["actors"][0]["effects"][0]["kind"],
        "invisibilityCloak"
    );
    assert_eq!(owner["events"].as_array().unwrap().len(), 2);
    assert_eq!(
        opponent["state"]["actors"][0],
        serde_json::json!({
            "type": "hidden", "id": 1, "seat": "one", "alive": true
        })
    );
    assert!(opponent["events"].as_array().unwrap().is_empty());
    assert_eq!(owner["state"]["bombs"], opponent["state"]["bombs"]);
    assert_eq!(opponent["state"]["bombs"].as_array().unwrap().len(), 1);
}

#[test]
fn both_cloaked_players_see_self_and_only_their_own_collection_event() {
    let mut state = cloak_state(true);
    let report = state.step(&[]).unwrap();
    let frames = prepare_snapshot_pair(&publication(&state, &[report])).unwrap();
    for (index, frame) in frames.iter().enumerate() {
        let value: serde_json::Value = serde_json::from_str(frame).unwrap();
        assert_eq!(value["state"]["actors"][index]["type"], "visible");
        assert_eq!(value["state"]["actors"][1 - index]["type"], "hidden");
        assert_eq!(value["events"].as_array().unwrap().len(), 1);
        assert_eq!(
            value["events"][0]["kind"]["seat"],
            if index == 0 { "one" } else { "two" }
        );
    }
}

#[test]
fn death_clears_cloak_and_publishes_the_dead_actor_to_both_recipients() {
    let mut state = cloak_state(false);
    state.step(&[]).unwrap();
    let cell = state.actors()[0].position().cell().unwrap();
    state.insert_test_flame(cell, game::Tick(5));
    let death = state.step(&[]).unwrap();
    let frames = prepare_snapshot_pair(&publication(&state, &[death])).unwrap();
    for frame in frames {
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(value["state"]["actors"][0]["type"], "visible");
        assert_eq!(value["state"]["actors"][0]["alive"], false);
        assert_eq!(value["events"][0]["kind"]["type"], "actorEliminated");
    }
}

#[test]
fn binding_rejects_neither_and_wrong_seat_audiences_without_recomputing_history() {
    for audience in [EventAudience::Neither, EventAudience::Two] {
        let mut state = cloak_state(false);
        let mut report = state.step(&[]).unwrap();
        report.events[0].audience = audience;
        let room = cloak_room(1);
        let record =
            game::generate_expanded_arena([0; 32], EnabledItems::INVISIBILITY_CLOAK).record();
        let capture = capture_expanded(&state, &[report]).unwrap();
        assert_eq!(
            bind_expanded(&room, room.round_id, &capture, Some(record)),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }

    let mut state = cloak_state(false);
    let mut report = state.step(&[]).unwrap();
    report.events[0].audience = EventAudience::Both;
    assert!(publication(&state, &[report]).reports.0[0].events[0].audience == EventAudience::Both);
}

#[test]
fn three_report_window_retains_private_preexpiry_and_public_expiry_and_death_events() {
    let mut state = publication_window_state();
    state.step(&[]).unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(game::Direction::East),
        }])
        .unwrap();
    for _ in 1..20 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Release(game::Direction::East),
        }])
        .unwrap();

    let mut probe = state.clone();
    let probe_start = probe.tick().0;
    let mut probe_report = probe
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(game::Direction::East),
        }])
        .unwrap();
    while !probe_report.events.iter().any(|event| {
        matches!(
            event.kind,
            game::StepEventKind::PickupCollected {
                kind: PickupKind::BombCapacity,
                ..
            }
        )
    }) {
        probe_report = probe.step(&[]).unwrap();
    }
    let travel_ticks = probe_report.tick.0 - probe_start;
    while state.tick().0 < 600 - travel_ticks {
        state.step(&[]).unwrap();
    }
    let mut private = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(game::Direction::East),
        }])
        .unwrap();
    while private.tick.0 < 600 {
        private = state.step(&[]).unwrap();
    }
    assert!(private.events.iter().any(|event| {
        event.audience == EventAudience::One
            && matches!(
                event.kind,
                game::StepEventKind::PickupCollected {
                    kind: PickupKind::BombCapacity,
                    ..
                }
            )
    }));

    let public = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    assert_eq!(public.tick, game::Tick(601));
    assert!(public.events.iter().any(|event| {
        event.audience == EventAudience::Both
            && matches!(event.kind, game::StepEventKind::BombPlaced(_))
    }));
    let cell = state.actors()[0].position().cell().unwrap();
    state.insert_test_flame(cell, game::Tick(605));
    let death = state.step(&[]).unwrap();
    assert_eq!(death.tick, game::Tick(602));
    assert!(
        death
            .events
            .iter()
            .all(|event| event.audience == EventAudience::Both)
    );

    let publication = publication(&state, &[private, public, death]);
    let frames = prepare_snapshot_pair(&publication).unwrap();
    let owner: serde_json::Value = serde_json::from_str(&frames[0]).unwrap();
    let opponent: serde_json::Value = serde_json::from_str(&frames[1]).unwrap();
    assert_eq!(owner["events"].as_array().unwrap().len(), 5);
    assert_eq!(opponent["events"].as_array().unwrap().len(), 4);
    assert_eq!(owner["events"][0]["kind"]["type"], "pickupCollected");
    assert_eq!(opponent["events"][0]["kind"]["type"], "bombPlaced");
    assert_eq!(opponent["events"][0]["tick"], 601);
    assert_eq!(opponent["events"][1]["kind"]["type"], "actorEliminated");
    assert_eq!(
        owner["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 2]
    );
    assert_eq!(
        opponent["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 0, 1, 2]
    );
    assert_eq!(owner["state"]["bombs"], opponent["state"]["bombs"]);
    assert_eq!(owner["state"]["arena"], opponent["state"]["arena"]);
    assert_eq!(owner["state"]["flames"], opponent["state"]["flames"]);
    assert_eq!(owner["state"]["pickups"], opponent["state"]["pickups"]);
    assert_eq!(owner["state"]["actors"][0]["alive"], false);
    assert_eq!(opponent["state"]["actors"][0]["type"], "visible");
}
