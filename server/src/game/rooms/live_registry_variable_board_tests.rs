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
            let admission = rooms
                .create_live(
                    binding(connection),
                    sequence(1),
                    now + std::time::Duration::from_millis((connection - 100) * 125),
                    configuration_on(&vec![SeatControl::Human; count], None, board),
                )
                .await
                .unwrap();
            let legacy = count == 2 && board == BoardSize::FifteenByThirteen;
            assert_eq!(matches!(admission, LiveAdmission::Legacy(_)), legacy);
            assert_eq!(
                personal(&admission)["room"]["rulesVersion"],
                if legacy { 5 } else { 6 }
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

#[tokio::test]
async fn small_two_player_cpu_rooms_use_policy_seventeen_roster_for_both_difficulties() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    for (index, difficulty) in [CpuDifficulty::Easy, CpuDifficulty::Standard]
        .into_iter()
        .enumerate()
    {
        let admission = rooms
            .create_live(
                binding(index as u64 + 1),
                sequence(1),
                now + std::time::Duration::from_millis(index as u64 * 125),
                configuration_on(
                    &[SeatControl::Human, SeatControl::Cpu],
                    Some(difficulty),
                    BoardSize::ElevenByNine,
                ),
            )
            .await
            .unwrap();
        assert!(matches!(admission, LiveAdmission::Roster(_)));
        assert_eq!(personal(&admission)["room"]["rulesVersion"], 6);
        assert_eq!(
            personal(&admission)["room"]["configuration"]["cpuDifficulty"],
            match difficulty {
                CpuDifficulty::Easy => "easy",
                CpuDifficulty::Standard => "standard",
            }
        );
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
