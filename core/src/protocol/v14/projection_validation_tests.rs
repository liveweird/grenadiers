use super::*;
use crate::protocol::{Cell, EntityId, Tick};

pub(super) fn valid_capture(count: crate::game::PlayerCount) -> projection_facts::ExpandedCapture {
    let mut state = tests::multiplayer_state(count);
    let report = state.step(&[]).unwrap();
    capture_expanded(&state, &[report]).unwrap()
}

pub(super) fn assert_valid(capture: &projection_facts::ExpandedCapture) {
    projection_validation::validate(&capture.world, &capture.reports, &capture.rules).unwrap();
}

fn round_state(
    count: crate::game::PlayerCount,
    participating: &[crate::game::SeatId],
    enabled: crate::game::EnabledItems,
) -> crate::game::State {
    let template = tests::multiplayer_state_with_items(count, enabled);
    let roster = template
        .actors()
        .iter()
        .rev()
        .map(|actor| crate::game::RoundActorInit {
            actor: crate::game::ActorInit {
                id: actor.id(),
                seat: actor.seat(),
                position: actor.position(),
                heading: actor.heading(),
                speed: actor.speed(),
            },
            participating: participating.contains(&actor.seat()),
        })
        .collect::<Vec<_>>();
    crate::game::State::new_round(
        *template.rules(),
        template.world().clone(),
        roster,
        crate::game::Tick(0),
        crate::game::RoundAssignment::first(),
        Some(crate::game::SimulationSeed::new([22; 32])),
    )
    .unwrap()
}

pub(super) fn generation(
    count: crate::game::PlayerCount,
    enabled: crate::game::EnabledItems,
) -> crate::game::GenerationRecord {
    crate::game::generate_multiplayer_arena(
        [22; 32],
        enabled,
        crate::game::ArenaGenerationConfig {
            player_count: count,
            board: crate::room::BoardSize::FifteenByThirteen,
            assignment: crate::game::RoundAssignment::first(),
        },
    )
    .unwrap()
    .record()
}

fn assert_bound_delivery(
    count: crate::game::PlayerCount,
    capture: &projection_facts::ExpandedCapture,
    enabled: crate::game::EnabledItems,
) {
    let mut room = tests::running_room(count, capture.world.physical_tick);
    room.settings = GameSettings::new(0, 256, 3, 180, capture.world.enabled_items).unwrap();
    let generation = generation(count, enabled);
    let publication = bind_expanded(&room, room.round_id, capture, Some(generation)).unwrap();
    assert_eq!(prepare_snapshots(&publication).unwrap().len(), count.get());
    let admitted = prepare_admission(
        AdmissionProjection {
            room: &room,
            state_epoch: Some(room.round_id),
            capture: Some(capture),
            generation: Some(generation),
        },
        PersonalAdmission {
            recipient: Seat::One,
            last_action_id: None,
            sequence: crate::protocol::Counter::new(1).unwrap(),
            resume_token: ResumeToken::new("a".repeat(64)).unwrap(),
        },
    )
    .unwrap();
    assert!(admitted.contains("\"type\":\"admitted\""));
}

#[test]
fn physical_facade_accepts_canonical_lineups() {
    for count in [
        crate::game::PlayerCount::Two,
        crate::game::PlayerCount::Three,
        crate::game::PlayerCount::Four,
    ] {
        let capture = valid_capture(count);
        assert_valid(&capture);
    }
}

#[test]
fn real_inactive_canonical_actors_capture_bind_and_deliver() {
    for (count, participating) in [
        (
            crate::game::PlayerCount::Three,
            vec![crate::game::SeatId::One, crate::game::SeatId::Three],
        ),
        (
            crate::game::PlayerCount::Four,
            vec![crate::game::SeatId::Two, crate::game::SeatId::Four],
        ),
    ] {
        let state = round_state(count, &participating, crate::game::EnabledItems::ALL);
        let capture = capture_expanded(&state, &[]).unwrap();
        assert_valid(&capture);
        assert_eq!(
            capture
                .world
                .actors
                .iter()
                .filter(|actor| actor.view.alive)
                .count(),
            2
        );
        let dead = capture
            .world
            .actors
            .iter()
            .find(|actor| !actor.view.alive)
            .unwrap();
        assert!(dead.view.inventory.equipment.is_none());
        assert!(dead.view.effects.as_slice().is_empty());
        assert!(dead.soft_wall_grace.is_empty());
        assert!(dead.unlit_oil_grace.is_empty());
        assert_bound_delivery(count, &capture, crate::game::EnabledItems::ALL);
    }
}

