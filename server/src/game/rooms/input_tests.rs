use grenadiers_core::game::{Direction, InputAction, SeatId, SeatInput, Tick};

use super::*;

#[test]
fn direction_change_presses_new_before_releasing_old() {
    let mut mailbox = InputMailbox::default();
    mailbox
        .movement(SeatId::One, Some(Direction::East), Tick(0))
        .unwrap();
    mailbox.take(Tick(1));
    mailbox
        .movement(SeatId::One, Some(Direction::West), Tick(1))
        .unwrap();

    assert_eq!(
        mailbox.take(Tick(2)),
        vec![
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::West),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::East),
            },
        ]
    );
}

#[test]
fn movement_expires_before_fifteenth_later_update_and_boundary_refresh_wins() {
    let mut expired = InputMailbox::default();
    expired
        .movement(SeatId::One, Some(Direction::East), Tick(0))
        .unwrap();
    assert_eq!(expired.take(Tick(1)).len(), 1);
    for tick in 2..16 {
        assert!(expired.take(Tick(tick)).is_empty());
    }
    assert_eq!(
        expired.take(Tick(16)),
        vec![SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }]
    );

    let mut refreshed = InputMailbox::default();
    refreshed
        .movement(SeatId::One, Some(Direction::East), Tick(0))
        .unwrap();
    refreshed.take(Tick(1));
    for tick in 2..16 {
        refreshed.take(Tick(tick));
    }
    refreshed
        .movement(SeatId::One, Some(Direction::East), Tick(15))
        .unwrap();
    assert!(refreshed.take(Tick(16)).is_empty());
}

#[test]
fn bomb_intent_survives_clear_and_action_id_is_consumed() {
    let mut mailbox = InputMailbox::default();
    mailbox.bomb(SeatId::One, 7).unwrap();
    mailbox.movement(SeatId::One, None, Tick(0)).unwrap();
    assert_eq!(
        mailbox.take(Tick(1)),
        vec![SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }]
    );
    assert_eq!(
        mailbox.bomb(SeatId::One, 7),
        Err(InputError::DuplicateAction)
    );
}

#[test]
fn every_movement_action_precedes_canonical_bomb_intents() {
    let mut mailbox = InputMailbox::default();
    mailbox.bomb(SeatId::One, 1).unwrap();
    mailbox
        .movement(SeatId::Two, Some(Direction::West), Tick(0))
        .unwrap();

    assert_eq!(
        mailbox.take(Tick(1)),
        vec![
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::West),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
        ]
    );
}

#[test]
fn eight_bomb_actions_are_retained_and_drained_in_order() {
    let mut mailbox = InputMailbox::default();
    for action_id in 1..=u32::from(COMMANDS_PER_UPDATE) {
        mailbox.bomb(SeatId::One, action_id).unwrap();
    }

    assert_eq!(
        mailbox.take(Tick(1)),
        vec![
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            };
            usize::from(COMMANDS_PER_UPDATE)
        ]
    );
    assert_eq!(
        mailbox.last_manual_action(SeatId::One),
        Some(u32::from(COMMANDS_PER_UPDATE))
    );
}

#[test]
fn mixed_manual_actions_share_deduplication_budget_and_keep_per_seat_order() {
    let mut mailbox = InputMailbox::default();
    mailbox
        .movement(SeatId::Two, Some(Direction::West), Tick(0))
        .unwrap();
    mailbox.equipment(SeatId::One, 1).unwrap();
    mailbox.bomb(SeatId::One, 2).unwrap();
    mailbox.equipment(SeatId::One, 3).unwrap();
    assert_eq!(
        mailbox.bomb(SeatId::One, 3),
        Err(InputError::DuplicateAction)
    );

    assert_eq!(
        mailbox.take(Tick(1)),
        vec![
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::West),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::UseEquipment,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::UseEquipment,
            },
        ]
    );
    assert_eq!(mailbox.last_manual_action(SeatId::One), Some(3));
}

