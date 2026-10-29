use serde::{Deserialize, Serialize};

use crate::game::{
    ActorInit, ArenaGenerationConfig, EffectKind, EnabledItems, PlayerCount, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig,
    World, generate_multiplayer_arena, generate_variable_multiplayer_arena,
};
use crate::protocol::Counter;
use crate::room::BoardSize as DomainBoard;
use crate::room::{LairConfig, LairSettings, LairTuning};

use super::*;

const CHECKED_CORPUS: &str =
    include_str!("../../../../api/fixtures/gameplay-v16-server-conformance.json");

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
    lairs: LairConfig,
    advance_ticks: usize,
    effects: (bool, bool),
    seed: [u8; 32],
) -> WorldCase {
    let (cloak_center_actor, freeze_mobs) = effects;
    let enabled = if cloak_center_actor {
        EnabledItems::INVISIBILITY_CLOAK
    } else if freeze_mobs {
        EnabledItems::STOPWATCH
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
    let rules = RulesConfig::environment_v8(
        crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
        cloud,
        lairs,
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
    for _ in 0..advance_ticks {
        state.step(&[]).unwrap();
        lifecycle.complete_running_tick(None).unwrap();
    }
    if freeze_mobs {
        state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, state.tick());
    }
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
        room: super::server_conformance_lifecycle::room_view_with_tuning(
            &lifecycle,
            cloud,
            match lairs {
                LairConfig::Disabled => LairTuning::new(false, LairSettings::DEFAULT),
                LairConfig::Enabled(settings) => LairTuning::new(true, settings),
            },
            true,
        ),
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
        LairConfig::Disabled,
        0,
        (false, false),
        [11; 32],
    );
    let single = world_case(
        DomainBoard::ThirteenByEleven,
        PlayerCount::Three,
        StormCloudConfig::Single,
        LairConfig::default(),
        0,
        (false, false),
        [13; 32],
    );
    let privacy = world_case(
        DomainBoard::FifteenByThirteen,
        PlayerCount::Four,
        StormCloudConfig::Single,
        LairConfig::default(),
        0,
        (true, false),
        [15; 32],
    );
    let populated = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap()),
        300,
        (false, false),
        [16; 32],
    );
    let frozen = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap()),
        300,
        (false, true),
        [17; 32],
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
        &populated,
        "populated11x9Lairs",
        &[Seat::One],
    ));
    cases.extend(snapshot_cases(&frozen, "frozen11x9Lairs", &[Seat::One]));
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
fn checked_corpus_matches_fresh_checked_v16_projection() {
    let actual = actual_corpus();
    let expected: Corpus = serde_json::from_str(CHECKED_CORPUS).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn checked_frames_satisfy_the_v16_server_schema() {
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v16.schema.json")).unwrap();
    schema["$ref"] = serde_json::json!("#/$defs/GameServerMessage");
    let validator = jsonschema::validator_for(&schema).unwrap();
    for case in actual_corpus().cases {
        let frame: serde_json::Value = serde_json::from_str(&case.frame).unwrap();
        assert!(validator.is_valid(&frame), "{}", case.case_id);
    }
}

#[test]
fn checked_publication_rejects_impossible_lair_facts() {
    let case = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap()),
        300,
        (false, false),
        [16; 32],
    );
    let mutations: &[fn(&mut ExpandedCapture)] = &[
        |capture| capture.lairs[0].spawn_remaining_ticks -= 1,
        |capture| capture.lairs[0].cell = crate::protocol::Cell::new(5, 4).unwrap(),
        |capture| capture.lairs[0].living_mobs += 1,
        |capture| capture.mobs[0].id = capture.base.world.actors[0].view.id,
        |capture| capture.mobs[0].lair_id = 1,
        |capture| capture.mobs[0].frozen = true,
    ];
    for mutate in mutations {
        let mut capture = case.capture.clone();
        mutate(&mut capture);
        assert!(matches!(
            bind_expanded(&case.room, case.room.round_id, &capture, case.generation),
            Err(ProjectionError::InvalidExpandedBinding)
        ));
    }
}

#[test]
fn cpu_observation_uses_checked_rules_eight_lair_facts() {
    let case = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap()),
        300,
        (false, true),
        [17; 32],
    );
    let publication = bind_expanded(
        &case.room,
        case.room.round_id,
        &case.capture,
        case.generation,
    )
    .unwrap();
    let world = crate::cpu::WorldId {
        room_instance: 7,
        physical_round: u64::from(case.room.round_id.get()),
    };
    let observed = observe_cpu_recipient(&publication, Seat::One, world).unwrap();
    assert_eq!(observed.parts().rules.version, 8);
    let lairs = observed.lairs().unwrap();
    assert_eq!(
        lairs.config(),
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap())
    );
    assert_eq!(lairs.lairs().len(), 2);
    assert_eq!(lairs.lairs()[0].next_spawn_at.0, case.room.tick.get() + 300);
    assert_eq!(lairs.mobs().len(), 2);
    assert!(lairs.mobs().iter().all(|mob| mob.frozen));

    let wrong = crate::cpu::WorldId {
        physical_round: 2,
        ..world
    };
    assert_eq!(
        observe_cpu_recipient(&publication, Seat::One, wrong),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}
