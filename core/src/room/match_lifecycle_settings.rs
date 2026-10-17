use super::*;

impl MatchLifecycle {
    #[must_use]
    pub const fn settings(&self) -> RoomSettings {
        self.settings
    }

    pub(super) fn set_settings_inner(
        &mut self,
        settings: RoomSettings,
    ) -> Result<MatchChange, MatchError> {
        if self.phase != MatchPhase::Waiting {
            return Err(MatchError::InvalidPhase);
        }
        if self.settings == settings {
            return Ok(MatchChange::Unchanged);
        }
        self.series.configure_unstarted(settings)?;
        self.settings = settings;
        self.clear_readiness();
        Ok(MatchChange::Changed)
    }
}
