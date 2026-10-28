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
    use super::PickupKind as W;
    use game::PickupKind as D;
    match value {
        W::BombCapacity => D::BombCapacity,
        W::BlastRange => D::BlastRange,
        W::MovementSpeed => D::MovementSpeed,
        W::Kick => D::Kick,
        W::Punch => D::Punch,
        W::Bracelet => D::Bracelet,
        W::WallPass => D::WallPass,
        W::Heart => D::Heart,
        W::RemoteBomb => D::RemoteBomb,
        W::PiercingBomb => D::PiercingBomb,
        W::PowerBomb => D::PowerBomb,
        W::Landmine => D::Landmine,
        W::Shield => D::Shield,
        W::Stopwatch => D::Stopwatch,
        W::InvisibilityCloak => D::InvisibilityCloak,
        W::GoldenFlame => D::GoldenFlame,
        W::ShortFuse => D::ShortFuse,
        W::Diarrhea => D::Diarrhea,
        W::SlowFoot => D::SlowFoot,
        W::ShortFlame => D::ShortFlame,
        W::LongFuse => D::LongFuse,
        W::ReverseControls => D::ReverseControls,
        W::OilBarrel => D::OilBarrel,
        W::Teleporter => D::Teleporter,
        W::InvisibleBombs => D::InvisibleBombs,
    }
}
