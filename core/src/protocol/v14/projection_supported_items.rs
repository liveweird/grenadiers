use super::{ItemId, ItemSet, RulesVersion};

pub(crate) fn available_items_for(version: RulesVersion) -> ItemSet {
    if version.get() < RulesVersion::EXPANDED_V5.get() {
        return ItemSet::empty();
    }
    ItemSet::new([
        ItemId::Kick,
        ItemId::Punch,
        ItemId::Bracelet,
        ItemId::WallPass,
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::Stopwatch,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
        ItemId::ReverseControls,
        ItemId::OilBarrel,
        ItemId::Teleporter,
    ])
    .expect("v14 item set is canonical")
}
