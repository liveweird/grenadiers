use crate::cpu::VisibleActor;
use crate::game::{Equipment, PickupKind, SpecialWeapon};

pub(super) fn classify(actor: &VisibleActor, kind: PickupKind) -> Option<(Option<u8>, bool)> {
    let incoming = match kind {
        PickupKind::Kick => Equipment::Kick,
        PickupKind::Punch => Equipment::Punch,
        PickupKind::Bracelet => Equipment::Bracelet,
        PickupKind::WallPass => Equipment::WallPass,
        PickupKind::Heart => Equipment::Heart,
        _ => return None,
    };
    Some(match (actor.equipment, incoming) {
        (Some(Equipment::WallPass), Equipment::WallPass) => (None, true),
        (Some(Equipment::WallPass), _) => (None, false),
        (Some(Equipment::Heart), Equipment::WallPass) => (None, false),
        (
            None | Some(Equipment::Kick | Equipment::Punch | Equipment::Bracelet),
            Equipment::WallPass,
        ) => (Some(7), true),
        (_, Equipment::Heart) if actor.equipment != Some(Equipment::Heart) => (Some(9), true),
        (Some(current), candidate) if current == candidate => (None, true),
        (None | Some(Equipment::Kick), Equipment::Punch | Equipment::Bracelet) => (Some(4), true),
        (
            Some(Equipment::Punch | Equipment::Bracelet),
            Equipment::Kick | Equipment::Punch | Equipment::Bracelet,
        ) => (None, false),
        _ => (None, actor.equipment != Some(Equipment::Heart)),
    })
}

pub(super) fn supported_weapon(
    actor: &VisibleActor,
    kind: PickupKind,
) -> Option<(Option<u8>, bool)> {
    let incoming = match kind {
        PickupKind::PiercingBomb => SpecialWeapon::Piercing,
        PickupKind::PowerBomb => SpecialWeapon::Power,
        _ => return None,
    };
    Some(match actor.weapon {
        None | Some(SpecialWeapon::Remote | SpecialWeapon::Landmine) => (Some(7), true),
        Some(current) if current == incoming => (None, true),
        Some(SpecialWeapon::Piercing | SpecialWeapon::Power) => (None, true),
    })
}
