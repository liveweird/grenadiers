use super::*;

#[tokio::test]
async fn all_board_and_count_profiles_dispatch_to_the_required_owner() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let mut connection = 100_u64;
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in 2..=4 {
            for environment in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
                let admission = rooms
                    .create_live(
                        binding(connection),
                        sequence(1),
                        now + std::time::Duration::from_millis((connection - 100) * 125),
                        configuration_on_environment(
                            &vec![SeatControl::Human; count],
                            None,
                            board,
                            environment,
                        ),
                    )
                    .await
                    .unwrap();
                assert!(matches!(admission, LiveAdmission::Roster(_)));
                assert_eq!(personal(&admission)["room"]["rulesVersion"], 8);
                assert_eq!(
                    personal(&admission)["room"]["settings"]["environment"]["stormCloud"],
                    environment == StormCloudConfig::Single
                );
                let (width, height) = board.dimensions();
                assert_eq!(
                    personal(&admission)["room"]["configuration"]["board"],
                    serde_json::json!({ "width": width, "height": height })
                );
                connection += 1;
            }
        }
    }
}

#[tokio::test]
async fn every_cpu_board_and_count_uses_policy_nineteen_for_both_environments() {
    let now = std::time::Instant::now();
    let mut index = 0_u64;
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let rooms = rooms(now);
        for count in 2..=4 {
            for environment in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
                for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
                    let mut controllers = vec![SeatControl::Human; count];
                    controllers[count - 1] = SeatControl::Cpu;
                    let admission = rooms
                        .create_live(
                            binding(index + 1),
                            sequence(1),
                            now + std::time::Duration::from_millis(index * 125),
                            configuration_on_environment(
                                &controllers,
                                Some(difficulty),
                                board,
                                environment,
                            ),
                        )
                        .await
                        .unwrap();
                    assert!(matches!(admission, LiveAdmission::Roster(_)));
                    assert_eq!(personal(&admission)["room"]["rulesVersion"], 8);
                    assert_eq!(
                        personal(&admission)["room"]["configuration"]["cpuDifficulty"],
                        match difficulty {
                            CpuDifficulty::Easy => "easy",
                            CpuDifficulty::Standard => "standard",
                        }
                    );
                    index += 1;
                }
            }
        }
    }
}

#[tokio::test]
async fn small_board_resume_preserves_immutable_configuration() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let first = rooms
        .create_live(
            binding(1),
            sequence(1),
            now,
            configuration_on(&[SeatControl::Human; 2], None, BoardSize::ThirteenByEleven),
        )
        .await
        .unwrap();
    let resumed = rooms
        .resume_live(
            first.room_id().clone(),
            binding(2),
            sequence(2),
            token(&first),
            now,
        )
        .await
        .unwrap();
    assert_eq!(
        personal(&resumed)["room"]["configuration"]["board"],
        serde_json::json!({ "width": 13, "height": 11 })
    );
}
