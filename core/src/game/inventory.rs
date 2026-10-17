/// Replaceable equipment that stays equipped until replaced, consumed, death, or round reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Equipment {
    Kick,
    Punch,
    Bracelet,
    WallPass,
    Heart,
}

/// A single-use modifier for the next successful manual bomb placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialWeapon {
    Remote,
    Piercing,
    Power,
    Landmine,
}

/// The two independent, replaceable item slots owned by one actor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    equipment: Option<Equipment>,
    weapon: Option<SpecialWeapon>,
}

impl Inventory {
    #[must_use]
    pub const fn equipment(self) -> Option<Equipment> {
        self.equipment
    }

    #[must_use]
    pub const fn weapon(self) -> Option<SpecialWeapon> {
        self.weapon
    }

    #[must_use]
    pub const fn can_kick(self) -> bool {
        matches!(self.equipment, Some(Equipment::Kick))
    }

    #[must_use]
    pub const fn can_punch(self) -> bool {
        matches!(self.equipment, Some(Equipment::Punch))
    }

    #[must_use]
    pub const fn can_shove(self) -> bool {
        matches!(self.equipment, Some(Equipment::Bracelet))
    }

    #[must_use]
    pub const fn can_pass_soft_walls(self) -> bool {
        matches!(self.equipment, Some(Equipment::WallPass))
    }

    pub fn replace_equipment(&mut self, equipment: Equipment) -> Option<Equipment> {
        self.equipment.replace(equipment)
    }

    pub fn replace_weapon(&mut self, weapon: SpecialWeapon) -> Option<SpecialWeapon> {
        self.weapon.replace(weapon)
    }

    pub(crate) fn consume_heart(&mut self) -> bool {
        if self.equipment == Some(Equipment::Heart) {
            self.equipment = None;
            true
        } else {
            false
        }
    }

    pub fn consume_weapon(&mut self) -> Option<SpecialWeapon> {
        self.weapon.take()
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::{Equipment, Inventory, SpecialWeapon};

    #[test]
    fn slots_replace_independently_and_capabilities_follow_equipment() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.replace_equipment(Equipment::Kick), None);
        assert!(inventory.can_kick());
        assert_eq!(inventory.replace_weapon(SpecialWeapon::Piercing), None);
        assert_eq!(
            inventory.replace_equipment(Equipment::WallPass),
            Some(Equipment::Kick)
        );
        assert!(!inventory.can_kick());
        assert!(inventory.can_pass_soft_walls());
        assert_eq!(inventory.weapon(), Some(SpecialWeapon::Piercing));
    }

    #[test]
    fn only_heart_is_consumed_and_clear_resets_both_slots() {
        let mut inventory = Inventory::default();
        inventory.replace_equipment(Equipment::Punch);
        inventory.replace_weapon(SpecialWeapon::Remote);
        assert!(!inventory.consume_heart());
        inventory.replace_equipment(Equipment::Heart);
        assert!(inventory.consume_heart());
        assert_eq!(inventory.equipment(), None);
        assert_eq!(inventory.weapon(), Some(SpecialWeapon::Remote));
        assert_eq!(inventory.consume_weapon(), Some(SpecialWeapon::Remote));
        assert_eq!(inventory.consume_weapon(), None);
        inventory.clear();
        assert_eq!(inventory, Inventory::default());
    }
}
