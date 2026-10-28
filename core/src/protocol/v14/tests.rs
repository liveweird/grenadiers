use super::*;
use crate::room::{BoardSize as DomainBoardSize, CpuDifficulty, SeatControl};
use jsonschema::validator_for;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
fn checked_configuration_requires_explicit_null_and_domain_invariants() {
    let valid = r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","cpu","human","cpu"],"cpuDifficulty":"standard","board":{"width":15,"height":13}}}"#;
    let GameClientMessage::CreateRoom { configuration, .. } = parse_client_message(valid).unwrap()
    else {
        panic!("wrong message")
    };
    assert_eq!(
        configuration.domain().controllers(),
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Human,
            SeatControl::Cpu
        ]
    );
    assert_eq!(
        configuration.domain().cpu_difficulty(),
        Some(CpuDifficulty::Standard)
    );
    assert_eq!(
        configuration.domain().board(),
        DomainBoardSize::FifteenByThirteen
    );

    for invalid in [
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["cpu","human"],"cpuDifficulty":"easy","board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","cpu"],"cpuDifficulty":null,"board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":"easy","board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":12,"height":10}}}"#,
    ] {
        assert_eq!(
            parse_client_message(invalid),
            Err(ParseClientMessageError::InvalidMessage)
        );
    }
}

#[test]
fn all_supported_lineups_and_boards_round_trip() {
    let mut lineups = Vec::new();
    for count in 2..=4 {
        for suffix in 0..(1 << (count - 1)) {
            let mut controllers = vec!["human"];
            for index in 0..count - 1 {
                controllers.push(if suffix & (1 << index) == 0 {
                    "human"
                } else {
                    "cpu"
                });
            }
            if suffix == 0 {
                lineups.push((controllers, None));
            } else {
                lineups.push((controllers.clone(), Some("easy")));
                lineups.push((controllers, Some("standard")));
            }
        }
    }
    for (controllers, difficulty) in lineups {
        for (width, height) in [(11, 9), (13, 11), (15, 13)] {
            let value = serde_json::json!({"type":"createRoom", "sequence":1, "configuration":{"controllers":controllers, "cpuDifficulty":difficulty, "board":{"width":width,"height":height}}});
            assert!(parse_client_message(&value.to_string()).is_ok(), "{value}");
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Conformance {
    valid_create_rooms: Vec<Case>,
    invalid_create_rooms: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    case_id: String,
    value: Value,
}

#[test]
fn parser_and_schema_agree_on_v14_configuration_fixture() {
    let corpus: Conformance = serde_json::from_str(include_str!(
        "../../../../api/fixtures/gameplay-v14-client-conformance.json"
    ))
    .unwrap();
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v14.schema.json")).unwrap();
    schema["$ref"] = serde_json::json!("#/$defs/GameClientMessage");
    let validator = validator_for(&schema).unwrap();
    for case in corpus.valid_create_rooms {
        assert!(
            parse_client_message(&case.value.to_string()).is_ok(),
            "{}",
            case.case_id
        );
        assert!(validator.is_valid(&case.value), "{}", case.case_id);
    }
    for case in corpus.invalid_create_rooms {
        assert!(
            parse_client_message(&case.value.to_string()).is_err(),
            "{}",
            case.case_id
        );
        assert!(!validator.is_valid(&case.value), "{}", case.case_id);
    }
}

pub(super) fn multiplayer_state(count: crate::game::PlayerCount) -> crate::game::State {
    multiplayer_state_with_items(count, crate::game::EnabledItems::EMPTY)
}

pub(super) fn multiplayer_state_with_items(
    count: crate::game::PlayerCount,
    enabled: crate::game::EnabledItems,
) -> crate::game::State {
    use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
    use crate::game::{
        ActorInit, Arena, CellId, Direction, EntityId, Position, RulesConfig, SeatId, SpeedLevel,
        Terrain, World,
    };
    let corners = [
        CellId::new(1, 1),
        CellId::new(13, 1),
        CellId::new(1, 11),
        CellId::new(13, 11),
    ];
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    let spawns = if count == crate::game::PlayerCount::Two {
        vec![corners[0], corners[3]]
    } else {
        corners[..count.get()].to_vec()
    };
    let arena = Arena::new(terrain, BTreeSet::new(), spawns).unwrap();
    let actors = count
        .seats()
        .iter()
        .map(|&seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(corners[seat.index()]),
            heading: if matches!(seat, SeatId::One | SeatId::Three) {
                Direction::East
            } else {
                Direction::West
            },
            speed: SpeedLevel::Base,
        })
        .collect::<Vec<_>>();
    let rules =
        RulesConfig::multiplayer_v6(crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled));
    if enabled.contains_teleporter() {
        crate::game::State::new_at_tick_with_assignment_and_seed(
            rules,
            World::new(arena),
            actors,
            crate::game::Tick(0),
            crate::game::RoundAssignment::first(),
            crate::game::SimulationSeed::new([14; 32]),
        )
        .unwrap()
    } else {
        crate::game::State::new(rules, World::new(arena), actors).unwrap()
    }
}

pub(super) fn running_room(
    count: crate::game::PlayerCount,
    tick: crate::protocol::Tick,
) -> RoomView {
    use crate::protocol::{Counter, RoomCode, RoomId};
    let configuration = crate::room::RoomConfiguration::new(
        &vec![crate::room::SeatControl::Human; count.get()],
        None,
        crate::room::BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let seats = [Seat::One, Seat::Two, Seat::Three, Seat::Four];
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, ItemSet::empty()).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: Some(Seat::One),
        available_items: ItemSet::new([
            ItemId::Kick,
            ItemId::Punch,
            ItemId::Bracelet,
            ItemId::WallPass,
            ItemId::Heart,
            ItemId::RemoteBomb,
            ItemId::PiercingBomb,
            ItemId::PowerBomb,
            ItemId::Landmine,
            ItemId::Shield,
            ItemId::Stopwatch,
            ItemId::InvisibilityCloak,
            ItemId::InvisibleBombs,
            ItemId::GoldenFlame,
            ItemId::ShortFuse,
            ItemId::Diarrhea,
            ItemId::SlowFoot,
            ItemId::ShortFlame,
            ItemId::LongFuse,
            ItemId::ReverseControls,
            ItemId::OilBarrel,
            ItemId::Teleporter,
        ])
        .unwrap(),
        configuration: RoomConfiguration::from_domain(configuration),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick,
        rules_version: RulesVersion::MULTIPLAYER_V6,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: seats
            .into_iter()
            .take(count.get())
            .map(|seat| Score {
                seat,
                wins: Wins::new(0).unwrap(),
            })
            .collect(),
        last_round: None,
        seats: seats
            .into_iter()
            .take(count.get())
            .map(|seat| SeatStatus {
                seat,
                occupied: true,
                ready: false,
                rematch_ready: false,
                controller: Controller::Human,
                connection: ConnectionStatus::Connected,
            })
            .collect(),
        phase: RoomPhase::Running,
    }
}

