use std::{sync::Arc, time::Instant as StdInstant};

use grenadiers_core::{
    game::SimulationSeed,
    protocol::{Counter, RoomCode, RoomId, v14},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, RoomConfiguration, RoomMode, RoomSettings,
        SeatControl,
    },
};
use tokio::time::Instant;

use super::{
    BindingKey, GameRooms, LiveAdmission, RoomError, RoomHandle,
    cpu::CpuController,
    live::LiveOwner,
    match_owner::PreparedMatchOwner,
    registry_lookup::RegistryOperation,
    roster_owner::PreparedRosterOwner,
    roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    roster_seed::SimulationRoot,
    types::RoomIdentity,
};

enum PreparedLiveOwner {
    Legacy(Box<PreparedMatchOwner>),
    Roster(Box<PreparedRosterOwner>),
}

impl GameRooms {
    pub(crate) async fn create_live(
        self: &Arc<Self>,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
        configuration: RoomConfiguration,
    ) -> Result<LiveAdmission, RoomError> {
        validate_configuration(configuration)?;
        let (handle, prepared, id, code) = {
            let mut registry = self.lock();
            if self.is_stopping() {
                return Err(RoomError::RoomNotFound);
            }
            if !registry.operations.take_create(now) {
                return Err(RoomError::RateLimited);
            }
            if registry.rooms.len() >= super::MAX_ROOMS {
                return Err(RoomError::CapacityReached);
            }
            let parts = self.generate_identity_parts(&mut registry)?;
            let (owner, prepared) = self.prepare_live_owner(configuration, &parts)?;
            let id = parts.id.clone();
            let code = parts.code.clone();
            self.commit_identity(&mut registry, &parts);
            let handle = RoomHandle { owner };
            registry.codes.insert(code.clone(), id.clone());
            registry.rooms.insert(id.clone(), handle.clone());
            (handle, prepared, id, code)
        };

        let weak = Arc::downgrade(self);
        let exit_id = id.clone();
        let exit_code = code.clone();
        match prepared {
            PreparedLiveOwner::Legacy(owner) => drop((*owner).start(move || {
                if let Some(rooms) = weak.upgrade() {
                    rooms.cleanup(&exit_id, &exit_code);
                }
            })),
            PreparedLiveOwner::Roster(owner) => drop((*owner).start(move || {
                if let Some(rooms) = weak.upgrade() {
                    rooms.cleanup(&exit_id, &exit_code);
                }
            })),
        }

        let mut cleanup = CreationCleanup::new(self, id, code);
        let mut admission = handle.owner.admit(true, binding, sequence).await?;
        admission.acknowledge_notices();
        cleanup.disarm();
        Ok(admission)
    }

    pub(crate) async fn join_live(
        &self,
        code: &RoomCode,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
    ) -> Result<LiveAdmission, RoomError> {
        let handle = self.lookup_code(code, now, RegistryOperation::Join)?;
        let mut admission = handle.owner.admit(false, binding, sequence).await?;
        admission.acknowledge_notices();
        Ok(admission)
    }

    pub(crate) async fn resume_live(
        &self,
        room_id: RoomId,
        binding: BindingKey,
        sequence: Counter,
        token: v14::ResumeToken,
        now: StdInstant,
    ) -> Result<LiveAdmission, RoomError> {
        let handle = self.lookup_id(&room_id, now, RegistryOperation::Resume)?;
        let mut admission = handle
            .owner
            .resume(binding, sequence, room_id, token)
            .await?;
        admission.acknowledge_notices();
        Ok(admission)
    }

    pub(crate) async fn action_live(
        &self,
        admission: &LiveAdmission,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: RosterAction,
    ) -> Result<(), RoomError> {
        self.handle_for(admission.room_id())?
            .owner
            .action(binding, room_id, round_id, action)
            .await
    }

    pub(crate) async fn leave_live(
        &self,
        admission: &LiveAdmission,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
    ) -> Result<(), RoomError> {
        self.handle_for(admission.room_id())?
            .owner
            .leave(binding, room_id, round_id)
            .await
    }

    fn prepare_live_owner(
        &self,
        configuration: RoomConfiguration,
        parts: &super::registry_identity::RegistryIdentityParts,
    ) -> Result<(LiveOwner, PreparedLiveOwner), RoomError> {
        if configuration.player_count().get() == 2 {
            return self.prepare_legacy_live_owner(configuration, parts);
        }
        self.prepare_roster_live_owner(configuration, parts)
    }

