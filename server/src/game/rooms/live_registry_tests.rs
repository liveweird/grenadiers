use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, v14},
    room::{BoardSize, CpuDifficulty, RoomConfiguration, SeatControl},
};

use super::{BindingKey, GameRooms, LiveAdmission, LiveDelivery, RoomError};
use crate::game::sessions::ConnectionId;

fn rooms(now: std::time::Instant) -> Arc<GameRooms> {
    let codes = AtomicU64::new(0);
    GameRooms::with_random_fill_and_round_seed(
        now,
        move |bytes| {
            if bytes.len() == 8 {
                bytes.copy_from_slice(&0x1234_5678_9abc_def0_u64.to_be_bytes());
            } else if bytes.len() == 5 {
                let value = codes.fetch_add(1, Ordering::Relaxed).to_be_bytes();
                bytes.copy_from_slice(&value[3..]);
            } else {
                bytes.fill(codes.fetch_add(1, Ordering::Relaxed) as u8 + 1);
            }
            Ok(())
        },
        crate::round_seed::RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(23);
            Ok(())
        }),
    )
}

fn binding(connection: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection),
        generation: 1,
    }
}

fn sequence(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn configuration(controls: &[SeatControl], difficulty: Option<CpuDifficulty>) -> RoomConfiguration {
    RoomConfiguration::new(controls, difficulty, BoardSize::FifteenByThirteen).unwrap()
}

fn personal(admission: &LiveAdmission) -> serde_json::Value {
    serde_json::from_str(admission.personal_frame()).unwrap()
}

fn token(admission: &LiveAdmission) -> v14::ResumeToken {
    v14::ResumeToken::new(
        personal(admission)["resumeToken"]
            .as_str()
            .unwrap()
            .to_owned(),
    )
    .unwrap()
}

#[tokio::test]
async fn two_player_live_rooms_use_v14_legacy_owner_for_duel_and_solo() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let duel = rooms
        .create_live(
            binding(1),
            sequence(1),
            now,
            configuration(&[SeatControl::Human; 2], None),
        )
        .await
        .unwrap();
    assert_eq!(duel.seat(), SeatId::One);
    assert_eq!(duel.room_id().as_str(), "123456789abcdef00000000000000001");
    assert_eq!(personal(&duel)["room"]["rulesVersion"], 5);
    assert_eq!(
        personal(&duel)["room"]["configuration"]["controllers"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let solo = rooms
        .create_live(
            binding(2),
            sequence(1),
            now + std::time::Duration::from_millis(125),
            configuration(
                &[SeatControl::Human, SeatControl::Cpu],
                Some(CpuDifficulty::Easy),
            ),
        )
        .await
        .unwrap();
    assert_eq!(solo.seat(), SeatId::One);
    assert_eq!(solo.room_id().as_str(), "123456789abcdef00000000000000002");
    assert_eq!(
        personal(&solo)["room"]["configuration"]["cpuDifficulty"],
        "easy"
    );

    let roster = rooms
        .create_live(
            binding(3),
            sequence(1),
            now + std::time::Duration::from_millis(250),
            configuration(&[SeatControl::Human; 3], None),
        )
        .await
        .unwrap();
    assert_eq!(
        roster.room_id().as_str(),
        "123456789abcdef00000000000000003"
    );
}

#[tokio::test]
async fn three_and_four_player_rosters_admit_only_human_seats_and_resume_privately() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let first = rooms
        .create_live(
            binding(1),
            sequence(1),
            now,
            configuration(
                &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
                Some(CpuDifficulty::Standard),
            ),
        )
        .await
        .unwrap();
    let code = match &first {
        LiveAdmission::Roster(value) => value.room.room_code.clone(),
        LiveAdmission::Legacy(_) => panic!("three-player room used legacy owner"),
    };
    let third = rooms
        .join_live(&code, binding(3), sequence(1), now)
        .await
        .unwrap();
    assert_eq!(third.seat(), SeatId::Three);
    let replacement = rooms
        .resume_live(
            third.room_id().clone(),
            binding(30),
            sequence(2),
            token(&third),
            now,
        )
        .await
        .unwrap();
    assert_eq!(replacement.seat(), SeatId::Three);
    assert_eq!(personal(&replacement)["youSeat"], "three");

    let four = rooms
        .create_live(
            binding(10),
            sequence(1),
            now + std::time::Duration::from_millis(125),
            configuration(&[SeatControl::Human; 4], None),
        )
        .await
        .unwrap();
    let code = match &four {
        LiveAdmission::Roster(value) => value.room.room_code.clone(),
        LiveAdmission::Legacy(_) => panic!("four-player room used legacy owner"),
    };
    let mut fourth = None;
    for (connection, seat) in [(11, SeatId::Two), (12, SeatId::Three), (13, SeatId::Four)] {
        let joined = rooms
            .join_live(&code, binding(connection), sequence(1), now)
            .await
            .unwrap();
        assert_eq!(joined.seat(), seat);
        if seat == SeatId::Four {
            fourth = Some(joined);
        }
    }
    let fourth = fourth.unwrap();
    let replacement = rooms
        .resume_live(
            fourth.room_id().clone(),
            binding(130),
            sequence(2),
            token(&fourth),
            now,
        )
        .await
        .unwrap();
    assert_eq!(replacement.seat(), SeatId::Four);
    assert_eq!(personal(&replacement)["youSeat"], "four");

    let cpu_roster = rooms
        .create_live(
            binding(20),
            sequence(1),
            now + std::time::Duration::from_millis(250),
            configuration(
                &[
                    SeatControl::Human,
                    SeatControl::Cpu,
                    SeatControl::Cpu,
                    SeatControl::Cpu,
                ],
                Some(CpuDifficulty::Easy),
            ),
        )
        .await
        .unwrap();
    let code = match &cpu_roster {
        LiveAdmission::Roster(value) => value.room.room_code.clone(),
        LiveAdmission::Legacy(_) => panic!("four-player CPU room used legacy owner"),
    };
    assert!(matches!(
        rooms.join_live(&code, binding(21), sequence(1), now).await,
        Err(RoomError::RoomFull)
    ));
}

#[tokio::test]
async fn unsupported_board_is_rejected_before_registry_identity_entropy() {
    let now = std::time::Instant::now();
    let calls = Arc::new(AtomicU64::new(0));
    let observed = Arc::clone(&calls);
    let rooms = GameRooms::with_random_fill(now, move |_| {
        observed.fetch_add(1, Ordering::Relaxed);
        Ok(())
    });
    let unsupported =
        RoomConfiguration::new(&[SeatControl::Human; 3], None, BoardSize::ThirteenByEleven)
            .unwrap();
    assert!(matches!(
        rooms
            .create_live(binding(1), sequence(1), now, unsupported)
            .await,
        Err(RoomError::UnsupportedConfiguration)
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let registry = rooms.lock();
    assert_eq!(registry.next_serial, 1);
    assert!(registry.prefix.is_none());
    assert!(registry.rooms.is_empty());
}

#[tokio::test]
async fn failed_roster_cpu_entropy_does_not_insert_or_consume_serial() {
    let now = std::time::Instant::now();
    let calls = Arc::new(AtomicU64::new(0));
    let observed = Arc::clone(&calls);
    let rooms = GameRooms::with_random_fill(now, move |bytes| {
        let call = observed.fetch_add(1, Ordering::Relaxed);
        if call == 2 {
            return Err(());
        }
        bytes.fill(7);
        Ok(())
    });
    let result = rooms
        .create_live(
            binding(1),
            sequence(1),
            now,
            configuration(
                &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
                Some(CpuDifficulty::Easy),
            ),
        )
        .await;
    assert!(matches!(result, Err(RoomError::CapacityReached)));
    let registry = rooms.lock();
    assert_eq!(registry.next_serial, 1);
    assert!(registry.rooms.is_empty());
    assert!(registry.codes.is_empty());
}

#[tokio::test]
async fn canceled_creator_future_removes_registry_entry_before_owner_admission() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    {
        let create = rooms.create_live(
            binding(1),
            sequence(1),
            now,
            configuration(&[SeatControl::Human; 4], None),
        );
        tokio::pin!(create);
        tokio::select! {
            biased;
            _ = &mut create => panic!("owner admitted creator synchronously"),
            () = std::future::ready(()) => {}
        }
    }
    assert!(rooms.lock().rooms.is_empty());
    assert!(rooms.lock().codes.is_empty());
}

#[tokio::test]
async fn bodyless_roster_notice_is_encoded_for_the_actual_recipient() {
    let now = std::time::Instant::now();
    let rooms = rooms(now);
    let mut admission = rooms
        .create_live(
            binding(1),
            sequence(1),
            now,
            configuration(&[SeatControl::Human; 4], None),
        )
        .await
        .unwrap();
    let LiveDelivery::Publication(frame) = admission.latest_delivery().unwrap() else {
        panic!("new room unexpectedly closed");
    };
    let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
    assert_eq!(value["type"], "roomState");
    assert_eq!(
        value["room"]["configuration"]["controllers"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[tokio::test]
async fn legacy_and_roster_owners_share_the_single_room_capacity() {
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
