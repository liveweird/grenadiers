use grenadiers_core::cpu::{Observation, WorkBudget};
use grenadiers_core::game::Tick;

use super::CpuController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ContactStatus {
    Continue,
    Invalidated,
}

impl CpuController {
    #[cfg(test)]
    pub(in crate::game::rooms) const fn has_committed_contact(&self) -> bool {
        self.committed_contact.is_some()
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn latest_observation_for_test(&self) -> Option<Observation> {
        self.latest_observation.clone()
    }

    pub(in crate::game::rooms) fn validate_contact(
        &mut self,
        observation: &Observation,
        current_tick: Tick,
    ) -> Option<ContactStatus> {
        let contact = self.committed_contact.as_ref()?;
        let valid = current_tick
            .0
            .checked_add(1)
            .is_some_and(|next| next < contact.safe_until().0)
            && observation.parts().tick <= current_tick
            && contact.observation_fits(observation, &mut WorkBudget::new(65_536));
        if valid {
            return Some(ContactStatus::Continue);
        }
        // Contact invalidation revokes movement, but not independently proven
        // origin-exit grace. Fresh planning must certify both hazards again.
        let origin = self
            .committed_route
            .as_ref()
            .and_then(|route| route.escape_origin(observation))
            .or_else(|| {
                self.escape_origin.as_ref().and_then(|origin| {
                    self.committed_pulse.as_ref().map_or_else(
                        || origin.retain_stopped(observation),
                        |pulse| {
                            pulse.retain_escape_origin(
                                origin,
                                observation,
                                observation.parts().tick,
                            )
                        },
                    )
                })
            });
        let origin_epoch = origin
            .as_ref()
            .and(self.committed_route_epoch.or(self.escape_origin_epoch));
        self.committed_contact = None;
        self.committed_pulse = None;
        self.invalidate_route();
        self.escaping = false;
        self.stop_admission = None;
        self.settled_stop_tick = None;
        self.escape_origin = origin;
        self.escape_origin_epoch = origin_epoch;
        self.cancel();
        Some(ContactStatus::Invalidated)
    }
}

#[cfg(test)]
mod tests {
    use grenadiers_core::{
        cpu::{
            CpuRng, MemoryKey, ObservedEnvironment, ObservedLairs, WorkBudget,
            forecast::ObservedHazardMemory, policy::UtilityMemory, survival::plan_lairs,
        },
        game::{LairView, Tick, lair_layout_v1},
        room::{BoardSize, LairConfig, LairSettings},
    };

    use super::*;

    #[test]
    fn invalid_contact_revokes_a_stopped_waiting_route_before_it_can_submit() {
        let (mut controller, _) = super::super::route_tests::committed_wait_controller();
        let observation = rules8_observation();
        let key = MemoryKey {
            world: observation.parts().world,
            cpu_seat: observation.parts().recipient,
        };
        let planned = plan_lairs(
            &observation,
            &UtilityMemory::new(key),
            &ObservedHazardMemory::new(key),
            observation.parts().tick,
            &mut CpuRng::from_seed(key, [31; 32]),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        controller.committed_contact = planned.contact;
        assert!(controller.has_committed_route());
        assert!(controller.settled_stop_tick.is_some());

        assert_eq!(
            controller.validate_contact(&observation, Tick(u64::MAX - 1)),
            Some(ContactStatus::Invalidated)
        );
        assert!(!controller.has_committed_route());
        assert!(controller.settled_stop_tick.is_none());
        assert_eq!(controller.poll_route_command(Tick(53), 4), None);
    }

    fn rules8_observation() -> grenadiers_core::cpu::Observation {
        let base = crate::game::rooms::cpu::tests::observation();
        let environment = ObservedEnvironment::new(
            grenadiers_core::game::StormCloudConfig::Disabled,
            None,
            vec![None, Some(0)],
        );
        let mut parts = base.into_parts();
        parts.rules.version = 8;
        let board = BoardSize::FifteenByThirteen;
        let cells = lair_layout_v1(board);
        for cell in cells {
            let index =
                usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
            parts.arena.terrain[index] = grenadiers_core::game::Terrain::Wall;
        }
        let lairs = cells
            .into_iter()
            .map(|cell| LairView {
                cell,
                next_spawn_at: Tick(300),
                living_mobs: 0,
            })
            .collect();
        grenadiers_core::cpu::Observation::new_rules_v8(
            parts,
            environment,
            ObservedLairs::new(
                LairConfig::Enabled(LairSettings::new(5, 3).unwrap()),
                lairs,
                vec![],
            ),
        )
        .unwrap()
    }
}
