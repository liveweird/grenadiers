use std::collections::BTreeSet;

use crate::game::{
    ActorInit, ArenaGenerationConfig, CellId, EnabledItems, PlayerCount, Position, RoundAssignment,
    RulesConfig, SeatId, SpeedLevel, SpikePitPhase as DomainPhase, StormCloudConfig, World,
    generate_multiplayer_arena,
};
use crate::protocol::{Counter, RoomCode, RoomId, Tick};
use crate::room::{
    BoardSize as DomainBoard, CpuDifficulty, LairRoomConfiguration, LairSettings, LairTuning,
    RespawnLairConfig, RoomConfiguration as DomainConfiguration, SeatControl, SpikePitConfig,
    SpikePitSettings, SpikePitTuning, SpikeRoomConfiguration,
};
use serde::Serialize;

use super::*;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../api/fixtures/gameplay-v19-server-conformance.json"
);

fn environment(enabled: bool) -> Environment {
    Environment {
        storm_cloud: false,
        lairs: Lairs {
            enabled: false,
            spawn_seconds: 10,
            max_living_mobs: 1,
        },
        spike_pits: SpikePits {
            enabled,
            warning_ticks: 6,
            raised_ticks: 60,
        },
    }
}

pub(crate) fn room(tick: u64, phase: RoomPhase, spike_enabled: bool) -> RoomView {
    let base = DomainConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Hard),
        DomainBoard::FifteenByThirteen,
    )
    .unwrap();
    let lairs = LairTuning::new(false, LairSettings::new(10, 1).unwrap());
    let pits = SpikePitTuning::new(spike_enabled, SpikePitSettings::new(6, 60).unwrap());
    let running = matches!(phase, RoomPhase::Running);
    RoomView {
        settings: RoomSettings::new(
            GameSettings::new(
                0,
                256,
                3,
                180,
                ItemSet::from_mask(ItemMask::new(0).unwrap()),
            )
            .unwrap(),
            environment(spike_enabled),
        ),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: (!running).then_some(Seat::One),
        available_items: ItemSet::from_mask(ItemMask::new(EnabledItems::ALL.bits()).unwrap()),
        configuration: RoomConfiguration::from_domain(SpikeRoomConfiguration::new(
            LairRoomConfiguration::new(base, StormCloudConfig::Disabled, lairs),
            pits,
        ))
        .unwrap(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(tick).unwrap(),
        rules_version: RulesVersion::ARENA_V10,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(u32::from(running)).unwrap(),
        scores: [Seat::One, Seat::Two]
            .map(|seat| Score {
                seat,
                wins: Wins::new(0).unwrap(),
            })
            .to_vec(),
        last_round: None,
        seats: [Seat::One, Seat::Two]
            .map(|seat| SeatStatus {
                seat,
                occupied: true,
                ready: seat == Seat::Two && !running,
                rematch_ready: false,
                controller: if seat == Seat::One {
                    Controller::Human
                } else {
                    Controller::Cpu {
                        difficulty: Difficulty::Hard,
                    }
                },
                connection: if seat == Seat::One {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::ServerControlled
                },
            })
            .to_vec(),
        phase,
    }
}

fn no_pit_world() -> World {
    let board = DomainBoard::FifteenByThirteen;
    let spawns = [CellId::new(1, 1), CellId::new(13, 11)];
    let potential = [
        CellId::new(1, 1),
        CellId::new(13, 1),
        CellId::new(1, 11),
        CellId::new(13, 11),
    ];
    let mut terrain = vec![crate::game::Terrain::Floor; board.cell_count()];
    for row in 0..board.height() {
        for column in 0..board.width() {
            if row == 0 || row + 1 == board.height() || column == 0 || column + 1 == board.width() {
                terrain[usize::from(row * board.width() + column)] = crate::game::Terrain::Wall;
            }
        }
    }
    let blocks = (1..board.height() - 1)
        .flat_map(|row| (1..board.width() - 1).map(move |column| CellId::new(column, row)))
        .filter(|cell| {
            potential
                .iter()
                .all(|spawn| spawn.column.abs_diff(cell.column) + spawn.row.abs_diff(cell.row) > 1)
        })
        .collect::<BTreeSet<_>>();
    World::new(crate::game::Arena::new_for_board(board, terrain, blocks, spawns).unwrap())
}

fn runtime(
    spike_enabled: bool,
    empty: bool,
    phase: DomainPhase,
) -> (crate::game::State, crate::game::GenerationRecord) {
    runtime_with_activation(
        spike_enabled,
        empty,
        phase,
        crate::game::PunchActivationPolicy::ManualV1,
    )
}

