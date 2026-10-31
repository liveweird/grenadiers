use super::*;

#[tokio::test]
async fn environment_roster_owners_share_the_single_room_capacity() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let mut admissions = Vec::new();
    for index in 0..31_u64 {
        admissions.push(
            rooms
                .create_live(
                    binding(index + 1),
                    sequence(1),
                    now + std::time::Duration::from_millis(index * 125),
                    configuration(&[SeatControl::Human; 2], None),
                )
                .await
                .unwrap(),
        );
    }
    admissions.push(
        rooms
            .create_live(
                binding(32),
                sequence(1),
                now + std::time::Duration::from_millis(31 * 125),
                configuration(&[SeatControl::Human; 4], None),
            )
            .await
            .unwrap(),
    );
    assert!(matches!(
        rooms
            .create_live(
                binding(33),
                sequence(1),
                now + std::time::Duration::from_millis(32 * 125),
                configuration(&[SeatControl::Human; 3], None),
            )
            .await,
        Err(RoomError::CapacityReached)
    ));
}
