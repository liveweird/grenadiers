use super::{
    Actor, ActorInit, LairState, PlayerCount, RoundAssignment, RulesConfig, State, StateError,
    StormCloudState, Tick, World, contacts, movement, random, validation,
};
use std::collections::BTreeSet;

impl State {
    pub(super) fn new_at_tick_with_assignment_and_randomness(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<ActorInit>>,
        tick: Tick,
        assignment: RoundAssignment,
        randomness: Option<Box<random::SimulationRandomness>>,
    ) -> Result<Self, StateError> {
        let mut actors = actors.into();
        let player_count = PlayerCount::try_from(actors.len())
            .map_err(|()| StateError::InvalidPlayerCount(actors.len()))?;
        if world.arena.spawns().len() != actors.len() {
            return Err(StateError::SpawnCountMismatch {
                actors: actors.len(),
                spawns: world.arena.spawns().len(),
            });
        }
        if rules.version() < 6 && player_count != PlayerCount::Two {
            return Err(StateError::UnsupportedPlayerCount {
                version: rules.version(),
                count: player_count,
            });
        }
        if rules.version() < 6 && world.arena.board() != crate::room::BoardSize::FifteenByThirteen {
            return Err(StateError::UnsupportedBoardSize {
                version: rules.version(),
                board: world.arena.board(),
            });
        }
        let enabled = rules.enabled_items();
        let cloud_enabled = matches!(
            rules.storm_cloud_config(),
            crate::game::rules::StormCloudConfig::Single
        );
        if rules.version() >= 6
            && (enabled.contains_teleporter() || cloud_enabled)
            && randomness.is_none()
        {
            return Err(StateError::MissingSimulationRandomness);
        }
        let mut ids = BTreeSet::new();
        for actor in &actors {
            if !ids.insert(actor.id) {
                return Err(StateError::DuplicateActorId(actor.id));
            }
        }
        if let Some(enabled) = validation::unsupported_enabled_items(rules) {
            return Err(StateError::UnsupportedEnabledItems(enabled));
        }
        for (cell, pickup) in world.pickups.iter() {
            if !pickup.enabled_by(rules.enabled_items()) {
                return Err(StateError::UnsupportedPickup { cell, kind: pickup });
            }
        }
        if world.oil.iter().next().is_some() && !rules.enabled_items().contains_oil_barrel() {
            return Err(StateError::UnsupportedOilBarrels);
        }
        if let Some(cell) = validation::invalid_oil(&world, tick) {
            return Err(StateError::InvalidOilBarrel(cell));
        }
        let mut seats = BTreeSet::new();
        for actor in &actors {
            if !seats.insert(actor.seat) {
                return Err(StateError::DuplicateSeat(actor.seat));
            }
        }
        actors.sort_by_key(|actor| actor.seat.index());
        if actors
            .iter()
            .map(|actor| actor.seat)
            .ne(player_count.seats().iter().copied())
        {
            return Err(StateError::NonCanonicalSeats);
        }
        for (&key, bomb) in &world.bombs {
            if key != bomb.id() {
                return Err(StateError::BombIdMismatch { key, id: bomb.id() });
            }
        }
        if let Some(id) = validation::unsupported_trigger(rules, &world) {
            return Err(StateError::UnsupportedBombTrigger(id));
        }
        if let Some(id) = validation::unsupported_blast(rules, &world) {
            return Err(StateError::UnsupportedBombBlast(id));
        }
        if let Some(owner) = validation::duplicate_remote_owner(&world) {
            return Err(StateError::DuplicateRemoteOwner(owner));
        }
        for actor in &actors {
            if world.bombs.contains_key(&actor.id) {
                return Err(StateError::DuplicateEntityId(actor.id));
            }
            if !movement::position_is_free(
                &world,
                actor.position,
                &BTreeSet::new(),
                false,
                &BTreeSet::new(),
                &BTreeSet::new(),
            ) {
                return Err(StateError::IllegalActorPosition(actor.id));
            }
        }
        if rules.version() >= 9 {
            let positions = actors
                .iter()
                .map(|actor| (actor.seat, actor.position))
                .collect::<Vec<_>>();
            if let Some((first, second)) =
                crate::game::player_collision::first_player_position_conflict(&positions)
            {
                return Err(StateError::ConflictingActorPositions { first, second });
            }
        }
        for bomb in world.bombs.values() {
            if !actors.iter().any(|actor| actor.id == bomb.owner()) {
                return Err(StateError::UnknownBombOwner(bomb.owner()));
            }
        }
        if let Some(id) = validation::unsupported_kick_history(rules, &world) {
            return Err(StateError::UnsupportedBombKickHistory(id));
        }
        if let Some(id) = validation::unsupported_punch_history(rules, &world) {
            return Err(StateError::UnsupportedBombPunchHistory(id));
        }

        let next_entity_id = world
            .bombs
            .keys()
            .next_back()
            .map_or(0, |id| u64::from(id.0) + 1);
        let mut initialized: Vec<_> = actors
            .into_iter()
            .map(|actor| Actor::new(actor, rules))
            .collect();
        contacts::initialize_grace(&world, &mut initialized);
        let storm_cloud = cloud_enabled.then(|| {
            let seed = randomness
                .as_ref()
                .expect("cloud randomness was validated")
                .seed();
            Box::new(StormCloudState::new(
                seed,
                world.arena.width(),
                world.arena.height(),
            ))
        });
        let lairs = LairState::new(
            rules.lair_config(),
            rules.version(),
            rules.mob_navigation_policy(),
            world.arena.board(),
            &world.arena,
            tick,
        )?;
        Ok(Self {
            rules,
            world,
            actors: initialized,
            player_count,
            tick,
            next_entity_id,
            running_updates: 0,
            result: None,
            assignment,
            last_teleport_tick: None,
            construction_tick: tick,
            randomness: (rules.version() >= 6).then_some(randomness).flatten(),
            storm_cloud,
            lairs,
        })
    }
}