pub(crate) fn runtime_with_activation(
    spike_enabled: bool,
    empty: bool,
    phase: DomainPhase,
    punch_activation: crate::game::PunchActivationPolicy,
) -> (crate::game::State, crate::game::GenerationRecord) {
    let generated = generate_multiplayer_arena(
        [18; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: DomainBoard::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap();
    let generation = generated.record();
    let (world, _) = if empty {
        (no_pit_world(), generation)
    } else {
        World::from_generated(generated).unwrap()
    };
    let spawns = world.arena().spawns().to_vec();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: crate::game::EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: crate::game::Direction::East,
        speed: SpeedLevel::Base,
    });
    let config = if spike_enabled {
        SpikePitConfig::Enabled(SpikePitSettings::new(6, 60).unwrap())
    } else {
        SpikePitConfig::Disabled
    };
    let rules = RulesConfig::environment_v10(
        crate::room::RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        config,
    )
    .with_punch_activation_policy(punch_activation);
    let mut state = crate::game::State::new(rules, world, actors).unwrap();
    if matches!(phase, DomainPhase::Warning | DomainPhase::Raised) {
        let pit = state.spike_pits().unwrap()[0].cell;
        state.set_test_actor_pose(
            SeatId::One,
            Position::center(pit),
            crate::game::Direction::East,
        );
        state.step(&[]).unwrap();
        if phase == DomainPhase::Raised {
            state.set_test_actor_pose(
                SeatId::One,
                Position::center(spawns[0]),
                crate::game::Direction::East,
            );
        }
    }
    while phase == DomainPhase::Raised
        && state.spike_pits().unwrap()[0].phase != DomainPhase::Raised
    {
        state.step(&[]).unwrap();
    }
    (state, generation)
}

fn snapshot_case(case_id: &str, enabled: bool, empty: bool, phase: DomainPhase) -> FixtureCase {
    let (state, generation) = runtime(enabled, empty, phase);
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = room(state.tick().0, RoomPhase::Running, enabled);
    validate_room_view(&room).unwrap();
    let publication = bind_expanded(&room, Counter::new(1).unwrap(), &capture, generation)
        .unwrap_or_else(|error| panic!("{case_id}: {error:?}"));
    let projected = project_recipient(&publication, Seat::One).unwrap();
    let frame = encode_server_message(&GameServerMessage::Snapshot {
        room,
        state: projected.state,
        events: projected.events,
    })
    .unwrap();
    FixtureCase::new(case_id, frame)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCase {
    case_id: String,
    value: serde_json::Value,
}

impl FixtureCase {
    fn new(case_id: &str, frame: String) -> Self {
        Self {
            case_id: case_id.into(),
            value: serde_json::from_str(&frame).unwrap(),
        }
    }

    fn value(case_id: &str, value: serde_json::Value) -> Self {
        Self {
            case_id: case_id.into(),
            value,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FixtureCorpus {
    producer: &'static str,
    valid_messages: Vec<FixtureCase>,
    invalid_messages: Vec<FixtureCase>,
}

pub(crate) fn corpus() -> FixtureCorpus {
    let lobby = room(0, RoomPhase::Waiting, true);
    let lobby_frame = encode_server_message(&GameServerMessage::RoomState {
        room: lobby.clone(),
    })
    .unwrap();
    let admitted = prepare_admission(
        AdmissionProjection {
            room: &lobby,
            state_epoch: None,
            capture: None,
            generation: None,
        },
        PersonalAdmission {
            recipient: Seat::One,
            last_action_id: None,
            sequence: Counter::new(1).unwrap(),
            resume_token: ResumeToken::new("c".repeat(64)).unwrap(),
        },
    )
    .unwrap();
    let lobby_value: serde_json::Value = serde_json::from_str(&lobby_frame).unwrap();
    let snapshot = snapshot_case("hard-snapshot", true, false, DomainPhase::Idle);
    let mut mixed = lobby_value.clone();
    mixed["room"]["seats"][1]["controller"]["difficulty"] = "standard".into();
    let mut null = lobby_value.clone();
    null["room"]["configuration"]["cpuDifficulty"] = serde_json::Value::Null;
    let mut extra = lobby_value.clone();
    extra["room"]["configuration"]["planner"] = "aggressive".into();
    let mut privacy = snapshot.value.clone();
    privacy["room"]["seats"][1]["controller"]["route"] =
        serde_json::json!([{"column": 2, "row": 1}]);
    FixtureCorpus {
        producer: "Rust gameplay v19 typed projection; policy execution remains dormant",
        valid_messages: vec![
            FixtureCase::value(
                "v19-authenticated",
                serde_json::json!({
                    "type": "authenticated", "protocolVersion": 19
                }),
            ),
            FixtureCase::new("hard-room-state", lobby_frame),
            FixtureCase::new("hard-admission", admitted),
            snapshot,
        ],
        invalid_messages: vec![
            FixtureCase::value("mixed-cpu-difficulties", mixed),
            FixtureCase::value("cpu-room-null-difficulty", null),
            FixtureCase::value("extra-room-planner-field", extra),
            FixtureCase::value("private-cpu-route", privacy),
            FixtureCase::value(
                "v18-identity",
                serde_json::json!({
                    "type": "authenticated", "protocolVersion": 18
                }),
            ),
        ],
    }
}

#[test]
fn checked_server_corpus_is_rust_generated() {
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/fixtures/gameplay-v19-server-conformance.json"
    )))
    .unwrap();
    assert_eq!(serde_json::to_value(corpus()).unwrap(), expected);
}

#[test]
#[ignore = "explicit fixture regeneration utility"]
fn regenerate_server_corpus() {
    std::fs::write(
        FIXTURE,
        serde_json::to_string_pretty(&corpus()).unwrap() + "\n",
    )
    .unwrap();
}
