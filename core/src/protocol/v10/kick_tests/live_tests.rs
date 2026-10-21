use super::*;

#[test]
fn live_capture_binding_and_admission_keep_active_motion_private() {
    let state = timed_state(false);
    let capture = capture_expanded(&state, &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    let kick = capture.world.0.kick_bombs[&bomb.id];
    assert_eq!(kick.bomb, bomb);
    assert!(kick.has_been_kicked);
    assert_eq!(kick.motion.unwrap().countdown, 0);
    assert_eq!(
        kick.motion.unwrap().direction,
        crate::protocol::Direction::East
    );

    let room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let generation = game::generate_expanded_arena([7; 32], EnabledItems::KICK).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    let mut frames = prepare_snapshot_pair(&publication).unwrap().to_vec();
    frames.push(
        prepare_expanded_admission(
            &room,
            Some(room.round_id),
            Some(&capture),
            Some(generation),
            Seat::One,
            None,
            room.revision,
            ResumeToken::new("a".repeat(64)).unwrap(),
        )
        .unwrap(),
    );
    for frame in frames {
        for private in ["motion", "countdown", "hasBeenKicked", "direction"] {
            assert!(!frame.contains(private));
        }
    }
}

#[test]
fn malformed_live_private_facts_reject_capture_revalidation_and_binding() {
    let state = timed_state(false);
    let capture = capture_expanded(&state, &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    let mut malformed = capture.world.0.clone();
    malformed
        .kick_bombs
        .get_mut(&bomb.id)
        .unwrap()
        .motion
        .as_mut()
        .unwrap()
        .countdown = 10;
    malformed.enabled_items = ItemSet::new([ItemId::Kick, ItemId::InvisibleBombs]).unwrap();
    malformed.actors[1].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibleBombs, 1).unwrap(),
    ])
    .unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            malformed.clone(),
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidBombs)
    );

    let mut rebound = capture;
    rebound.world.0 = malformed;
    let room = room(rebound.world.0.enabled_items, rebound.world.0.physical_tick);
    let generation = game::generate_expanded_arena([7; 32], EnabledItems::KICK).record();
    assert_eq!(
        bind_expanded(&room, room.round_id, &rebound, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn private_fact_association_history_and_contact_exclusion_are_validated() {
    let capture = capture_expanded(&timed_state(false), &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    for mutation in 0..3 {
        let mut facts = capture.world.0.clone();
        let kick = facts.kick_bombs.get_mut(&bomb.id).unwrap();
        match mutation {
            0 => kick.bomb.cell = Cell::new(6, 5).unwrap(),
            1 => kick.bomb.owner = crate::protocol::EntityId::new(2),
            _ => kick.has_been_kicked = false,
        }
        assert!(
            crate::protocol::v8::projection_validation::validate_world_for_items(
                facts,
                super::expanded_items(),
            )
            .is_err()
        );
    }

    let mut contact = capture.world.0;
    contact.timed_bombs.clear();
    contact.bombs[0].trigger = BombTrigger::contact();
    contact.kick_bombs.get_mut(&bomb.id).unwrap().bomb.trigger = BombTrigger::contact();
    contact.enabled_items = ItemSet::new([ItemId::Kick, ItemId::Landmine]).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            contact,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidBombs)
    );
}