#[test]
fn cloak_privacy_and_fourth_human_admission_use_checked_publication() {
    use crate::game::{
        ArenaGenerationConfig, EffectKind as DomainEffect, EnabledItems, RoundAssignment,
    };
    let enabled = EnabledItems::from_bits(
        EnabledItems::INVISIBILITY_CLOAK.bits() | EnabledItems::INVISIBLE_BOMBS.bits(),
    )
    .unwrap();
    let mut state = multiplayer_state_with_items(crate::game::PlayerCount::Four, enabled);
    state.apply_test_effect(
        crate::game::SeatId::Two,
        DomainEffect::InvisibilityCloak,
        crate::game::Tick(0),
    );
    state.apply_test_effect(
        crate::game::SeatId::One,
        DomainEffect::InvisibleBombs,
        crate::game::Tick(0),
    );
    state
        .insert_test_bomb(
            crate::game::EntityId(100),
            crate::game::CellId::new(7, 7),
            crate::game::EntityId(10),
            2,
            crate::game::Tick(0),
            crate::game::BombTrigger::Timed {
                detonates_at: crate::game::Tick(180),
            },
        )
        .unwrap();
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let mut room = running_room(crate::game::PlayerCount::Four, capture.world.physical_tick);
    room.settings = GameSettings::new(
        0,
        256,
        3,
        180,
        ItemSet::new([ItemId::InvisibilityCloak, ItemId::InvisibleBombs]).unwrap(),
    )
    .unwrap();
    let mut noncanonical = room.clone();
    noncanonical.seats.swap(0, 1);
    assert_eq!(
        validate_room_view(&noncanonical),
        Err(ProjectionError::InvalidExpandedBinding)
    );
    let generation = crate::game::generate_multiplayer_arena(
        [7; 32],
        enabled,
        ArenaGenerationConfig {
            player_count: crate::game::PlayerCount::Four,
            board: crate::room::BoardSize::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap()
    .record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    let snapshots = prepare_snapshots(&publication).unwrap();
    assert_eq!(
        snapshots.iter().map(|(seat, _)| *seat).collect::<Vec<_>>(),
        vec![Seat::One, Seat::Two, Seat::Three, Seat::Four]
    );
    for recipient in [Seat::One, Seat::Three, Seat::Four] {
        assert!(matches!(
            project_recipient(&publication, recipient)
                .unwrap()
                .state
                .actors[1],
            ActorView::Hidden {
                seat: Seat::Two,
                ..
            }
        ));
    }
    assert!(matches!(
        project_recipient(&publication, Seat::Two)
            .unwrap()
            .state
            .actors[1],
        ActorView::Visible {
            seat: Seat::Two,
            ..
        }
    ));
    assert!(
        project_recipient(&publication, Seat::One)
            .unwrap()
            .state
            .bombs
            .is_empty()
    );
    assert_eq!(
        project_recipient(&publication, Seat::Two)
            .unwrap()
            .state
            .bombs
            .len(),
        1
    );
    let admission = prepare_admission(
        AdmissionProjection {
            room: &room,
            state_epoch: Some(room.round_id),
            capture: Some(&capture),
            generation: Some(generation),
        },
        PersonalAdmission {
            recipient: Seat::Four,
            last_action_id: None,
            sequence: crate::protocol::Counter::new(1).unwrap(),
            resume_token: ResumeToken::new("a".repeat(64)).unwrap(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&admission).unwrap()["youSeat"],
        "four"
    );
    let mut corrupt = capture.clone();
    corrupt.world.actors[0].view.seat = Seat::Two;
    assert_eq!(
        bind_expanded(&room, room.round_id, &corrupt, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding),
    );
}
