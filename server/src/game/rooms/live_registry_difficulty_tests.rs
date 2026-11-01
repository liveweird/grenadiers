use super::*;

#[tokio::test]
async fn all_live_cpu_tiers_publish_only_gameplay19_with_room_wide_identity() {
    for (difficulty, wire) in [
        (CpuDifficulty::Easy, "easy"),
        (CpuDifficulty::Standard, "standard"),
        (CpuDifficulty::Hard, "hard"),
    ] {
        let now = std::time::Instant::now();
        let rooms = rooms(now);
        let admission = rooms
            .create_live(
                binding(1),
                sequence(1),
                now,
                configuration(
                    &[
                        SeatControl::Human,
                        SeatControl::Cpu,
                        SeatControl::Cpu,
                        SeatControl::Cpu,
                    ],
                    Some(difficulty),
                ),
            )
            .await
            .unwrap();
        let frame = personal(&admission);
        // Gameplay version is negotiated by authentication, not repeated in admission.
        assert_eq!(frame["type"], "admitted");
        let room = &frame["room"];
        assert_eq!(room["configuration"]["cpuDifficulty"], wire);
        for row in &room["seats"].as_array().unwrap()[1..] {
            assert_eq!(
                row["controller"],
                serde_json::json!({"type":"cpu", "difficulty":wire})
            );
            assert_eq!(row["connection"]["type"], "serverControlled");
        }
    }
}
