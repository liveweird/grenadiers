use crate::cpu::WorldId;
use crate::game::{self, EnabledItems, PlayerCount, SeatId};
use crate::room::{BoardSize, SeatControl};

use super::*;

fn state(
    board: BoardSize,
    count: PlayerCount,
    enabled: EnabledItems,
) -> (game::State, game::GenerationRecord) {
    let config = game::ArenaGenerationConfig {
        board,
        player_count: count,
        assignment: game::RoundAssignment::first(),
    };
    let generated = if board == BoardSize::FifteenByThirteen {
        game::generate_multiplayer_arena([31; 32], enabled, config)
    } else {
        game::generate_variable_multiplayer_arena([31; 32], enabled, config)
    }
    .unwrap();
    let actors = count
        .seats()
        .iter()
        .map(|&seat| game::ActorInit {
            id: game::EntityId(10 + seat.index() as u32),
            seat,
            position: game::Position::center(
                config
                    .assignment
                    .spawn_for(generated.arena(), seat)
                    .unwrap(),
            ),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        })
        .collect::<Vec<_>>();
    let (world, record) = game::World::from_generated(generated).unwrap();
    let state = game::State::new_at_tick_with_assignment_and_seed(
        game::RulesConfig::multiplayer_v6(
            crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
        ),
        world,
        actors,
        game::Tick(0),
        config.assignment,
        game::SimulationSeed::new([43; 32]),
    )
    .unwrap();
    (state, record)
}

fn room(
    board: BoardSize,
    count: PlayerCount,
    tick: crate::protocol::Tick,
    enabled: EnabledItems,
) -> RoomView {
    let mut room = tests::running_room(count, tick);
    room.configuration = RoomConfiguration::from_domain(
        crate::room::RoomConfiguration::new(&vec![SeatControl::Human; count.get()], None, board)
            .unwrap(),
    );
    assert!(matches!(enabled, EnabledItems::EMPTY | EnabledItems::ALL));
    room.settings = GameSettings::new(
        0,
        256,
        3,
        180,
        if enabled == EnabledItems::ALL {
            room.available_items
        } else {
            ItemSet::empty()
        },
    )
    .unwrap();
    room
}

#[test]
fn every_board_and_count_captures_binds_serializes_and_observes() {
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v14.schema.json")).unwrap();
    schema["$ref"] = serde_json::json!("#/$defs/GameServerMessage");
    let validator = jsonschema::validator_for(&schema).unwrap();
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            let (mut state, generation) = state(board, count, EnabledItems::EMPTY);
            let report = state.step(&[]).unwrap();
            let capture = capture_expanded(&state, &[report]).unwrap();
            let room = room(
                board,
                count,
                capture.world.physical_tick,
                EnabledItems::EMPTY,
            );
            let bound = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
            let frames = prepare_snapshots(&bound).unwrap();
            assert_eq!(frames.len(), count.get());
            for (recipient, frame) in frames {
                let projected = project_recipient(&bound, recipient).unwrap();
                let observation = observe_cpu_recipient(
                    &bound,
                    recipient,
                    WorldId {
                        room_instance: 1,
                        physical_round: 1,
                    },
                )
                .unwrap();
                assert_eq!(projected.state.arena.terrain.len(), board.cell_count());
                assert_eq!(observation.parts().arena.width, board.width());
                assert_eq!(observation.parts().arena.height, board.height());
                assert_eq!(observation.parts().actors.len(), count.get());
                assert!(frame.len() <= MAX_SERVER_MESSAGE_BYTES);
                let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
                assert_eq!(value["state"]["arena"]["width"], board.width());
                assert!(validator.is_valid(&value), "{board:?}/{count:?}");
            }
        }
    }
}

#[test]
fn historical_rules_and_wrong_generation_cannot_bind_small_boards() {
    for board in [BoardSize::ElevenByNine, BoardSize::ThirteenByEleven] {
        let (mut state, generation) = state(board, PlayerCount::Two, EnabledItems::EMPTY);
        let report = state.step(&[]).unwrap();
        let capture = capture_expanded(&state, &[report]).unwrap();
        let mut room = room(
            board,
            PlayerCount::Two,
            capture.world.physical_tick,
            EnabledItems::EMPTY,
        );
        for version in 1..=5 {
            room.rules_version = RulesVersion::new(version).unwrap();
            assert_eq!(
                validate_room_view(&room),
                Err(ProjectionError::InvalidRules)
            );
        }
        room.rules_version = RulesVersion::MULTIPLAYER_V6;
        let old = projection_validation_tests::generation(PlayerCount::Two, EnabledItems::EMPTY);
        assert_eq!(
            bind_expanded(&room, room.round_id, &capture, Some(old)),
            Err(ProjectionError::InvalidExpandedBinding)
        );
        assert!(bind_expanded(&room, room.round_id, &capture, Some(generation)).is_ok());
    }
}

