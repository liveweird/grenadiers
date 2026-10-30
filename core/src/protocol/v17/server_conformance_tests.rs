use serde::{Deserialize, Serialize};

use crate::game::{EnabledItems, PlayerCount, StormCloudConfig};
use crate::room::{BoardSize as DomainBoard, RespawnLairConfig, RespawnLairSettings};

use super::*;

const CHECKED_CORPUS: &str =
    include_str!("../../../../api/fixtures/gameplay-v17-server-conformance.json");

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FrameCase {
    case_id: String,
    scenario: String,
    recipient: Option<Seat>,
    frame: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct Corpus {
    cases: Vec<FrameCase>,
}

pub(super) use super::server_test_support::{WorldCase, counter, world_case};

pub(super) fn snapshot_cases(
    case: &WorldCase,
    scenario: &str,
    recipients: &[Seat],
) -> Vec<FrameCase> {
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
        RespawnLairConfig::Disabled,
        0,
        (false, false),
        [11; 32],
    );
    let single = world_case(
        DomainBoard::ThirteenByEleven,
        PlayerCount::Three,
        StormCloudConfig::Single,
        RespawnLairConfig::default(),
        0,
        (false, false),
        [13; 32],
    );
    let privacy = world_case(
        DomainBoard::FifteenByThirteen,
        PlayerCount::Four,
        StormCloudConfig::Single,
        RespawnLairConfig::default(),
        0,
        (true, false),
        [15; 32],
    );
    let populated = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        300,
        (false, false),
        [16; 32],
    );
    let frozen = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
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
        case_id: "disabled-11x9-cpu21-admission".into(),
        scenario: "waitingAdmission".into(),
        recipient: Some(Seat::One),
        frame: admission,
    }];
    cases.extend(snapshot_cases(&disabled, "disabled11x9Cpu21", &[Seat::One]));
    cases.extend(snapshot_cases(
        &populated,
        "populated11x9Lairs",
        &[Seat::One],
    ));
    cases.extend(snapshot_cases(&frozen, "frozen11x9Lairs", &[Seat::One]));
    cases.extend(snapshot_cases(
        &single,
        "single13x11Cpu21Timer",
        &[Seat::One],
    ));
    cases.push(FrameCase {
        case_id: "single-15x13-cpu21-room-state".into(),
        scenario: "roomState".into(),
        recipient: None,
        frame: encode_server_message(&GameServerMessage::RoomState {
            room: privacy.room.clone(),
        })
        .unwrap(),
    });
    cases.extend(snapshot_cases(
        &privacy,
        "single15x13Cpu21Cloak",
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
    cases.extend(super::respawn_corpus_cases::cases());
    cases.extend(super::mine_corpus_cases::cases());
    Corpus { cases }
}

#[test]
fn checked_corpus_matches_fresh_checked_v17_projection() {
    let actual = actual_corpus();
    if std::env::var_os("GRENADIERS_UPDATE_GAMEPLAY_V17_CORPUS").is_some() {
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../api/fixtures/gameplay-v17-server-conformance.json"
            ),
            serde_json::to_string_pretty(&actual).unwrap() + "\n",
        )
        .unwrap();
        return;
    }
    let expected: Corpus = serde_json::from_str(CHECKED_CORPUS).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn checked_frames_satisfy_the_v17_server_schema() {
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v17.schema.json")).unwrap();
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
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        300,
        (false, false),
        [16; 32],
    );
    let mutations: &[fn(&mut ExpandedCapture)] = &[
        |capture| capture.lairs[0].spawn_remaining_ticks = Some(1),
        |capture| capture.lairs[0].cell = crate::protocol::Cell::new(5, 4).unwrap(),
        |capture| capture.lairs[0].living_mobs += 1,
        |capture| capture.mobs[0].id = capture.base.world.actors[0].view.id,
        |capture| capture.mobs[0].lair_id = 1,
        |capture| capture.mobs[0].frozen = true,
        |capture| capture.base.world.arena.terrain[48] = Terrain::Floor,
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
fn cpu_observation_uses_checked_rules_nine_lair_facts() {
    let case = world_case(
        DomainBoard::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
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
    assert_eq!(observed.parts().rules.version, 9);
    let lairs = observed.respawn_lairs().unwrap();
    assert_eq!(
        lairs.config(),
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    );
    assert_eq!(lairs.lairs().len(), 2);
    assert_eq!(lairs.lairs()[0].next_spawn_at, None);
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
