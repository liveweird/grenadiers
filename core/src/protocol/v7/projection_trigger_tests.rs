use crate::game::{EventAudience, StepEvent, StepEventKind};
use crate::protocol::{Cell, EntityId, RulesVersion, Tick};

use super::projection_facts::{ClassicCapture, EventFacts, ProjectionError, ReportFacts};
use super::projection_test_support::{base_world, classic_state_at, room};
use super::projection_validation::{validate_reports, validate_world};
use super::*;

fn bomb_report(
    state: &crate::game::State,
    placed_at: u64,
    detonates_at: u64,
    placed: bool,
) -> crate::game::StepReport {
    let tick = state.tick();
    let bomb = crate::game::BombView {
        id: crate::game::EntityId(3),
        cell: crate::game::CellId::new(1, 1),
        owner: crate::game::EntityId(1),
        range: 2,
        placed_at: crate::game::Tick(placed_at),
        trigger: crate::game::BombTrigger::Timed {
            detonates_at: crate::game::Tick(detonates_at),
        },
    };
    crate::game::StepReport {
        tick,
        traces: crate::game::SeatId::ALL
            .map(|seat| crate::game::MovementTrace::new(state.actors()[seat.index()].position())),
        events: vec![StepEvent::new(
            tick,
            EventAudience::Both,
            if placed {
                StepEventKind::BombPlaced(bomb)
            } else {
                StepEventKind::BombDetonated(bomb)
            },
        )],
    }
}

fn paused_bomb() -> BombView {
    BombView {
        id: EntityId::new(3),
        cell: Cell::new(2, 2).unwrap(),
        owner: EntityId::new(1),
        range: 2,
        placed_at: Tick::new(9).unwrap(),
        trigger: BombTrigger::timed(10, true).unwrap(),
        blast: BlastKind::Standard,
    }
}

#[test]
fn event_timer_rejects_overdue_and_preserves_equal_and_positive_chain_deadlines() {
    let state = classic_state_at(179);
    assert_eq!(
        capture_classic(&state, &[bomb_report(&state, 0, 178, false)]),
        Err(ProjectionError::InvalidEvent)
    );

    for (event_tick, expected_remaining) in [(179, 1), (180, 0)] {
        let state = classic_state_at(event_tick);
        let capture = capture_classic(&state, &[bomb_report(&state, 0, 180, false)]).unwrap();
        let room = room(
            event_tick,
            GameSettings::for_rules(RulesVersion::ClassicV1),
            ItemSet::empty(),
        );
        let publication = bind_classic(&room, room.round_id, &capture).unwrap();
        let [one, two] = prepare_snapshot_pair(&publication).unwrap();
        assert_eq!(one, two);
        let value: serde_json::Value = serde_json::from_str(&one).unwrap();
        assert_eq!(
            value["events"][0]["kind"]["bomb"]["trigger"]["remainingTicks"],
            expected_remaining
        );
    }
}

#[test]
fn classic_binding_rejects_wrong_fuse_in_placement_detonation_and_world_records() {
    let state = classic_state_at(10);
    let room = room(
        10,
        GameSettings::for_rules(RulesVersion::ClassicV1),
        ItemSet::empty(),
    );
    for report in [
        bomb_report(&state, 10, 11, true),
        bomb_report(&state, 0, 181, false),
    ] {
        let capture = capture_classic(&state, &[report]).unwrap();
        assert_eq!(
            bind_classic(&room, room.round_id, &capture),
            Err(ProjectionError::InvalidClassicBinding)
        );
    }

    let mut state = classic_state_at(10);
    crate::game::test_support::add_test_bomb_with_deadline(
        &mut state,
        crate::game::EntityId(3),
        crate::game::CellId::new(1, 1),
        crate::game::EntityId(1),
        2,
        crate::game::Tick(0),
        crate::game::Tick(11),
    )
    .unwrap();
    let capture = capture_classic(&state, &[]).unwrap();
    assert_eq!(
        bind_classic(&room, room.round_id, &capture),
        Err(ProjectionError::InvalidClassicBinding)
    );
}

