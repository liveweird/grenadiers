use super::*;

fn abandoned(cpu_count: usize, completed: bool) -> RoomView {
    let tick = crate::protocol::Tick::new(190).unwrap();
    let mut room = super::tests::running_room(crate::game::PlayerCount::Three, tick);
    let controls = if cpu_count == 2 {
        [
            crate::room::SeatControl::Human,
            crate::room::SeatControl::Cpu,
            crate::room::SeatControl::Cpu,
        ]
    } else {
        [
            crate::room::SeatControl::Human,
            crate::room::SeatControl::Cpu,
            crate::room::SeatControl::Human,
        ]
    };
    room.configuration = RoomConfiguration::from_domain(
        crate::room::RoomConfiguration::new(
            &controls,
            Some(crate::room::CpuDifficulty::Standard),
            crate::room::BoardSize::FifteenByThirteen,
        )
        .unwrap(),
    )
    .unwrap();
    room.settings_editor = None;
    for (status, control) in room.seats.iter_mut().zip(controls) {
        match control {
            crate::room::SeatControl::Human => status.connection = ConnectionStatus::Expired,
            crate::room::SeatControl::Cpu => {
                status.controller = Controller::Cpu {
                    difficulty: Difficulty::Standard,
                };
                status.connection = ConnectionStatus::ServerControlled;
            }
        }
    }
    let ended_at = crate::protocol::Tick::new(10).unwrap();
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result: MatchResult::Abandoned,
        rematch_at: tick,
    };
    for status in &mut room.seats {
        status.rematch_ready = matches!(status.controller, Controller::Cpu { .. });
    }
    if completed {
        room.scores[1].wins = Wins::new(1).unwrap();
        room.last_round = Some(CompletedRound {
            round_id: room.round_id,
            round_number: crate::protocol::Counter::new(1).unwrap(),
            ended_at,
            result: RoundResult::Winner { seat: Seat::Two },
        });
    }
    room
}

#[test]
fn abandoned_mixed_rooms_keep_server_controlled_cpu_rows() {
    for count in 1..=2 {
        for completed in [false, true] {
            let room = abandoned(count, completed);
            assert!(
                validate_room_view(&room).is_ok(),
                "{count} CPUs completed={completed}"
            );
            let mut before_hold = room;
            before_hold.tick = crate::protocol::Tick::new(10).unwrap();
            for status in &mut before_hold.seats {
                status.rematch_ready = false;
            }
            assert!(validate_room_view(&before_hold).is_ok());
        }
    }
}

#[test]
fn abandonment_rejects_connected_and_reserved_humans() {
    for connection in [
        ConnectionStatus::Connected,
        ConnectionStatus::Disconnected {
            expires_at: crate::protocol::Tick::new(200).unwrap(),
        },
    ] {
        let mut room = abandoned(2, false);
        room.seats[0].connection = connection;
        room.settings_editor = Some(Seat::One);
        assert_eq!(
            validate_room_view(&room),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}