#[test]
fn duplicate_precedes_the_eight_command_limit_for_mixed_manuals() {
    let mut mailbox = InputMailbox::default();
    for action_id in 1..=u32::from(COMMANDS_PER_UPDATE) {
        if action_id % 2 == 0 {
            mailbox.equipment(SeatId::One, action_id).unwrap();
        } else {
            mailbox.bomb(SeatId::One, action_id).unwrap();
        }
    }
    assert_eq!(
        mailbox.bomb(SeatId::One, u32::from(COMMANDS_PER_UPDATE)),
        Err(InputError::DuplicateAction)
    );
    assert_eq!(
        mailbox.equipment(SeatId::One, u32::from(COMMANDS_PER_UPDATE) + 1),
        Err(InputError::RateLimited)
    );
    assert_eq!(
        mailbox.last_manual_action(SeatId::One),
        Some(u32::from(COMMANDS_PER_UPDATE))
    );
    assert_eq!(
        mailbox.take(Tick(1)).len(),
        usize::from(COMMANDS_PER_UPDATE)
    );
}

#[test]
fn clear_and_ttl_keep_manuals_while_a_fence_drops_them_without_resetting_ids() {
    let mut mailbox = InputMailbox::default();
    mailbox.equipment(SeatId::One, 4).unwrap();
    mailbox.movement(SeatId::One, None, Tick(0)).unwrap();
    assert_eq!(
        mailbox.take(Tick(1)),
        vec![SeatInput {
            seat: SeatId::One,
            action: InputAction::UseEquipment,
        }]
    );

    mailbox.equipment(SeatId::One, 5).unwrap();
    mailbox.stop(SeatId::One);
    assert_eq!(
        mailbox.take(Tick(2)),
        vec![SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }]
    );
    assert_eq!(
        mailbox.bomb(SeatId::One, 5),
        Err(InputError::DuplicateAction)
    );
}

#[test]
fn movement_and_manual_actions_share_budget_without_advancing_rejected_high_water() {
    let mut mailbox = InputMailbox::default();
    mailbox
        .movement(SeatId::One, Some(Direction::East), Tick(0))
        .unwrap();
    for action_id in 1..u32::from(COMMANDS_PER_UPDATE) {
        mailbox.bomb(SeatId::One, action_id).unwrap();
    }

    assert_eq!(
        mailbox.bomb(SeatId::One, u32::from(COMMANDS_PER_UPDATE)),
        Err(InputError::RateLimited)
    );
    assert_eq!(
        mailbox.last_manual_action(SeatId::One),
        Some(u32::from(COMMANDS_PER_UPDATE) - 1)
    );
    let drained = mailbox.take(Tick(1));
    assert!(matches!(
        drained.first(),
        Some(SeatInput {
            action: InputAction::Press(Direction::East),
            ..
        })
    ));
    assert_eq!(
        drained[1..]
            .iter()
            .filter(|input| input.action == InputAction::PlaceBomb)
            .count(),
        usize::from(COMMANDS_PER_UPDATE) - 1
    );
}

#[test]
fn movement_expiry_preserves_queued_manual_actions() {
    let mut mailbox = InputMailbox::default();
    mailbox
        .movement(SeatId::One, Some(Direction::East), Tick(0))
        .unwrap();
    mailbox.take(Tick(1));
    for tick in 2..16 {
        assert!(mailbox.take(Tick(tick)).is_empty());
    }
    mailbox.bomb(SeatId::One, 1).unwrap();

    assert_eq!(
        mailbox.take(Tick(16)),
        vec![
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
        ]
    );
}

#[test]
fn admitted_command_budget_is_independent_per_seat() {
    let mut mailbox = InputMailbox::default();
    for _ in 0..COMMANDS_PER_UPDATE {
        mailbox.movement(SeatId::One, None, Tick(0)).unwrap();
    }
    assert_eq!(
        mailbox.movement(SeatId::One, None, Tick(0)),
        Err(InputError::RateLimited)
    );
    mailbox.movement(SeatId::Two, None, Tick(0)).unwrap();
}

#[test]
fn stopping_one_seat_fences_controls_without_resetting_bomb_deduplication() {
    let mut mailbox = InputMailbox::default();
    mailbox
        .movement(SeatId::One, Some(Direction::East), Tick(0))
        .unwrap();
    mailbox.bomb(SeatId::One, 7).unwrap();
    mailbox
        .movement(SeatId::Two, Some(Direction::West), Tick(0))
        .unwrap();

    mailbox.stop(SeatId::One);

    assert_eq!(
        mailbox.take(Tick(1)),
        vec![
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::West),
            },
        ]
    );
    assert_eq!(
        mailbox.bomb(SeatId::One, 7),
        Err(InputError::DuplicateAction)
    );
    mailbox.bomb(SeatId::One, 8).unwrap();
}
