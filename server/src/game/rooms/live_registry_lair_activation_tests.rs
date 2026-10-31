use super::*;

fn cpu_configuration() -> LairRoomConfiguration {
    LairRoomConfiguration::new(
        RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(CpuDifficulty::Standard),
            BoardSize::FifteenByThirteen,
        )
        .unwrap(),
        StormCloudConfig::Single,
        LairTuning::new(true, LairSettings::new(5, 1).unwrap()),
    )
}

async fn advance_ticks(count: usize) {
    for _ in 0..count {
        tokio::time::advance(std::time::Duration::from_millis(17)).await;
        tokio::task::yield_now().await;
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn live_lair_cpu_progresses_through_checked_v18_snapshots() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let mut admission = rooms
        .create_live(binding(1), sequence(1), now, cpu_configuration())
        .await
        .unwrap();
    let room_id = admission.room_id().clone();
    rooms
        .action_live(
            &admission,
            binding(1),
            room_id,
            1,
            super::super::RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .await
        .unwrap();

    advance_ticks(190).await;
    let LiveDelivery::Publication(first) = admission.latest_delivery().unwrap() else {
        panic!("active CPU room unexpectedly closed");
    };
    let first: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(first["type"], "snapshot");
    assert_eq!(first["room"]["rulesVersion"], 10);
    assert_eq!(first["room"]["phase"]["type"], "running");
    let initial_tick = first["state"]["physicalTick"].as_u64().unwrap();

    advance_ticks(20).await;
    let LiveDelivery::Publication(progressed) = admission.latest_delivery().unwrap() else {
        panic!("active CPU room unexpectedly closed");
    };
    let progressed: serde_json::Value = serde_json::from_str(&progressed).unwrap();
    assert_eq!(progressed["type"], "snapshot");
    let progressed_tick = progressed["state"]["physicalTick"].as_u64().unwrap();
    assert!(progressed_tick > initial_tick);
    assert_eq!(
        progressed["state"]["runningUpdates"].as_u64().unwrap(),
        progressed_tick - 180
    );
    assert_eq!(
        progressed["room"]["tick"].as_u64().unwrap(),
        progressed_tick
    );
}