#[test]
fn complete_private_cells_and_actor_footprints_are_checked_before_projection() {
    let board = BoardSize::ElevenByNine;
    let (mut state, generation) = state(board, PlayerCount::Four, EnabledItems::ALL);
    state.apply_test_effect(
        SeatId::Two,
        game::EffectKind::InvisibilityCloak,
        game::Tick(0),
    );
    state.apply_test_effect(SeatId::One, game::EffectKind::InvisibleBombs, game::Tick(0));
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let room = room(
        board,
        PlayerCount::Four,
        capture.world.physical_tick,
        EnabledItems::ALL,
    );
    let bound = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    assert!(matches!(
        project_recipient(&bound, Seat::One).unwrap().state.actors[1],
        ActorView::Hidden { .. }
    ));
    let outside = crate::protocol::Cell::new(12, 1).unwrap();
    assert_eq!(
        super::projection_geometry::terrain_at(&capture.world, outside),
        None
    );
    let outside_row = crate::protocol::Cell::new(1, 10).unwrap();
    assert_eq!(
        super::projection_geometry::terrain_at(&capture.world, outside_row),
        None
    );
    let mut bad_actor = capture.clone();
    bad_actor.world.actors[1].view.position =
        crate::protocol::Position::new(11 * 1024 - 319, 1536).unwrap();
    assert_eq!(
        projection_validation::validate(&bad_actor.world, &bad_actor.reports, &bad_actor.rules),
        Err(ProjectionError::InvalidActors)
    );
    assert_eq!(
        bind_expanded(&room, room.round_id, &bad_actor, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
    let mut bad_event = capture.clone();
    bad_event.reports[0]
        .events
        .push(projection_facts::EventFacts {
            tick: capture.world.physical_tick,
            audience: game::EventAudience::all(PlayerCount::Four),
            kind: EventKind::PickupCollected {
                id: capture.world.actors[0].view.id,
                seat: Seat::One,
                cell: outside,
                kind: PickupKind::BombCapacity,
            },
            bomb_history: None,
        });
    assert_eq!(
        bind_expanded(&room, room.round_id, &bad_event, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
    let mut bad_block = capture.clone();
    bad_block.world.arena.blocks.push(outside);
    bad_block
        .world
        .arena
        .blocks
        .sort_by_key(|cell| (cell.row(), cell.column()));
    assert_eq!(
        projection_validation::validate(&bad_block.world, &bad_block.reports, &bad_block.rules),
        Err(ProjectionError::InvalidArena)
    );
    let mut bad_oil = capture.clone();
    bad_oil.world.oil.clear();
    bad_oil.world.oil.push(projection_facts::OilFacts {
        cell: outside,
        phase: projection_facts::OilPhase::Unlit,
    });
    assert_eq!(
        projection_validation::validate(&bad_oil.world, &bad_oil.reports, &bad_oil.rules),
        Err(ProjectionError::InvalidItems)
    );
}

#[test]
fn small_board_cpu_and_human_receive_the_same_filtered_world() {
    for board in [BoardSize::ElevenByNine, BoardSize::ThirteenByEleven] {
        let (mut state, generation) = state(board, PlayerCount::Four, EnabledItems::ALL);
        state.apply_test_effect(
            SeatId::Two,
            game::EffectKind::InvisibilityCloak,
            game::Tick(0),
        );
        state.apply_test_effect(SeatId::One, game::EffectKind::InvisibleBombs, game::Tick(0));
        state.apply_test_effect(SeatId::Three, game::EffectKind::Stopwatch, game::Tick(0));
        let collector = state.actors()[3].position().cell().unwrap();
        state.insert_test_pickup(collector, game::PickupKind::Teleporter);
        let report = state.step(&[]).unwrap();
        let capture = capture_expanded(&state, &[report]).unwrap();
        let room = room(
            board,
            PlayerCount::Four,
            capture.world.physical_tick,
            EnabledItems::ALL,
        );
        let bound = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
        for recipient in [Seat::One, Seat::Two, Seat::Three, Seat::Four] {
            let human = project_recipient(&bound, recipient).unwrap().state;
            let cpu = observe_cpu_recipient(
                &bound,
                recipient,
                WorldId {
                    room_instance: 1,
                    physical_round: 1,
                },
            )
            .unwrap();
            for (wire, observed) in human.actors.iter().zip(&cpu.parts().actors) {
                match (wire, observed) {
                    (
                        ActorView::Hidden { id, .. },
                        crate::cpu::ObservedActor::Hidden {
                            id: observed_id, ..
                        },
                    ) => {
                        assert_eq!(id.get(), observed_id.0);
                    }
                    (
                        ActorView::Visible {
                            position,
                            voluntary_actions_frozen,
                            last_teleport_tick,
                            ..
                        },
                        crate::cpu::ObservedActor::Visible(actor),
                    ) => {
                        assert_eq!(actor.position.x, i32::from(position.x()));
                        assert_eq!(actor.position.y, i32::from(position.y()));
                        assert_eq!(actor.voluntary_actions_frozen, *voluntary_actions_frozen);
                        assert_eq!(
                            actor.last_teleport_tick.map(|tick| tick.0),
                            last_teleport_tick.map(|tick| tick.get())
                        );
                    }
                    _ => panic!("CPU privacy differs from human projection"),
                }
            }
            assert_eq!(
                matches!(cpu.parts().bombs, crate::cpu::BombKnowledge::Unavailable),
                recipient == Seat::One
            );
        }
    }
}