#[test]
fn classic_binding_rejects_overdue_frozen_world_bombs_but_accepts_exact_boundary() {
    let mut finished = classic_state_at(180);
    for seat in crate::game::SeatId::ALL {
        let cell = finished.actors()[seat.index()].position().cell().unwrap();
        finished.insert_test_flame(cell, crate::game::Tick(182));
    }
    finished.step(&[]).unwrap();
    assert!(finished.result().is_some());

    let mut result_room = room(
        200,
        GameSettings::for_rules(RulesVersion::ClassicV1),
        ItemSet::empty(),
    );
    let result = crate::protocol::WireRoundResult::Draw {
        reason: crate::protocol::DrawReason::MutualElimination,
    };
    result_room.last_round = Some(CompletedRound {
        round_id: result_room.round_id,
        round_number: result_room.round_id,
        ended_at: Tick::new(181).unwrap(),
        result,
    });
    result_room.phase = RoomPhase::Result {
        hold_until: Tick::new(361).unwrap(),
    };

    for (placed_at, expected) in [(0, false), (1, true)] {
        let mut state = finished.clone();
        crate::game::test_support::add_test_bomb_with_deadline(
            &mut state,
            crate::game::EntityId(3),
            crate::game::CellId::new(2, 1),
            crate::game::EntityId(1),
            2,
            crate::game::Tick(placed_at),
            crate::game::Tick(placed_at + 180),
        )
        .unwrap();
        let capture = capture_classic(&state, &[]).unwrap();
        assert_eq!(
            bind_classic(&result_room, result_room.round_id, &capture).is_ok(),
            expected
        );
        if expected {
            let encoded = prepare_classic_admission(
                &result_room,
                Some(result_room.round_id),
                Some(&capture),
                crate::protocol::Seat::One,
                None,
                crate::protocol::Counter::new(1).unwrap(),
                ResumeToken::new("c".repeat(64)).unwrap(),
            )
            .unwrap();
            assert!(
                !serde_json::from_str::<serde_json::Value>(&encoded).unwrap()["state"].is_null()
            );
        }
    }
}

#[test]
fn paused_world_bombs_require_enabled_stopwatch_and_still_fail_classic_binding() {
    let stopwatch = ItemSet::new([ItemId::Stopwatch]).unwrap();
    let mut facts = base_world(10);
    facts.actors[0].live_bomb_count = 1;
    facts.bombs.push(paused_bomb());
    assert_eq!(
        validate_world(facts.clone()),
        Err(ProjectionError::InvalidItems)
    );

    facts.available_items = stopwatch;
    assert_eq!(
        validate_world(facts.clone()),
        Err(ProjectionError::InvalidItems)
    );
    facts.enabled_items = stopwatch;
    let validated = validate_world(facts).unwrap();

    let original = capture_classic(&classic_state_at(10), &[]).unwrap();
    let capture = ClassicCapture {
        world: validated,
        ..original
    };
    let matching = room(
        10,
        GameSettings::new(0, 128, 3, 180, stopwatch).unwrap(),
        stopwatch,
    );
    assert_eq!(
        bind_classic(&matching, matching.round_id, &capture),
        Err(ProjectionError::InvalidClassicBinding)
    );
}

#[test]
fn paused_event_bombs_require_stopwatch_enabled_at_the_publication_boundary() {
    let stopwatch = ItemSet::new([ItemId::Stopwatch]).unwrap();
    let report = ReportFacts {
        tick: Tick::new(10).unwrap(),
        events: vec![EventFacts {
            tick: Tick::new(10).unwrap(),
            audience: EventAudience::Both,
            kind: EventKind::BombDetonated {
                bomb: paused_bomb(),
            },
        }],
    };

    let missing = validate_world(base_world(10)).unwrap();
    assert_eq!(
        validate_reports(&missing, vec![report.clone()]),
        Err(ProjectionError::InvalidItems)
    );
    let mut available_only = base_world(10);
    available_only.available_items = stopwatch;
    let available_only = validate_world(available_only).unwrap();
    assert_eq!(
        validate_reports(&available_only, vec![report.clone()]),
        Err(ProjectionError::InvalidItems)
    );
    let mut enabled = base_world(10);
    enabled.available_items = stopwatch;
    enabled.enabled_items = stopwatch;
    let enabled = validate_world(enabled).unwrap();
    assert!(validate_reports(&enabled, vec![report]).is_ok());
}
