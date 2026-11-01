use crate::{
    game::StormCloudConfig,
    protocol::{v14, v15, v16, v17, v18},
    room::{
        BoardSize, CpuDifficulty, EnvironmentRoomConfiguration, LairRoomConfiguration,
        LairSettings, LairTuning, RespawnLairSettings, RespawnLairTuning, RoomConfiguration,
        SeatControl, SpikePitSettings, SpikePitTuning, SpikeRoomConfiguration,
    },
};

fn hard_base() -> RoomConfiguration {
    RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap()
}

#[test]
fn v19_configuration_round_trips_hard_directly_to_the_domain() {
    let source = r#"{"controllers":["human","cpu"],"cpuDifficulty":"hard","board":{"width":15,"height":13},"environment":{"stormCloud":false,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1},"spikePits":{"enabled":true,"warningTicks":36,"raisedTicks":180}}}"#;
    let wire: super::RoomConfiguration = serde_json::from_str(source).unwrap();
    assert_eq!(
        wire.domain().base().base().cpu_difficulty(),
        Some(CpuDifficulty::Hard)
    );
    assert_eq!(serde_json::to_string(&wire).unwrap(), source);
}

#[test]
fn archived_configuration_projections_explicitly_reject_hard() {
    let base = hard_base();
    assert_eq!(
        v14::RoomConfiguration::from_domain(base),
        Err(v14::ArchivedConfigurationError::UnsupportedDifficulty)
    );
    assert_eq!(
        v15::RoomConfiguration::from_domain(EnvironmentRoomConfiguration::new(
            base,
            StormCloudConfig::Disabled,
        )),
        Err(v14::ArchivedConfigurationError::UnsupportedDifficulty)
    );
    let lairs = LairTuning::new(false, LairSettings::new(10, 1).unwrap());
    assert_eq!(
        v16::RoomConfiguration::from_domain(LairRoomConfiguration::new(
            base,
            StormCloudConfig::Disabled,
            lairs,
        )),
        Err(v14::ArchivedConfigurationError::UnsupportedDifficulty)
    );
    let respawn = RespawnLairTuning::new(false, RespawnLairSettings::new(10).unwrap());
    let respawn_room =
        LairRoomConfiguration::new(base, StormCloudConfig::Disabled, respawn.as_legacy());
    assert_eq!(
        v17::RoomConfiguration::from_domain(respawn_room.clone()),
        Err(v14::ArchivedConfigurationError::UnsupportedDifficulty)
    );
    let spike = SpikeRoomConfiguration::new(
        respawn_room,
        SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
    );
    assert_eq!(
        v18::RoomConfiguration::from_domain(spike),
        Err(v14::ArchivedConfigurationError::UnsupportedDifficulty)
    );
}

#[test]
fn checked_client_corpus_accepts_only_closed_v19_messages() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/fixtures/gameplay-v19-client-conformance.json"
    )))
    .unwrap();
    for case in corpus["validMessages"].as_array().unwrap() {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(
            super::parse_client_message(&encoded).is_ok(),
            "{}",
            case["caseId"]
        );
    }
    for case in corpus["invalidMessages"].as_array().unwrap() {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(
            super::parse_client_message(&encoded).is_err(),
            "{}",
            case["caseId"]
        );
    }
}
