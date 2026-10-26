/// Deterministic units available to one decision attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkBudget {
    limit: u32,
    spent: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkError {
    Exhausted,
}

impl WorkBudget {
    #[must_use]
    pub const fn new(limit: u32) -> Self {
        Self { limit, spent: 0 }
    }

    pub fn try_take(&mut self, units: u32) -> Result<(), WorkError> {
        let Some(spent) = self.spent.checked_add(units) else {
            return Err(WorkError::Exhausted);
        };
        if spent > self.limit {
            return Err(WorkError::Exhausted);
        }
        self.spent = spent;
        Ok(())
    }

    #[must_use]
    pub const fn limit(self) -> u32 {
        self.limit
    }

    #[must_use]
    pub const fn spent(self) -> u32 {
        self.spent
    }

    #[must_use]
    pub const fn remaining(self) -> u32 {
        self.limit - self.spent
    }

    #[must_use]
    pub const fn exhausted(self) -> bool {
        self.spent == self.limit
    }
}
