use std::{sync::Arc, time::Instant as StdInstant};

use grenadiers_core::{
    game::{EnabledItems, SimulationSeed, StormCloudConfig},
    protocol::{Counter, RoomCode, RoomId, v14},
    room::{
        CpuDifficulty, CpuProfile, RespawnLairTuning, RoomSettings, SpikePitTuning,
        SpikeRoomConfiguration,
    },
};
use tokio::time::Instant;

use super::{
    BindingKey, GameRooms, LiveAdmission, RoomError, RoomHandle,
    cpu::CpuController,
    live::LiveOwner,
    registry_lookup::RegistryOperation,
    roster_owner::PreparedRosterOwner,
    roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    roster_seed::SimulationRoot,
};

enum PreparedLiveOwner {
    Roster(Box<PreparedRosterOwner>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LairSettingsAction {
    pub(crate) settings_revision: u32,
    pub(crate) settings: RoomSettings,
    pub(crate) cloud: StormCloudConfig,
    pub(crate) tuning: RespawnLairTuning,
    pub(crate) spike_pits: SpikePitTuning,
}

impl GameRooms {
    pub(crate) async fn create_live(
        self: &Arc<Self>,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
        configuration: impl Into<SpikeRoomConfiguration>,
    ) -> Result<LiveAdmission, RoomError> {
        let configuration = configuration.into();
        validate_configuration(&configuration)?;
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

    pub(crate) async fn lair_settings_live(
        &self,
        admission: &LiveAdmission,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: LairSettingsAction,
    ) -> Result<(), RoomError> {
        self.handle_for(admission.room_id())?
            .owner
            .lair_settings(binding, room_id, round_id, action)
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
        configuration: SpikeRoomConfiguration,
        parts: &super::registry_identity::RegistryIdentityParts,
    ) -> Result<(LiveOwner, PreparedLiveOwner), RoomError> {
        self.prepare_roster_live_owner(configuration, parts)
    }

    fn prepare_roster_live_owner(
        &self,
        configuration: SpikeRoomConfiguration,
        parts: &super::registry_identity::RegistryIdentityParts,
    ) -> Result<(LiveOwner, PreparedLiveOwner), RoomError> {
        let base = *configuration.base().base();
        let cloud = configuration.base().storm_cloud();
        let tuning = RespawnLairTuning::try_from(configuration.base().lairs())
            .map_err(|_| RoomError::UnsupportedConfiguration)?;
        let spike_pits = configuration.spike_pits();
        let root = SimulationRoot::generate().map_err(|_| RoomError::CapacityReached)?;
        let simulation_seed = Arc::new(move |epoch| root.for_round(epoch));
        let identity = RosterIdentity {
            id: parts.id.clone(),
            code: parts.code.clone(),
            registry_serial: parts.serial,
            configuration: base,
        };
        let runtime = RosterRuntime::new_spike_v19_with_navigation(
            identity,
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
            cloud,
            tuning,
            spike_pits,
            self.round_seed_source.clone(),
            simulation_seed as Arc<dyn Fn(u64) -> Result<SimulationSeed, ()> + Send + Sync>,
            grenadiers_core::game::MobNavigationPolicy::TimedEscapeV3,
        )
        .map_err(|_| RoomError::CapacityReached)?;
        let profile = match base.cpu_difficulty() {
            Some(CpuDifficulty::Easy) => CpuProfile::CORNER_REFUGES_V56_EASY,
            Some(CpuDifficulty::Standard) => CpuProfile::CORNER_REFUGES_V56_STANDARD,
            Some(CpuDifficulty::Hard) => CpuProfile::HARD_V58,
            None => CpuProfile::CORNER_REFUGES_V56_STANDARD,
        };
        let mut cpus = Vec::new();
        for seat in base.cpu_seats() {
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

fn validate_configuration(configuration: &SpikeRoomConfiguration) -> Result<(), RoomError> {
    if configuration.base().base().human_seats().next().is_none() {
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
