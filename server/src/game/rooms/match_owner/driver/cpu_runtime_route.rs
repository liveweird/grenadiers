use grenadiers_core::{
    cpu::{MovementIntent, survival::RouteStatus},
    game::SeatId,
    room::MatchPhase,
};

use super::{Driver, fatal};
use crate::game::rooms::match_runtime::{MatchAction, MatchRuntimeError};

impl Driver {
    pub(super) fn apply_cpu_route(&mut self) -> Result<(), MatchRuntimeError> {
        if self.runtime.lifecycle().phase() != MatchPhase::Running {
            return Ok(());
        }
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        let Some(status) = self
            .cpu
            .as_mut()
            .and_then(|cpu| cpu.poll_route_command(tick, epoch))
        else {
            return Ok(());
        };
        let RouteStatus::Submit(command) = status else {
            if status == RouteStatus::Invalidated {
                self.fallback_cpu_stop()?;
            }
            return Ok(());
        };
        debug_assert_eq!(command.admission_tick(), tick);
        let movement = match command.movement() {
            MovementIntent::Direction(direction) => Some(direction),
            MovementIntent::Stop => None,
        };
        match self
            .runtime
            .action(SeatId::Two, epoch, MatchAction::Movement(movement))
        {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => {
                self.cpu
                    .as_mut()
                    .expect("route command has controller")
                    .invalidate_route();
                self.fallback_cpu_stop()?;
                return Ok(());
            }
        }
        self.cpu_stationary = movement.is_none();
        self.cpu_stop_at = None;
        let cpu = self.cpu.as_mut().expect("route command has controller");
        if movement.is_some() {
            cpu.on_movement_admitted();
        } else {
            cpu.on_stop_admitted(tick);
        }
        if !cpu.commit_route_command(command) {
            cpu.invalidate_route();
            self.fallback_cpu_stop()?;
        }
        Ok(())
    }

    fn fallback_cpu_stop(&mut self) -> Result<(), MatchRuntimeError> {
        self.cpu_stop_at = Some(self.runtime.lifecycle().tick());
        self.apply_cpu_stop()
    }
}
