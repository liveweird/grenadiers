use super::*;
use grenadiers_core::room::{BoardSize, SeatControl};

fn configured(count: usize) -> InputMailbox {
    InputMailbox::for_configuration(
        RoomConfiguration::new(&vec![SeatControl::Human; count], None, BoardSize::default())
            .unwrap(),
    )
}

#[test]
fn four_seats_drain_movement_before_manual_actions_in_canonical_order() {
    let mut mailbox = configured(4);
    for seat in SeatId::ALL.into_iter().rev() {
        mailbox.bomb(seat, 1).unwrap();
        mailbox
            .movement(seat, Some(Direction::East), Tick(0))
            .unwrap();
        mailbox.equipment(seat, 2).unwrap();
    }
    let inputs = mailbox.take(Tick(1));
    assert_eq!(inputs.len(), 12);
    for (input, seat) in inputs[..4].iter().zip(SeatId::ALL) {
        assert_eq!(
            *input,
            SeatInput {
                seat,
                action: InputAction::Press(Direction::East)
            }
        );
    }
    for (inputs, seat) in inputs[4..].as_chunks::<2>().0.iter().zip(SeatId::ALL) {
        assert_eq!(
            *inputs,
            [
                SeatInput {
                    seat,
                    action: InputAction::PlaceBomb
                },
                SeatInput {
                    seat,
                    action: InputAction::UseEquipment
                },
            ]
        );
        assert_eq!(mailbox.last_manual_action(seat), Some(2));
    }
}

#[test]
fn absent_seats_reject_commands_without_panicking_or_affecting_real_inputs() {
    for count in 2..=3 {
        let mut mailbox = configured(count);
        mailbox.bomb(SeatId::One, 1).unwrap();
        for &seat in &SeatId::ALL[count..] {
            assert_eq!(mailbox.bomb(seat, 1), Err(InputError::UnknownSeat));
            assert_eq!(mailbox.equipment(seat, 1), Err(InputError::UnknownSeat));
            assert_eq!(
                mailbox.movement(seat, Some(Direction::East), Tick(0)),
                Err(InputError::UnknownSeat)
            );
            assert_eq!(mailbox.last_manual_action(seat), None);
            mailbox.stop(seat);
        }
        assert_eq!(
            mailbox.take(Tick(1)),
            [SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb
            }]
        );
    }
}

#[test]
fn higher_seat_budget_fencing_and_round_reset_are_independent() {
    let mut mailbox = configured(4);
    for id in 1..=u32::from(COMMANDS_PER_UPDATE) {
        mailbox.bomb(SeatId::Three, id).unwrap();
    }
    assert_eq!(mailbox.bomb(SeatId::Three, 9), Err(InputError::RateLimited));
    mailbox.equipment(SeatId::Four, 1).unwrap();
    mailbox.stop(SeatId::Three);
    assert_eq!(
        mailbox.take(Tick(1)),
        [
            SeatInput {
                seat: SeatId::Three,
                action: InputAction::Clear
            },
            SeatInput {
                seat: SeatId::Four,
                action: InputAction::UseEquipment
            },
        ]
    );
    assert_eq!(
        mailbox.bomb(SeatId::Three, 8),
        Err(InputError::DuplicateAction)
    );
    assert_eq!(
        mailbox.bomb(SeatId::Four, 1),
        Err(InputError::DuplicateAction)
    );
    mailbox.reset_round();
    for seat in SeatId::ALL {
        mailbox.bomb(seat, 1).unwrap();
    }
    assert_eq!(mailbox.take(Tick(2)).len(), 4);
}

#[test]
fn higher_seat_movement_ttl_and_pending_clear_preserve_deduplication() {
    let mut mailbox = configured(4);
    for seat in [SeatId::Three, SeatId::Four] {
        mailbox
            .movement(seat, Some(Direction::West), Tick(0))
            .unwrap();
        mailbox.bomb(seat, 7).unwrap();
    }
    assert_eq!(mailbox.take(Tick(1)).len(), 4);
    assert_eq!(
        mailbox.take(Tick(16)),
        [
            SeatInput {
                seat: SeatId::Three,
                action: InputAction::Clear
            },
            SeatInput {
                seat: SeatId::Four,
                action: InputAction::Clear
            },
        ]
    );
    mailbox.equipment(SeatId::Four, 8).unwrap();
    mailbox.clear_pending();
    assert!(mailbox.take(Tick(17)).is_empty());
    assert_eq!(
        mailbox.equipment(SeatId::Four, 8),
        Err(InputError::DuplicateAction)
    );
}