#[test]
fn real_departures_with_live_owned_bombs_capture_bind_and_deliver() {
    for (count, departed) in [
        (crate::game::PlayerCount::Three, [true, false, false, false]),
        (crate::game::PlayerCount::Four, [true, true, false, false]),
    ] {
        let seats = count.seats();
        let mut state = round_state(count, seats, crate::game::EnabledItems::EMPTY);
        let mut report = state
            .step(&[crate::game::SeatInput {
                seat: crate::game::SeatId::One,
                action: crate::game::InputAction::PlaceBomb,
            }])
            .unwrap();
        report
            .events
            .extend(state.eliminate_departed(departed).unwrap());
        let frozen = state.clone();
        assert!(state.eliminate_departed(departed).unwrap().is_empty());
        assert_eq!(state, frozen);

        let capture = capture_expanded(&state, &[report]).unwrap();
        assert_valid(&capture);
        let owner = capture.world.actors[0].view.id;
        assert!(!capture.world.actors[0].view.alive);
        assert!(capture.world.bombs.iter().any(|bomb| bomb.owner == owner));
        assert_bound_delivery(count, &capture, crate::game::EnabledItems::EMPTY);
    }
}

#[test]
fn finalized_departure_physical_body_is_frozen() {
    let mut state = round_state(
        crate::game::PlayerCount::Three,
        crate::game::PlayerCount::Three.seats(),
        crate::game::EnabledItems::EMPTY,
    );
    let events = state
        .eliminate_departed([true, true, false, false])
        .unwrap();
    assert!(events.iter().any(|event| matches!(
        event.kind,
        crate::game::StepEventKind::RoundEnded(crate::game::RoundResult::Winner(
            crate::game::SeatId::Three
        ))
    )));
    let before = capture_expanded(&state, &[]).unwrap();
    assert!(
        state
            .eliminate_departed([false, false, true, false])
            .unwrap()
            .is_empty()
    );
    let after = capture_expanded(&state, &[]).unwrap();
    assert_eq!(after, before);
}

#[test]
fn physical_facade_rejects_private_world_corruption_families() {
    let capture = valid_capture(crate::game::PlayerCount::Four);
    let mut corruptions = Vec::new();

    let mut value = capture.clone();
    value.world.remaining_ticks -= 1;
    corruptions.push(value);

    let mut value = capture.clone();
    value.world.arena.blocks = vec![Cell::new(0, 0).unwrap()];
    corruptions.push(value);

    let mut value = capture.clone();
    value.world.actors[1].view.seat = Seat::One;
    corruptions.push(value);

    let mut value = capture.clone();
    value.world.actors[0].view.bomb_range = 9;
    corruptions.push(value);

    let mut value = capture.clone();
    value.world.private_pickup_cells = vec![Cell::new(2, 2).unwrap(), Cell::new(1, 1).unwrap()];
    corruptions.push(value);

    let mut value = capture.clone();
    value.reports[0].trace_count = 1;
    corruptions.push(value);

    let mut value = capture.clone();
    value.reports[0].tick = Tick::new(value.world.physical_tick.get() + 1).unwrap();
    corruptions.push(value);

    for (index, corrupted) in corruptions.iter().enumerate() {
        assert!(
            projection_validation::validate(&corrupted.world, &corrupted.reports, &corrupted.rules)
                .is_err(),
            "corruption family {index} was admitted"
        );
    }
}

#[test]
fn timed_bomb_history_is_a_complete_physical_proof() {
    let mut capture = valid_capture(crate::game::PlayerCount::Four);
    let tick = capture.world.physical_tick;
    let placed_at = Tick::new(tick.get() - 1).unwrap();
    let bomb = BombView {
        id: EntityId::new(100),
        cell: Cell::new(2, 1).unwrap(),
        owner: capture.world.actors[0].view.id,
        range: 2,
        placed_at,
        trigger: BombTrigger::timed(179, false).unwrap(),
        blast: BlastKind::Standard,
    };
    let history = projection_facts::BombHistory {
        id: bomb.id,
        trigger: projection_facts::TriggerHistory::Timed {
            captured_fuse_ticks: 180,
            paused_ticks: 0,
            detonates_at: Tick::new(placed_at.get() + 180).unwrap(),
            paused: false,
        },
        motion: None,
        has_been_kicked: false,
        has_been_punched: false,
    };
    capture.world.actors[0].view.live_bomb_count = 1;
    capture.world.bombs.push(bomb);
    capture.world.bomb_history.push(history);
    assert_valid(&capture);

    let mut bad_deadline = capture.clone();
    let projection_facts::TriggerHistory::Timed {
        ref mut detonates_at,
        ..
    } = bad_deadline.world.bomb_history[0].trigger
    else {
        unreachable!()
    };
    *detonates_at = Tick::new(detonates_at.get() + 1).unwrap();
    assert!(
        projection_validation::validate(
            &bad_deadline.world,
            &bad_deadline.reports,
            &bad_deadline.rules
        )
        .is_err()
    );

    let mut bad_private_id = capture.clone();
    bad_private_id.world.bomb_history[0].id = EntityId::new(101);
    assert!(
        projection_validation::validate(
            &bad_private_id.world,
            &bad_private_id.reports,
            &bad_private_id.rules
        )
        .is_err()
    );

    let mut blind_public_change = capture.clone();
    blind_public_change.world.bombs[0].trigger = BombTrigger::remote();
    assert!(
        projection_validation::validate(
            &blind_public_change.world,
            &blind_public_change.reports,
            &blind_public_change.rules
        )
        .is_err()
    );
}
