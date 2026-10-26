use crate::game;

pub(super) const fn domain_effect(value: super::EffectKind) -> game::EffectKind {
    match value {
        super::EffectKind::Shield => game::EffectKind::Shield,
        super::EffectKind::Stopwatch => game::EffectKind::Stopwatch,
        super::EffectKind::InvisibilityCloak => game::EffectKind::InvisibilityCloak,
        super::EffectKind::GoldenFlame => game::EffectKind::GoldenFlame,
        super::EffectKind::ShortFuse => game::EffectKind::ShortFuse,
        super::EffectKind::Diarrhea => game::EffectKind::Diarrhea,
        super::EffectKind::SlowFoot => game::EffectKind::SlowFoot,
        super::EffectKind::ShortFlame => game::EffectKind::ShortFlame,
        super::EffectKind::LongFuse => game::EffectKind::LongFuse,
        super::EffectKind::ReverseControls => game::EffectKind::ReverseControls,
        super::EffectKind::InvisibleBombs => game::EffectKind::InvisibleBombs,
    }
}

pub(super) const fn domain_pickup(value: super::PickupKind) -> game::PickupKind {
    match value {
        super::PickupKind::BombCapacity => game::PickupKind::BombCapacity,
        super::PickupKind::BlastRange => game::PickupKind::BlastRange,
        super::PickupKind::MovementSpeed => game::PickupKind::MovementSpeed,
        super::PickupKind::Kick => game::PickupKind::Kick,
        super::PickupKind::Punch => game::PickupKind::Punch,
        super::PickupKind::Bracelet => game::PickupKind::Bracelet,
        super::PickupKind::WallPass => game::PickupKind::WallPass,
        super::PickupKind::Heart => game::PickupKind::Heart,
        super::PickupKind::RemoteBomb => game::PickupKind::RemoteBomb,
        super::PickupKind::PiercingBomb => game::PickupKind::PiercingBomb,
        super::PickupKind::PowerBomb => game::PickupKind::PowerBomb,
        super::PickupKind::Landmine => game::PickupKind::Landmine,
        super::PickupKind::Shield => game::PickupKind::Shield,
        super::PickupKind::Stopwatch => game::PickupKind::Stopwatch,
        super::PickupKind::InvisibilityCloak => game::PickupKind::InvisibilityCloak,
        super::PickupKind::GoldenFlame => game::PickupKind::GoldenFlame,
        super::PickupKind::ShortFuse => game::PickupKind::ShortFuse,
        super::PickupKind::Diarrhea => game::PickupKind::Diarrhea,
        super::PickupKind::SlowFoot => game::PickupKind::SlowFoot,
        super::PickupKind::ShortFlame => game::PickupKind::ShortFlame,
        super::PickupKind::LongFuse => game::PickupKind::LongFuse,
        super::PickupKind::ReverseControls => game::PickupKind::ReverseControls,
        super::PickupKind::OilBarrel => game::PickupKind::OilBarrel,
        super::PickupKind::Teleporter => game::PickupKind::Teleporter,
        super::PickupKind::InvisibleBombs => game::PickupKind::InvisibleBombs,
    }
}
