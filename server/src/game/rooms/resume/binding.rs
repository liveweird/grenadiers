use super::super::types::BindingKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct BindingFence {
    current: BindingKey,
}

impl BindingFence {
    pub(in crate::game::rooms) const fn new(current: BindingKey) -> Self {
        Self { current }
    }

    pub(in crate::game::rooms) const fn current(self) -> BindingKey {
        self.current
    }

    pub(in crate::game::rooms) fn matches(self, candidate: BindingKey) -> bool {
        self.current == candidate
    }

    pub(in crate::game::rooms) fn replace_if_current(
        &mut self,
        expected: BindingKey,
        replacement: BindingKey,
    ) -> bool {
        if !self.matches(expected) {
            return false;
        }
        self.current = replacement;
        true
    }
}
