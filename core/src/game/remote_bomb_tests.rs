use std::collections::BTreeMap;

use crate::game::{
    ActorInit, BlastKind, Bomb, BombTrigger, Direction, EffectKind, EnabledItems, EntityId,
    InputAction, Pickup, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon,
    SpeedLevel, State, StateError, StepError, StepEventKind, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn state(enabled: EnabledItems, pickup: Option<PickupKind>, tick: Tick) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns().to_vec();
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            pickup
                .map(|kind| BTreeMap::from([(spawns[0], kind)]))
                .unwrap_or_default(),
        )
        .unwrap(),
        SeatId::DUEL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        tick,
    )
    .unwrap()
}

fn bomb_action() -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }]
}

fn arm_remote(game: &mut State) {
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Remote);
}

#[test]
fn remote_and_power_replace_each_other_in_the_single_weapon_slot() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::POWER_BOMB.bits())
            .unwrap();
    let mut game = state(enabled, Some(PickupKind::RemoteBomb), Tick(0));
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Remote)
    );

    let cell = game.actors[0].position.cell().unwrap();
    game.world
        .pickups
        .exposed
        .insert(cell, Pickup::new(PickupKind::PowerBomb));
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );

    game.world
        .pickups
        .exposed
        .insert(cell, Pickup::new(PickupKind::RemoteBomb));
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Remote)
    );
}

fn actors() -> [ActorInit; 2] {
    let arena = fixed_arena();
    let spawns = arena.spawns().to_vec();
    SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 10),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    })
}

fn remote(id: u32, owner: EntityId, cell: crate::game::CellId) -> Bomb {
    Bomb::new(EntityId(id), cell, owner, 2, Tick(0), BombTrigger::Remote).unwrap()
}

#[test]
fn constructor_and_step_reject_duplicate_live_remote_owners_atomically() {
    let rules = RulesConfig::expanded_v5(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::REMOTE_BOMB),
    );
    let mut world = World::new(fixed_arena());
    world.bombs.insert(
        EntityId(30),
        remote(30, EntityId(10), crate::game::CellId::new(5, 5)),
    );
    world.bombs.insert(
        EntityId(31),
        remote(31, EntityId(10), crate::game::CellId::new(7, 5)),
    );
    assert_eq!(
        State::new(rules, world, actors()),
        Err(StateError::DuplicateRemoteOwner(EntityId(10)))
    );

    let mut game = state(EnabledItems::REMOTE_BOMB, None, Tick(0));
    game.insert_bomb(remote(
        30,
        game.actors[0].id,
        crate::game::CellId::new(5, 5),
    ));
    game.insert_bomb(remote(
        31,
        game.actors[0].id,
        crate::game::CellId::new(7, 5),
    ));
    let before = game.clone();
    assert_eq!(
        game.step(&[]),
        Err(StepError::DuplicateRemoteOwner(EntityId(10)))
    );
    assert_eq!(game, before);
}

#[test]
fn remote_requires_its_item_and_can_only_have_a_standard_blast() {
    let mut disabled = World::new(fixed_arena());
    disabled.bombs.insert(
        EntityId(30),
        remote(30, EntityId(10), crate::game::CellId::new(5, 5)),
    );
    assert_eq!(
        State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT),
            disabled,
            actors()
        ),
        Err(StateError::UnsupportedBombTrigger(EntityId(30)))
    );

    let enabled =
        EnabledItems::from_bits(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::POWER_BOMB.bits())
            .unwrap();
    let mut special = World::new(fixed_arena());
    let bomb = Bomb::new_with_blast(
        EntityId(30),
        crate::game::CellId::new(5, 5),
        EntityId(10),
        2,
        Tick(0),
        BombTrigger::Remote,
        BlastKind::Power,
    )
    .unwrap();
    special.bombs.insert(bomb.id(), bomb);
    assert_eq!(
        State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
            special,
            actors(),
        ),
        Err(StateError::UnsupportedBombBlast(EntityId(30)))
    );
}

