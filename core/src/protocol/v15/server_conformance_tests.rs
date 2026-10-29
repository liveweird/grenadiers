use serde::{Deserialize, Serialize};

use crate::game::{
    ActorInit, ArenaGenerationConfig, EffectKind, EnabledItems, PlayerCount, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig,
    World, generate_multiplayer_arena, generate_variable_multiplayer_arena,
};
use crate::protocol::{Counter, RoomCode, RoomId, Tick};
use crate::room::{
    BoardSize as DomainBoard, CpuDifficulty, CpuProfile, EnvironmentRoomConfiguration,
    RoomConfiguration as DomainConfiguration, SeatControl,
};

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

fn seat(value: SeatId) -> Seat {
    match value {
        SeatId::One => Seat::One,
        SeatId::Two => Seat::Two,
        SeatId::Three => Seat::Three,
        SeatId::Four => Seat::Four,
    }
}

fn environment(value: StormCloudConfig) -> Environment {
    Environment {
        storm_cloud: value == StormCloudConfig::Single,
    }
}

fn available_items() -> ItemSet {
    ItemSet::from_mask(ItemMask::new(EnabledItems::ALL.bits()).unwrap())
}

fn settings(enabled: EnabledItems, cloud: StormCloudConfig) -> RoomSettings {
    RoomSettings::new(
        GameSettings::new(
            0,
            256,
            3,
            180,
            ItemSet::from_mask(ItemMask::new(enabled.bits()).unwrap()),
        )
        .unwrap(),
        environment(cloud),
    )
}

fn controls(count: PlayerCount) -> Vec<SeatControl> {
    match count {
        PlayerCount::Two => vec![SeatControl::Human, SeatControl::Cpu],
        PlayerCount::Three => vec![SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        PlayerCount::Four => vec![
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Human,
            SeatControl::Cpu,
        ],
    }
}

fn difficulty(count: PlayerCount) -> CpuDifficulty {
    if count == PlayerCount::Three {
        assert_eq!(CpuProfile::ENVIRONMENT_EASY.policy.version(), 18);
        CpuProfile::ENVIRONMENT_EASY.difficulty
    } else {
        assert_eq!(CpuProfile::ENVIRONMENT_STANDARD.policy.version(), 18);
        CpuProfile::ENVIRONMENT_STANDARD.difficulty
    }
}

fn wire_difficulty(value: CpuDifficulty) -> Difficulty {
    match value {
        CpuDifficulty::Easy => Difficulty::Easy,
        CpuDifficulty::Standard => Difficulty::Standard,
    }
}

fn room(
    count: PlayerCount,
    board: DomainBoard,
    enabled: EnabledItems,
    cloud: StormCloudConfig,
    tick: Tick,
    phase: RoomPhase,
) -> RoomView {
    let controls = controls(count);
    let cpu_difficulty = difficulty(count);
    let cpu_ready = matches!(&phase, RoomPhase::Waiting | RoomPhase::Countdown { .. });
    let domain = DomainConfiguration::new(&controls, Some(cpu_difficulty), board).unwrap();
    RoomView {
        settings: settings(enabled, cloud),
        settings_revision: counter(1),
        settings_editor: Some(Seat::One),
        available_items: available_items(),
        configuration: RoomConfiguration::from_domain(EnvironmentRoomConfiguration::new(
            domain, cloud,
        )),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: counter(1),
        revision: counter(1),
        tick,
        rules_version: RulesVersion::ENVIRONMENT_V7,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: count
            .seats()
            .iter()
            .map(|&value| Score {
                seat: seat(value),
                wins: Wins::new(0).unwrap(),
            })
            .collect(),
        last_round: None,
        seats: count
            .seats()
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let cpu = controls[index] == SeatControl::Cpu;
                SeatStatus {
                    seat: seat(value),
                    occupied: true,
                    ready: cpu && cpu_ready,
                    rematch_ready: false,
                    controller: if cpu {
                        Controller::Cpu {
                            difficulty: wire_difficulty(cpu_difficulty),
                        }
                    } else {
                        Controller::Human
                    },
                    connection: if cpu {
                        ConnectionStatus::ServerControlled
                    } else {
                        ConnectionStatus::Connected
                    },
                }
            })
            .collect(),
        phase,
    }
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
    let mut state = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        crate::game::Tick(0),
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
    }
    let capture = capture_expanded(&state, &reports).unwrap();
    let tick = Tick::new(state.tick().0).unwrap();
    WorldCase {
        room: room(count, board, enabled, cloud, tick, RoomPhase::Running),
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
    let waiting = room(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        StormCloudConfig::Disabled,
        Tick::new(0).unwrap(),
        RoomPhase::Waiting,
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
    for phase in [
        RoomPhase::Waiting,
        RoomPhase::Countdown {
            kind: CountdownKind::Initial,
            ends_at: Tick::new(180).unwrap(),
        },
    ] {
        let mut candidate = room(
            PlayerCount::Two,
            DomainBoard::ElevenByNine,
            EnabledItems::EMPTY,
            StormCloudConfig::Disabled,
            Tick::new(0).unwrap(),
            phase,
        );
        assert!(super::projection::validate_room(&candidate).is_ok());
        candidate.seats[1].ready = false;
        assert_eq!(
            super::projection::validate_room(&candidate),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}