    fn prepare_legacy_live_owner(
        &self,
        configuration: RoomConfiguration,
        parts: &super::registry_identity::RegistryIdentityParts,
    ) -> Result<(LiveOwner, PreparedLiveOwner), RoomError> {
        let (mode, private_cpu_seed) = match configuration.controllers() {
            [SeatControl::Human, SeatControl::Human] => (RoomMode::HumanDuel, None),
            [SeatControl::Human, SeatControl::Cpu] => {
                let profile = match configuration.cpu_difficulty() {
                    Some(CpuDifficulty::Easy) => CpuProfile::CURRENT_EASY,
                    Some(CpuDifficulty::Standard) => CpuProfile::CURRENT,
                    None => return Err(RoomError::UnsupportedConfiguration),
                };
                let mut seed = [0; 32];
                (self.random_fill)(&mut seed).map_err(|_| RoomError::CapacityReached)?;
                (RoomMode::Solo(profile), Some(seed))
            }
            _ => return Err(RoomError::UnsupportedConfiguration),
        };
        let identity = RoomIdentity {
            id: parts.id.clone(),
            code: parts.code.clone(),
            profile: self.profile,
            mode,
            registry_serial: parts.serial,
            private_cpu_seed,
        };
        let (handle, owner) = PreparedMatchOwner::new_live14_with_planner(
            identity,
            Instant::now(),
            self.round_seed_source.clone(),
            Arc::clone(&self.random_fill),
            self.cpu_planner.clone(),
            self.shutdown(),
        )
        .map_err(|_| RoomError::CapacityReached)?;
        Ok((
            LiveOwner::Legacy(handle),
            PreparedLiveOwner::Legacy(Box::new(owner)),
        ))
    }

    fn prepare_roster_live_owner(
        &self,
        configuration: RoomConfiguration,
        parts: &super::registry_identity::RegistryIdentityParts,
    ) -> Result<(LiveOwner, PreparedLiveOwner), RoomError> {
        let root = SimulationRoot::generate().map_err(|_| RoomError::CapacityReached)?;
        let simulation_seed = Arc::new(move |epoch| root.for_round(epoch));
        let identity = RosterIdentity {
            id: parts.id.clone(),
            code: parts.code.clone(),
            registry_serial: parts.serial,
            configuration,
        };
        let runtime = RosterRuntime::new(
            identity,
            RoomSettings::DEFAULT,
            self.round_seed_source.clone(),
            simulation_seed as Arc<dyn Fn(u64) -> Result<SimulationSeed, ()> + Send + Sync>,
        )
        .map_err(|_| RoomError::CapacityReached)?;
        let profile = match configuration.cpu_difficulty() {
            Some(CpuDifficulty::Easy) => CpuProfile::MULTIPLAYER_EASY,
            Some(CpuDifficulty::Standard) => CpuProfile::MULTIPLAYER_STANDARD,
            None => CpuProfile::MULTIPLAYER_STANDARD,
        };
        let mut cpus = Vec::new();
        for seat in configuration.cpu_seats() {
            let mut seed = [0; 32];
            (self.random_fill)(&mut seed).map_err(|_| RoomError::CapacityReached)?;
            cpus.push(CpuController::new(
                self.cpu_planner.clone(),
                seat,
                parts.serial,
                seed,
                profile,
            ));
        }
        let (handle, owner) = PreparedRosterOwner::new(
            runtime,
            cpus,
            Instant::now(),
            Arc::clone(&self.random_fill),
            self.shutdown(),
        )
        .map_err(|_| RoomError::CapacityReached)?;
        Ok((
            LiveOwner::Roster(handle),
            PreparedLiveOwner::Roster(Box::new(owner)),
        ))
    }
}

fn validate_configuration(configuration: RoomConfiguration) -> Result<(), RoomError> {
    if configuration.board() != BoardSize::FifteenByThirteen
        || configuration.human_seats().next().is_none()
    {
        return Err(RoomError::UnsupportedConfiguration);
    }
    Ok(())
}

struct CreationCleanup<'a> {
    rooms: &'a GameRooms,
    id: RoomId,
    code: RoomCode,
    armed: bool,
}

impl<'a> CreationCleanup<'a> {
    fn new(rooms: &'a GameRooms, id: RoomId, code: RoomCode) -> Self {
        Self {
            rooms,
            id,
            code,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for CreationCleanup<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.rooms.cleanup(&self.id, &self.code);
        }
    }
}