#[test]
fn charge_places_one_immutable_remote_then_next_manual_action_detonates_same_id() {
    let mut game = state(
        EnabledItems::REMOTE_BOMB,
        Some(PickupKind::RemoteBomb),
        Tick(0),
    );
    game.step(&[]).unwrap();
    game.step(&bomb_action()).unwrap();
    let bomb = *game.world.bombs.values().next().unwrap();
    assert_eq!(bomb.trigger(), BombTrigger::Remote);
    assert_eq!(bomb.blast(), BlastKind::Standard);
    assert_eq!(bomb.range(), 2);
    assert_eq!(game.actors[0].inventory.weapon(), None);
    let next_id = game.next_entity_id;

    game.actors[0].position = Position::center(crate::game::CellId::new(5, 5));
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Power);
    let report = game.step(&bomb_action()).unwrap();
    assert!(game.world.bombs.is_empty());
    assert_eq!(game.next_entity_id, next_id);
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::BombDetonated(view) if view.id == bomb.id()
    )));
    assert!(
        !report
            .events
            .iter()
            .any(|event| matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
}

#[test]
fn exhausted_allocator_full_capacity_and_fuse_overflow_cannot_block_remote_trigger() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::REMOTE_BOMB.bits()
            | EnabledItems::POWER_BOMB.bits()
            | EnabledItems::PIERCING_BOMB.bits()
            | EnabledItems::LONG_FUSE.bits(),
    )
    .unwrap();
    for held_weapon in [SpecialWeapon::Power, SpecialWeapon::Piercing] {
        let mut game = state(enabled, None, Tick(u64::MAX - 1));
        let remote = Bomb::new(
            EntityId(30),
            crate::game::CellId::new(5, 5),
            game.actors[0].id,
            2,
            Tick(10),
            BombTrigger::Remote,
        )
        .unwrap();
        game.insert_bomb(remote);
        game.next_entity_id = u64::from(u32::MAX) + 1;
        game.actors[0].inventory.replace_weapon(held_weapon);
        game.actors[0]
            .effects
            .apply(EffectKind::LongFuse, Tick(u64::MAX - 601))
            .unwrap();
        assert_eq!(
            game.live_bomb_count(SeatId::One),
            usize::from(game.actors[0].capacity)
        );

        let report = game.step(&bomb_action()).unwrap();

        assert!(!game.world.bombs.contains_key(&remote.id()));
        assert_eq!(game.next_entity_id, u64::from(u32::MAX) + 1);
        assert_eq!(game.actors[0].inventory.weapon(), Some(held_weapon));
        assert!(report.events.iter().any(|event| matches!(
            event.kind,
            StepEventKind::BombDetonated(view) if view.id == remote.id()
        )));
        assert!(
            !report
                .events
                .iter()
                .any(|event| matches!(event.kind, StepEventKind::BombPlaced(_)))
        );
    }
}

#[test]
fn additional_same_tick_manual_actions_cannot_trigger_a_new_remote() {
    let mut game = state(EnabledItems::REMOTE_BOMB, None, Tick(0));
    arm_remote(&mut game);
    let report = game
        .step(&[bomb_action()[0], bomb_action()[0], bomb_action()[0]])
        .unwrap();
    assert_eq!(game.world.bombs.len(), 1);
    assert_eq!(
        game.world.bombs.values().next().unwrap().trigger(),
        BombTrigger::Remote
    );
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(event.kind, StepEventKind::BombPlaced(_)))
            .count(),
        1
    );
    assert!(
        !report
            .events
            .iter()
            .any(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
    );
}

#[test]
fn remote_placement_bypasses_fuse_math_at_the_tick_horizon() {
    let mut game = state(EnabledItems::REMOTE_BOMB, None, Tick(u64::MAX - 1));
    arm_remote(&mut game);
    game.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(u64::MAX - 601))
        .unwrap();
    game.step(&bomb_action()).unwrap();
    let bomb = game.world.bombs.values().next().unwrap();
    assert_eq!(bomb.placed_at(), Tick(u64::MAX));
    assert_eq!(bomb.trigger(), BombTrigger::Remote);
}

#[test]
fn automatic_drops_never_trigger_or_consume_remote_weapons() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut game = state(enabled, None, Tick(0));
    arm_remote(&mut game);
    game.actors[0]
        .effects
        .apply(EffectKind::Diarrhea, Tick(0))
        .unwrap();
    game.step(&[]).unwrap();
    let bomb = game.world.bombs.values().next().unwrap();
    assert!(matches!(bomb.trigger(), BombTrigger::Timed { .. }));
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Remote)
    );
}

#[test]
fn existing_fire_death_suppresses_the_retained_manual_trigger() {
    let mut game = state(EnabledItems::REMOTE_BOMB, None, Tick(0));
    let owner = game.actors[0].id;
    let remote = Bomb::new(
        EntityId(30),
        crate::game::CellId::new(5, 5),
        owner,
        1,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    game.insert_bomb(remote);
    let actor_cell = game.actors[0].position.cell().unwrap();
    game.world.hazards.insert(actor_cell, Tick(10));
    let report = game.step(&bomb_action()).unwrap();
    assert!(!game.actors[0].alive);
    assert!(game.world.bombs.contains_key(&remote.id()));
    assert!(
        !report
            .events
            .iter()
            .any(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
    );
}

#[test]
fn pickup_overflow_rolls_back_remote_detonation_and_every_step_mutation() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::SHIELD.bits())
            .unwrap();
    let mut game = state(enabled, Some(PickupKind::Shield), Tick(u64::MAX - 300));
    let remote = Bomb::new(
        EntityId(30),
        crate::game::CellId::new(7, 5),
        game.actors[0].id,
        1,
        game.tick,
        BombTrigger::Remote,
    )
    .unwrap();
    game.insert_bomb(remote);
    let before = game.clone();
    assert_eq!(
        game.step(&bomb_action()),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(game, before);
}
