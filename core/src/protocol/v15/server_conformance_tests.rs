use serde::{Deserialize, Serialize};

use crate::game::{
    ActorInit, ArenaGenerationConfig, EffectKind, EnabledItems, PlayerCount, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig,
    World, generate_multiplayer_arena, generate_variable_multiplayer_arena,
};
use crate::protocol::{Counter, Tick};
use crate::room::BoardSize as DomainBoard;

use super::*;

const CHECKED_CORPUS: &str =
    include_str!("../../../../api/fixtures/gameplay-v15-server-conformance.json");

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct FrameCase {
    case_id: String,
    scenario: String,
    recipient: Option<Seat>,
    frame: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct Corpus {
    cases: Vec<FrameCase>,
}

struct WorldCase {
    room: RoomView,
    capture: ExpandedCapture,
    generation: crate::game::GenerationRecord,
}

fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn generated(
    board: DomainBoard,
    count: PlayerCount,
    enabled: EnabledItems,
    seed: [u8; 32],
) -> crate::game::GeneratedArena {
    let config = ArenaGenerationConfig {
        player_count: count,
        board,
        assignment: RoundAssignment::first(),
    };
    if board == DomainBoard::FifteenByThirteen {
        generate_multiplayer_arena(seed, enabled, config).unwrap()
    } else {
        generate_variable_multiplayer_arena(seed, enabled, config).unwrap()
    }
}

fn world_case(
    board: DomainBoard,
    count: PlayerCount,
    cloud: StormCloudConfig,
    cloak_center_actor: bool,
    seed: [u8; 32],
) -> WorldCase {
    let enabled = if cloak_center_actor {
        EnabledItems::INVISIBILITY_CLOAK
    } else {
        EnabledItems::EMPTY
    };
    let (world, generation) =
        World::from_generated(generated(board, count, enabled, seed)).unwrap();
    let spawns = world.arena().spawns();
    let actors = count
        .seats()
        .iter()
        .map(|&value| ActorInit {
            id: crate::game::EntityId(10 + value.index() as u32),
            seat: value,
            position: Position::center(spawns[value.index()]),
            heading: crate::game::Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect::<Vec<_>>();
    let rules = RulesConfig::environment_v7(
        crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
        cloud,
    );
    let mut lifecycle = super::server_conformance_lifecycle::lifecycle(count, board, enabled, true);
    let mut state = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        lifecycle.tick(),
        RoundAssignment::first(),
        SimulationSeed::new(seed),
    )
    .unwrap();
    let mut reports = Vec::new();
    if cloud == StormCloudConfig::Single {
        let center = state.storm_cloud().unwrap().position;
        let centered_seat = if cloak_center_actor {
            SeatId::Two
        } else {
            SeatId::One
        };
        state.set_test_actor_pose(centered_seat, center, crate::game::Direction::East);
        if cloak_center_actor {
            state.apply_test_effect(
                SeatId::Two,
                EffectKind::InvisibilityCloak,
                crate::game::Tick(0),
            );
        }
        reports.push(state.step(&[]).unwrap());
        lifecycle.complete_running_tick(None).unwrap();
    }
    let capture = capture_expanded(&state, &reports).unwrap();
    WorldCase {
        room: super::server_conformance_lifecycle::room_view(&lifecycle, cloud, true),
        capture,
        generation,
    }
}

fn snapshot_cases(case: &WorldCase, scenario: &str, recipients: &[Seat]) -> Vec<FrameCase> {
    let publication = bind_expanded(
        &case.room,
        case.room.round_id,
        &case.capture,
        case.generation,
    )
    .unwrap();
    let snapshots = prepare_snapshots(&publication).unwrap();
    recipients
        .iter()
        .map(|&recipient| FrameCase {
            case_id: format!("{scenario}-{}", format!("{recipient:?}").to_lowercase()),
            scenario: scenario.into(),
            recipient: Some(recipient),
            frame: snapshots
                .iter()
                .find(|(seat, _)| *seat == recipient)
                .unwrap()
                .1
                .clone(),
        })
        .collect()
}

fn actual_corpus() -> Corpus {
    let disabled = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        false,
        [11; 32],
    );
    let single = world_case(
        DomainBoard::ThirteenByEleven,
        PlayerCount::Three,
        StormCloudConfig::Single,
        false,
        [13; 32],
    );
    let privacy = world_case(
        DomainBoard::FifteenByThirteen,
        PlayerCount::Four,
        StormCloudConfig::Single,
        true,
        [15; 32],
    );
    let waiting_lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    let waiting = super::server_conformance_lifecycle::room_view(
        &waiting_lifecycle,
        StormCloudConfig::Disabled,
        false,
    );
    let admission = prepare_admission(
        AdmissionProjection {
            room: &waiting,
            state_epoch: None,
            capture: None,
            generation: None,
        },
        PersonalAdmission {
            recipient: Seat::One,
            last_action_id: None,
            sequence: counter(1),
            resume_token: ResumeToken::new("c".repeat(64)).unwrap(),
        },
    )
    .unwrap();
    let mut cases = vec![FrameCase {
        case_id: "disabled-11x9-cpu18-admission".into(),
        scenario: "waitingAdmission".into(),
        recipient: Some(Seat::One),
        frame: admission,
    }];
    cases.extend(snapshot_cases(&disabled, "disabled11x9Cpu18", &[Seat::One]));
    cases.extend(snapshot_cases(
        &single,
        "single13x11Cpu18Timer",
        &[Seat::One],
    ));
    cases.push(FrameCase {
        case_id: "single-15x13-cpu18-room-state".into(),
        scenario: "roomState".into(),
        recipient: None,
        frame: encode_server_message(&GameServerMessage::RoomState {
            room: privacy.room.clone(),
        })
        .unwrap(),
    });
    cases.extend(snapshot_cases(
        &privacy,
        "single15x13Cpu18Cloak",
        &[Seat::One, Seat::Two],
    ));
    cases.extend(
        super::server_conformance_lifecycle_cases::room_cases()
            .into_iter()
            .map(|(case_id, scenario, room)| FrameCase {
                case_id: case_id.into(),
                scenario: scenario.into(),
                recipient: None,
                frame: encode_server_message(&GameServerMessage::RoomState { room }).unwrap(),
            }),
    );
    cases.extend(
        super::server_conformance_lifecycle_cases::admission_cases()
            .into_iter()
            .map(|(case_id, scenario, recipient, frame)| FrameCase {
                case_id: case_id.into(),
                scenario: scenario.into(),
                recipient: Some(recipient),
                frame,
            }),
    );
    Corpus { cases }
}

#[test]
fn checked_corpus_matches_fresh_checked_v15_projection() {
    let actual = actual_corpus();
    let expected: Corpus = serde_json::from_str(CHECKED_CORPUS).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn checked_frames_satisfy_the_v15_server_schema() {
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v15.schema.json")).unwrap();
    schema["$ref"] = serde_json::json!("#/$defs/GameServerMessage");
    let validator = jsonschema::validator_for(&schema).unwrap();
    for case in actual_corpus().cases {
        let frame: serde_json::Value = serde_json::from_str(&case.frame).unwrap();
        assert!(validator.is_valid(&frame), "{}", case.case_id);
    }
}

#[test]
fn checked_room_validation_rejects_false_cpu_readiness_before_start() {
    let mut lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    let waiting = super::server_conformance_lifecycle::room_view(
        &lifecycle,
        StormCloudConfig::Disabled,
        false,
    );
    lifecycle.set_ready(SeatId::One, true).unwrap();
    let countdown = super::server_conformance_lifecycle::room_view(
        &lifecycle,
        StormCloudConfig::Disabled,
        false,
    );
    for mut candidate in [waiting, countdown] {
        assert!(super::projection::validate_room(&candidate).is_ok());
        candidate.seats[1].ready = false;
        assert_eq!(
            super::projection::validate_room(&candidate),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

fn running_room() -> RoomView {
    let lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    super::server_conformance_lifecycle::room_view(&lifecycle, StormCloudConfig::Disabled, true)
}

fn waiting_room() -> RoomView {
    let lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    super::server_conformance_lifecycle::room_view(&lifecycle, StormCloudConfig::Disabled, false)
}

#[test]
fn checked_room_validation_rejects_reviewed_metadata_and_history_mutations() {
    let mutations: &[fn(&mut RoomView)] = &[
        |room| room.available_items = ItemSet::empty(),
        |room| room.seats[0].occupied = false,
        |room| room.seats[0].ready = true,
        |room| room.seats[1].ready = true,
        |room| room.seats[0].rematch_ready = true,
        |room| room.scores[0].wins = Wins::new(4).unwrap(),
        |room| {
            room.seats[0].connection = ConnectionStatus::Disconnected {
                expires_at: Tick::new(room.tick.get() + 601).unwrap(),
            };
        },
        |room| {
            room.phase = RoomPhase::MatchResult {
                ended_at: room.tick,
                result: MatchResult::Winner {
                    seat: Seat::Four,
                    reason: MatchWinnerReason::Forfeit,
                },
                rematch_at: Tick::new(room.tick.get() + 180).unwrap(),
            };
        },
    ];
    for mutate in mutations {
        let mut room = running_room();
        mutate(&mut room);
        assert_eq!(
            super::projection::validate_room(&room),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }

    let mut waiting = waiting_room();
    waiting.round_number = RoundCount::new(1).unwrap();
    assert!(super::projection::validate_room(&waiting).is_err());
    let mut waiting = waiting_room();
    waiting.scores[0].wins = Wins::new(1).unwrap();
    assert!(super::projection::validate_room(&waiting).is_err());
    let mut waiting = waiting_room();
    waiting.last_round = Some(CompletedRound {
        round_id: counter(1),
        round_number: counter(1),
        ended_at: waiting.tick,
        result: RoundResult::Winner { seat: Seat::One },
    });
    assert!(super::projection::validate_room(&waiting).is_err());
}

#[test]
fn checked_room_validation_rejects_terminal_deadline_mutation() {
    let mut lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    for round in 0..3 {
        lifecycle
            .complete_current_tick(crate::game::RoundResult::Winner(SeatId::One))
            .unwrap();
        if round < 2 {
            for _ in 0..crate::room::COUNTDOWN_TICKS * 2 {
                lifecycle.advance_idle_tick().unwrap();
            }
        }
    }
    let mut terminal = super::server_conformance_lifecycle::room_view(
        &lifecycle,
        StormCloudConfig::Disabled,
        true,
    );
    assert!(super::projection::validate_room(&terminal).is_ok());
    let RoomPhase::MatchResult { rematch_at, .. } = &mut terminal.phase else {
        panic!("expected terminal room");
    };
    *rematch_at = Tick::new(rematch_at.get() + 1).unwrap();
    assert_eq!(
        super::projection::validate_room(&terminal),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}
